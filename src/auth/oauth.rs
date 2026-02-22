use std::path::Path;
use std::net::TcpListener;
use std::io::{Read, Write};
use serde::{Deserialize, Serialize};

/// Fixed port for OAuth redirect - using 127.0.0.1 which Google accepts with any port
pub const OAUTH_REDIRECT_PORT: u16 = 8085;
pub const OAUTH_REDIRECT_URI: &str = "http://127.0.0.1:8085";

/// All scopes needed for Google Workspace APIs
pub const SCOPES: &[&str] = &[
    "https://www.googleapis.com/auth/gmail.modify",
    "https://www.googleapis.com/auth/drive",
    "https://www.googleapis.com/auth/calendar",
    "https://www.googleapis.com/auth/documents",
    "https://www.googleapis.com/auth/spreadsheets",
    "https://www.googleapis.com/auth/presentations",
    "https://www.googleapis.com/auth/tasks",
];

/// OAuth2 credentials from credentials.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthCredentials {
    pub client_id: String,
    pub client_secret: String,
    pub auth_uri: String,
    pub token_uri: String,
    pub redirect_uris: Vec<String>,
    pub project_id: Option<String>,
}

/// Token response from Google OAuth2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<u64>,
    pub token_type: Option<String>,
    pub scope: Option<String>,
}

/// Stored token with expiry information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthToken {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<i64>, // Unix timestamp
}

impl OAuthToken {
    /// Check if the token is expired (with 60 second buffer)
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            expires_at <= now + 60 // 60 second buffer
        } else {
            false // If no expiry, assume not expired
        }
    }
}

/// Custom OAuth2 authenticator (replaces yup-oauth2)
pub struct WorkspaceAuthenticator {
    credentials: OAuthCredentials,
    token: Option<OAuthToken>,
    token_cache_path: std::path::PathBuf,
    http_client: reqwest::Client,
}

impl WorkspaceAuthenticator {
    /// Create a new authenticator
    pub fn new(credentials: OAuthCredentials, token_cache_path: std::path::PathBuf) -> Self {
        Self {
            credentials,
            token: None,
            token_cache_path,
            http_client: reqwest::Client::new(),
        }
    }

    /// Load cached token from disk
    pub fn load_cached_token(&mut self) -> Result<(), AuthError> {
        if self.token_cache_path.exists() {
            let content = std::fs::read_to_string(&self.token_cache_path)
                .map_err(|e| AuthError::StorageError(format!("Failed to read token cache: {}", e)))?;

            // Try to parse the yup-oauth2 format first (for backwards compatibility)
            if let Ok(tokens) = serde_json::from_str::<Vec<serde_json::Value>>(&content) {
                if let Some(first) = tokens.first() {
                    if let Some(token_obj) = first.get("token") {
                        let access_token = token_obj.get("access_token")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        let refresh_token = token_obj.get("refresh_token")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());

                        // Parse expires_at from yup-oauth2 format [year, day, month, hour, min, nano, ...]
                        let expires_at = token_obj.get("expires_at")
                            .and_then(|v| v.as_array())
                            .map(|arr| {
                                // Simplified: just use current time + 1 hour if we can't parse
                                std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .map(|d| d.as_secs() as i64 + 3600)
                                    .unwrap_or(0)
                            });

                        if let Some(access) = access_token {
                            self.token = Some(OAuthToken {
                                access_token: access,
                                refresh_token,
                                expires_at,
                            });
                            return Ok(());
                        }
                    }
                }
            }

            // Try our own format
            if let Ok(token) = serde_json::from_str::<OAuthToken>(&content) {
                self.token = Some(token);
                return Ok(());
            }

            return Err(AuthError::StorageError("Failed to parse token cache".to_string()));
        }
        Err(AuthError::StorageError("No token cache found".to_string()))
    }

    /// Save token to disk
    fn save_token(&self, token: &OAuthToken) -> Result<(), AuthError> {
        if let Some(parent) = self.token_cache_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| AuthError::StorageError(format!("Failed to create cache dir: {}", e)))?;
        }

        let content = serde_json::to_string_pretty(token)
            .map_err(|e| AuthError::StorageError(format!("Failed to serialize token: {}", e)))?;

        std::fs::write(&self.token_cache_path, content)
            .map_err(|e| AuthError::StorageError(format!("Failed to write token cache: {}", e)))?;

        Ok(())
    }

    /// Build the authorization URL
    pub fn build_auth_url(&self) -> String {
        let scopes = SCOPES.join(" ");
        format!(
            "{}?response_type=code&client_id={}&redirect_uri={}&scope={}&access_type=offline&prompt=consent",
            self.credentials.auth_uri,
            urlencoding::encode(&self.credentials.client_id),
            urlencoding::encode(OAUTH_REDIRECT_URI),
            urlencoding::encode(&scopes)
        )
    }

    /// Run the interactive OAuth flow
    pub async fn run_auth_flow(&mut self) -> Result<OAuthToken, AuthError> {
        let auth_url = self.build_auth_url();

        println!("Please open this URL in your browser to authorize:");
        println!("\n{}\n", auth_url);
        println!("Waiting for authorization on {}...", OAUTH_REDIRECT_URI);

        // Start local server to capture the callback
        let code = self.capture_auth_code()?;

        println!("Authorization code received, exchanging for token...");

        // Exchange code for token
        let token = self.exchange_code(&code).await?;

        // Save and store token
        self.save_token(&token)?;
        self.token = Some(token.clone());

        Ok(token)
    }

    /// Start a local server and capture the auth code
    fn capture_auth_code(&self) -> Result<String, AuthError> {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", OAUTH_REDIRECT_PORT))
            .map_err(|e| AuthError::FlowFailed(format!("Failed to bind to port {}: {}", OAUTH_REDIRECT_PORT, e)))?;

        // Accept one connection
        let (mut stream, _) = listener.accept()
            .map_err(|e| AuthError::FlowFailed(format!("Failed to accept connection: {}", e)))?;

        // Read the request
        let mut buffer = [0; 4096];
        let n = stream.read(&mut buffer)
            .map_err(|e| AuthError::FlowFailed(format!("Failed to read request: {}", e)))?;

        let request = String::from_utf8_lossy(&buffer[..n]);

        // Extract the code from the query string
        let code = request
            .lines()
            .next()
            .and_then(|line| {
                // GET /?code=...&scope=... HTTP/1.1
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let path = parts[1];
                    if let Some(query_start) = path.find('?') {
                        let query = &path[query_start + 1..];
                        for param in query.split('&') {
                            let kv: Vec<&str> = param.splitn(2, '=').collect();
                            if kv.len() == 2 && kv[0] == "code" {
                                return Some(urlencoding::decode(kv[1]).unwrap_or_default().to_string());
                            }
                        }
                    }
                }
                None
            })
            .ok_or_else(|| AuthError::FlowFailed("No authorization code in callback".to_string()))?;

        // Send success response
        let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n\
            <html><body><h1>Authorization successful!</h1>\
            <p>You can close this window and return to the terminal.</p></body></html>";

        let _ = stream.write_all(response.as_bytes());

        Ok(code)
    }

    /// Exchange authorization code for tokens
    async fn exchange_code(&self, code: &str) -> Result<OAuthToken, AuthError> {
        let params = [
            ("code", code),
            ("client_id", &self.credentials.client_id),
            ("client_secret", &self.credentials.client_secret),
            ("redirect_uri", OAUTH_REDIRECT_URI),
            ("grant_type", "authorization_code"),
        ];

        let response = self.http_client
            .post(&self.credentials.token_uri)
            .form(&params)
            .send()
            .await
            .map_err(|e| AuthError::FlowFailed(format!("Token exchange request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AuthError::FlowFailed(format!("Token exchange failed: {}", error_text)));
        }

        let token_response: TokenResponse = response.json().await
            .map_err(|e| AuthError::FlowFailed(format!("Failed to parse token response: {}", e)))?;

        let expires_at = token_response.expires_in.map(|secs| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64 + secs as i64)
                .unwrap_or(0)
        });

        Ok(OAuthToken {
            access_token: token_response.access_token,
            refresh_token: token_response.refresh_token,
            expires_at,
        })
    }

    /// Refresh the access token
    async fn refresh_token(&mut self) -> Result<OAuthToken, AuthError> {
        let refresh_token = self.token
            .as_ref()
            .and_then(|t| t.refresh_token.as_ref())
            .ok_or_else(|| AuthError::TokenFailed("No refresh token available".to_string()))?
            .clone();

        let params = [
            ("refresh_token", refresh_token.as_str()),
            ("client_id", &self.credentials.client_id),
            ("client_secret", &self.credentials.client_secret),
            ("grant_type", "refresh_token"),
        ];

        let response = self.http_client
            .post(&self.credentials.token_uri)
            .form(&params)
            .send()
            .await
            .map_err(|e| AuthError::TokenFailed(format!("Refresh request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AuthError::TokenFailed(format!("Token refresh failed: {}", error_text)));
        }

        let token_response: TokenResponse = response.json().await
            .map_err(|e| AuthError::TokenFailed(format!("Failed to parse refresh response: {}", e)))?;

        let expires_at = token_response.expires_in.map(|secs| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64 + secs as i64)
                .unwrap_or(0)
        });

        // Keep the old refresh token if a new one wasn't provided
        let new_token = OAuthToken {
            access_token: token_response.access_token,
            refresh_token: token_response.refresh_token.or_else(|| {
                self.token.as_ref().and_then(|t| t.refresh_token.clone())
            }),
            expires_at,
        };

        self.save_token(&new_token)?;
        self.token = Some(new_token.clone());

        Ok(new_token)
    }

    /// Get a valid access token (refreshing if needed)
    pub async fn get_token(&mut self, _scopes: &[&str]) -> Result<String, AuthError> {
        // Try to load cached token if we don't have one
        if self.token.is_none() {
            let _ = self.load_cached_token();
        }

        if let Some(ref token) = self.token {
            if !token.is_expired() {
                return Ok(token.access_token.clone());
            }

            // Try to refresh
            if token.refresh_token.is_some() {
                match self.refresh_token().await {
                    Ok(new_token) => return Ok(new_token.access_token),
                    Err(_) => {
                        // Refresh failed, need to re-authenticate
                    }
                }
            }
        }

        // No valid token, need to run auth flow
        let token = self.run_auth_flow().await?;
        Ok(token.access_token)
    }
}

/// Create an authenticator using OAuth2 installed application flow (interactive)
pub async fn create_installed_flow_auth(
    credentials_path: &Path,
    token_cache_path: &Path,
) -> Result<WorkspaceAuthenticator, AuthError> {
    let credentials = read_oauth_credentials(credentials_path).await?;
    let mut auth = WorkspaceAuthenticator::new(credentials, token_cache_path.to_path_buf());

    // Try to load cached token
    let _ = auth.load_cached_token();

    Ok(auth)
}

/// Read OAuth2 credentials from credentials.json
async fn read_oauth_credentials(path: &Path) -> Result<OAuthCredentials, AuthError> {
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| AuthError::InvalidCredentials(format!("Failed to read credentials: {}", e)))?;

    let secret: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| AuthError::InvalidCredentials(format!("Invalid JSON: {}", e)))?;

    // Handle both "installed" and "web" application types
    let creds = if let Some(installed) = secret.get("installed") {
        parse_credentials_object(installed)?
    } else if let Some(web) = secret.get("web") {
        parse_credentials_object(web)?
    } else {
        return Err(AuthError::InvalidCredentials(
            "credentials.json must contain 'installed' or 'web' key".to_string()
        ));
    };

    Ok(creds)
}

fn parse_credentials_object(obj: &serde_json::Value) -> Result<OAuthCredentials, AuthError> {
    let get_str = |key: &str| -> Result<String, AuthError> {
        obj.get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| AuthError::InvalidCredentials(format!("Missing field: {}", key)))
    };

    let get_str_opt = |key: &str| -> Option<String> {
        obj.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
    };

    let get_vec = |key: &str| -> Vec<String> {
        obj.get(key)
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default()
    };

    let client_id = get_str("client_id")?;
    let client_secret = get_str("client_secret")?;

    if client_id.trim().is_empty() {
        return Err(AuthError::InvalidCredentials("client_id cannot be empty".to_string()));
    }
    if client_secret.trim().is_empty() {
        return Err(AuthError::InvalidCredentials("client_secret cannot be empty".to_string()));
    }

    Ok(OAuthCredentials {
        client_id,
        client_secret,
        auth_uri: get_str_opt("auth_uri").unwrap_or_else(|| "https://accounts.google.com/o/oauth2/auth".to_string()),
        token_uri: get_str_opt("token_uri").unwrap_or_else(|| "https://oauth2.googleapis.com/token".to_string()),
        redirect_uris: get_vec("redirect_uris"),
        project_id: get_str_opt("project_id"),
    })
}

/// Get an access token for the given scopes
pub async fn get_token(
    auth: &mut WorkspaceAuthenticator,
    scopes: &[&str],
) -> Result<String, AuthError> {
    auth.get_token(scopes).await
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid credentials: {0}")]
    InvalidCredentials(String),

    #[error("Authentication flow failed: {0}")]
    FlowFailed(String),

    #[error("Failed to get token: {0}")]
    TokenFailed(String),

    #[error("Token storage error: {0}")]
    StorageError(String),
}
