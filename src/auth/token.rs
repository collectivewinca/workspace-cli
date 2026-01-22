use std::path::PathBuf;
use crate::config::Config;
use super::oauth::{self, AuthError, WorkspaceAuthenticator, SCOPES};
use super::keyring_storage::{KeyringError, TokenStorage, StoredToken};

/// Manages authentication and token lifecycle
pub struct TokenManager {
    authenticator: Option<WorkspaceAuthenticator>,
    storage: TokenStorage,
    config: Config,
    credentials_path: Option<PathBuf>,
    current_account: String,
}

impl TokenManager {
    /// Create a new token manager with the given config
    pub fn new(config: Config) -> Self {
        let account = config.auth.current_account
            .clone()
            .unwrap_or_else(|| "default".to_string());

        Self {
            authenticator: None,
            storage: TokenStorage::new(&account),
            credentials_path: None,
            current_account: account,
            config,
        }
    }

    /// Create a new token manager for a specific account
    pub fn new_for_account(config: Config, account: &str) -> Self {
        Self {
            authenticator: None,
            storage: TokenStorage::new(account),
            credentials_path: None,
            current_account: account.to_string(),
            config,
        }
    }

    /// Try to restore authenticator from cached tokens
    /// Call this before making API requests
    pub async fn ensure_authenticated(&mut self) -> Result<(), TokenManagerError> {
        // Already have an authenticator
        if self.authenticator.is_some() {
            // Validate that the authenticator can still get tokens
            // This checks for expiry and refreshes if needed
            if let Ok(_) = self.get_access_token().await {
                return Ok(());
            }
            // If token fetch fails, clear the authenticator and retry
            self.authenticator = None;
        }

        let token_cache = self.token_cache_path();

        // Check if token cache exists
        if !token_cache.exists() {
            return Err(TokenManagerError::NotAuthenticated);
        }

        // Try to find credentials path for this account
        let creds_path = self.get_credentials_path_for_account()
            .or_else(|| self.credentials_path.clone())
            .or_else(|| self.config.auth.credentials_path.clone())
            .or_else(|| self.find_credentials_file());

        let creds_path = creds_path.ok_or(TokenManagerError::MissingCredentials(
            "No credentials file found. Run 'workspace-cli auth login --credentials <path>' first.".to_string()
        ))?;

        // Restore authenticator from cached tokens
        let mut auth = oauth::create_installed_flow_auth(&creds_path, &token_cache)
            .await
            .map_err(TokenManagerError::Auth)?;

        // Verify we can get a token before considering authentication successful
        oauth::get_token(&mut auth, SCOPES)
            .await
            .map_err(TokenManagerError::Auth)?;

        self.authenticator = Some(auth);
        self.credentials_path = Some(creds_path);
        Ok(())
    }

    /// Get credentials path for specific account from config
    fn get_credentials_path_for_account(&self) -> Option<PathBuf> {
        self.config.auth.accounts
            .get(&self.current_account)
            .cloned()
    }

    /// Find credentials file in common locations
    fn find_credentials_file(&self) -> Option<PathBuf> {
        let candidates = [
            // Current directory
            PathBuf::from("credentials.json"),
            // Config directory
            Config::config_dir().map(|d| d.join("credentials.json")).unwrap_or_default(),
            // Home directory
            dirs::home_dir().map(|d| d.join("credentials.json")).unwrap_or_default(),
            dirs::home_dir().map(|d| d.join(".credentials.json")).unwrap_or_default(),
        ];

        candidates.into_iter().find(|p| p.exists())
    }

    /// Initialize with interactive OAuth2 flow
    pub async fn login_interactive(&mut self, credentials_path: Option<PathBuf>) -> Result<(), TokenManagerError> {
        let creds_path = credentials_path
            .or_else(|| self.config.auth.credentials_path.clone())
            .ok_or_else(|| TokenManagerError::MissingCredentials(
                "No credentials path provided. Use --credentials or set WORKSPACE_CREDENTIALS_PATH".to_string()
            ))?;

        // Validate that credentials file exists
        if !creds_path.exists() {
            return Err(TokenManagerError::MissingCredentials(
                format!("Credentials file not found at: {}", creds_path.display())
            ));
        }

        let token_cache = self.token_cache_path();

        // Ensure the config directory exists
        if let Some(parent) = token_cache.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                TokenManagerError::MissingCredentials(format!("Failed to create config directory: {}", e))
            })?;
        }

        let mut auth = oauth::create_installed_flow_auth(&creds_path, &token_cache)
            .await
            .map_err(TokenManagerError::Auth)?;

        // Get a token (this will trigger the OAuth flow if needed)
        let token = oauth::get_token(&mut auth, SCOPES)
            .await
            .map_err(TokenManagerError::Auth)?;

        // Store token info with current timestamp + estimated expiry (3600 seconds is typical)
        let expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()
            .map(|d| (d.as_secs() + 3600) as i64);

        self.storage.store(&StoredToken {
            access_token: token.clone(),
            refresh_token: None, // Our authenticator handles refresh internally
            expires_at,
        }).map_err(TokenManagerError::Storage)?;

        self.authenticator = Some(auth);
        self.credentials_path = Some(creds_path);
        Ok(())
    }

    /// Get an access token for API calls
    pub async fn get_access_token(&mut self) -> Result<String, TokenManagerError> {
        let auth = self.authenticator.as_mut()
            .ok_or(TokenManagerError::NotAuthenticated)?;

        oauth::get_token(auth, SCOPES)
            .await
            .map_err(TokenManagerError::Auth)
    }

    /// Get token for specific scopes
    pub async fn get_token_for_scopes(&mut self, scopes: &[&str]) -> Result<String, TokenManagerError> {
        let auth = self.authenticator.as_mut()
            .ok_or(TokenManagerError::NotAuthenticated)?;

        oauth::get_token(auth, scopes)
            .await
            .map_err(TokenManagerError::Auth)
    }

    /// Check if we have stored credentials
    pub fn is_authenticated(&self) -> bool {
        self.authenticator.is_some() || self.token_cache_path().exists()
    }

    /// Clear all stored tokens (logout)
    pub fn logout(&mut self) -> Result<(), TokenManagerError> {
        // Clear the authenticator to free resources
        self.authenticator = None;
        self.credentials_path = None;

        self.storage.delete().map_err(TokenManagerError::Storage)?;

        // Also try to remove the token cache file
        let cache_path = self.token_cache_path();
        if cache_path.exists() {
            std::fs::remove_file(cache_path)
                .map_err(|e| TokenManagerError::Storage(
                    KeyringError::DeleteFailed(format!("Failed to remove token cache: {}", e))
                ))?;
        }

        Ok(())
    }

    /// Get authentication status info
    pub fn status(&self) -> AuthStatus {
        AuthStatus {
            authenticated: self.is_authenticated(),
            storage_type: self.storage.storage_type().to_string(),
            token_cache_path: self.token_cache_path(),
        }
    }

    /// Get the token cache file path (account-specific)
    fn token_cache_path(&self) -> PathBuf {
        Config::config_dir()
            .map(|d| d.join(format!("token_cache_{}.json", self.current_account)))
            .unwrap_or_else(|| PathBuf::from(format!("token_cache_{}.json", self.current_account)))
    }

    /// Get current account identifier
    pub fn current_account(&self) -> &str {
        &self.current_account
    }

    /// List all authenticated accounts
    pub fn list_accounts() -> Result<Vec<String>, TokenManagerError> {
        let config_dir = Config::config_dir()
            .ok_or_else(|| TokenManagerError::Other("Could not determine config directory".into()))?;

        let mut accounts = Vec::new();

        // Find all token_cache_*.json files
        if let Ok(entries) = std::fs::read_dir(&config_dir) {
            for entry in entries.flatten() {
                let filename = entry.file_name();
                let filename_str = filename.to_string_lossy();

                if filename_str.starts_with("token_cache_") && filename_str.ends_with(".json") {
                    // Extract account name from token_cache_{account}.json
                    let account = filename_str
                        .strip_prefix("token_cache_")
                        .and_then(|s| s.strip_suffix(".json"))
                        .map(|s| s.to_string());

                    if let Some(acc) = account {
                        accounts.push(acc);
                    }
                }
            }
        }

        Ok(accounts)
    }
}

/// Authentication status information
#[derive(Debug, Clone, serde::Serialize)]
pub struct AuthStatus {
    pub authenticated: bool,
    pub storage_type: String,
    pub token_cache_path: PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum TokenManagerError {
    #[error("Not authenticated. Run 'workspace-cli auth login' first.")]
    NotAuthenticated,

    #[error("Missing credentials: {0}")]
    MissingCredentials(String),

    #[error("Authentication error: {0}")]
    Auth(#[from] AuthError),

    #[error("Token storage error: {0}")]
    Storage(#[from] KeyringError),

    #[error("{0}")]
    Other(String),
}
