# Multi-Account Support Implementation Summary

## Changes Made

### 1. Configuration (`src/config/settings.rs`)

**Added fields to `AuthConfig`:**
```rust
pub struct AuthConfig {
    // ... existing fields ...
    
    /// Current active account (email or identifier)
    pub current_account: Option<String>,
    
    /// Map of account names to their credentials paths
    pub accounts: std::collections::HashMap<String, PathBuf>,
}
```

**Why:** Track which account is currently active and remember credentials for each account.

---

### 2. Token Manager (`src/auth/token.rs`)

**Added field to `TokenManager`:**
```rust
pub struct TokenManager {
    // ... existing fields ...
    current_account: String,
}
```

**New methods:**
- `new_for_account(config, account)` - Create token manager for specific account
- `current_account()` - Get current account name
- `list_accounts()` - List all authenticated accounts (static method)

**Modified methods:**
- `new()` - Now uses account from config (defaults to "default")
- `token_cache_path()` - Now account-specific: `token_cache_{account}.json`

**New error variant:**
- `TokenManagerError::Other(String)` - For general errors

**Why:** Each account needs its own token storage and the manager needs to know which account it's managing.

---

### 3. Auth Commands (`src/main.rs`)

**Updated `AuthCommands` enum:**

```rust
enum AuthCommands {
    Login {
        credentials: Option<String>,
        account: Option<String>,  // NEW: specify account name
    },
    Logout {
        account: Option<String>,  // NEW: logout specific account
        all: bool,                // NEW: logout all accounts
    },
    Status,
    Accounts,  // NEW: list all accounts
    Switch {   // NEW: switch between accounts
        account: String,
    },
}
```

**Updated command handlers:**

1. **Login** - Now creates account-specific token manager and saves account to config
2. **Logout** - Supports `--account` and `--all` flags
3. **Status** - Shows current account name
4. **Accounts** - NEW: Lists all authenticated accounts with current indicator
5. **Switch** - NEW: Changes current account in config

**Why:** Users need commands to manage multiple accounts.

---

### 4. Documentation

**Updated files:**
- `README.md` - Added multi-account section and updated auth commands table
- `src/main.rs` - Updated help text with multi-account examples
- `MULTI_ACCOUNT_GUIDE.md` - NEW: Comprehensive guide for multi-account usage

---

## File Structure Changes

### Before (Single Account)
```
~/.config/workspace-cli/
├── config.toml
├── token_cache.json              # Single cache file
└── tokens_default.json           # Single token file
```

### After (Multi Account)
```
~/.config/workspace-cli/
├── config.toml                   # Tracks current_account
├── token_cache_default.json      # Per-account cache files
├── token_cache_work@company.com.json
├── token_cache_personal@gmail.com.json
├── tokens_default.json           # Per-account token files
├── tokens_work@company.com.json
└── tokens_personal@gmail.com.json
```

Each account has isolated storage with account name in filename.

---

## Backward Compatibility

✅ **Fully backward compatible!**

- Existing users with no account specified will use `"default"` as account name
- Old `token_cache.json` will be migrated to `token_cache_default.json` on first use
- No breaking changes to existing commands

---

## How to Build and Test

### Build the project
```bash
cd D:\Modules\workspace-cli
cargo build
```

### Install locally
```bash
cargo install --path .
```

### Test multi-account functionality

1. **Login with first account:**
   ```bash
   workspace-cli auth login --credentials credentials.json --account labh@collectivewin.ca
   ```

2. **Login with second account:**
   ```bash
   workspace-cli auth login --credentials credentials.json --account test@example.com
   ```

3. **List accounts:**
   ```bash
   workspace-cli auth accounts
   ```

4. **Check status:**
   ```bash
   workspace-cli auth status
   ```

5. **Switch accounts:**
   ```bash
   workspace-cli auth switch test@example.com
   workspace-cli auth status
   ```

6. **Use Gmail with switched account:**
   ```bash
   workspace-cli gmail list --limit 5
   ```

7. **Logout specific account:**
   ```bash
   workspace-cli auth logout --account test@example.com
   ```

---

## Technical Details

### Token Storage Strategy

Each account uses the existing dual-storage strategy:

1. **Primary**: OS Keyring with account-specific key
   - Service: `workspace-cli`
   - Account: `{account_name}` (e.g., `work@company.com`)

2. **Fallback**: File storage with account-specific filename
   - Path: `~/.config/workspace-cli/tokens_{account_name}.json`

### Account Discovery

The `list_accounts()` method discovers accounts by:
1. Scanning config directory for `token_cache_*.json` files
2. Extracting account names from filenames
3. Returns list of all accounts with valid token caches

### Current Account Tracking

The active account is stored in `config.toml`:
```toml
[auth]
current_account = "work@company.com"

[auth.accounts]
"work@company.com" = "D:\\path\\to\\credentials.json"
"personal@gmail.com" = "D:\\path\\to\\credentials.json"
```

This allows:
- Persistence across CLI invocations
- Remembering which credentials each account used
- Fast account switching without token regeneration

---

## Benefits

1. **No More Logout/Login Dance** 🎉
   - Switch accounts instantly
   - No browser authentication on switch
   - Tokens remain valid for all accounts

2. **Better Organization** 📁
   - Clear separation between work/personal
   - Each account has isolated storage
   - Easy to identify which account is active

3. **Scriptable** 🤖
   ```bash
   # Process multiple accounts in a loop
   for account in work@company.com personal@gmail.com; do
       workspace-cli auth switch "$account"
       workspace-cli gmail list --query "is:unread"
   done
   ```

4. **Secure** 🔒
   - Each account uses separate keyring entries
   - No token cross-contamination
   - Logout one account doesn't affect others

---

## Testing Checklist

After building, verify these scenarios:

- [ ] Login with account name
- [ ] Login without account name (auto-generated)
- [ ] List accounts shows all authenticated accounts
- [ ] Switch between accounts works
- [ ] Auth status shows correct current account
- [ ] Gmail/Drive commands use switched account
- [ ] Logout specific account works
- [ ] Logout all accounts works
- [ ] Token files are account-specific
- [ ] Config.toml tracks current account
- [ ] Backward compatibility (existing "default" account works)

---

## Questions?

See `MULTI_ACCOUNT_GUIDE.md` for user-facing documentation and examples.

---

**Implementation complete!** ✅
