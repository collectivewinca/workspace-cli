# Multi-Account Support Guide

## Overview

`workspace-cli` now supports managing multiple Google accounts simultaneously! No more logging out and logging back in to switch accounts.

## Key Features

✅ **Multiple Accounts**: Login with as many Google accounts as you need  
✅ **Instant Switching**: Switch between accounts without re-authentication  
✅ **Isolated Storage**: Each account has its own secure token storage  
✅ **Easy Management**: List, switch, and logout accounts with simple commands  

---

## Quick Start

### 1. Login with Multiple Accounts

```bash
# Login with your first account (work email)
workspace-cli auth login --credentials credentials.json --account work@company.com

# Login with your second account (personal email)
workspace-cli auth login --credentials credentials.json --account personal@gmail.com

# Login with third account (client email)
workspace-cli auth login --credentials credentials.json --account client@project.com
```

**Note**: If you don't specify `--account`, it will auto-generate an account name like `account_1737072000`.

### 2. List All Your Accounts

```bash
workspace-cli auth accounts
```

**Output:**
```json
{
  "accounts": [
    {"name": "work@company.com", "current": true},
    {"name": "personal@gmail.com", "current": false},
    {"name": "client@project.com", "current": false}
  ]
}
```

### 3. Switch Between Accounts

```bash
# Switch to personal account
workspace-cli auth switch personal@gmail.com

# Now all commands use personal@gmail.com
workspace-cli gmail list --limit 5

# Switch to work account
workspace-cli auth switch work@company.com

# Now all commands use work@company.com
workspace-cli gmail list --limit 5
```

**Switching is instant** - no browser popup, no re-authentication!

### 4. Check Current Account Status

```bash
workspace-cli auth status
```

**Output:**
```json
{
  "authenticated": true,
  "current_account": "work@company.com",
  "storage_type": "keyring",
  "token_cache_path": "C:\\Users\\username\\.config\\workspace-cli\\token_cache_work@company.com.json"
}
```

### 5. Logout Accounts

```bash
# Logout current account only
workspace-cli auth logout

# Logout specific account
workspace-cli auth logout --account personal@gmail.com

# Logout ALL accounts at once
workspace-cli auth logout --all
```

---

## How It Works

### Token Storage

Each account has its own isolated token storage:

**Windows:**
- Primary: Windows Credential Manager (secure)
- Fallback: `%APPDATA%\workspace-cli\tokens_{account}.json`
- Cache: `%APPDATA%\workspace-cli\token_cache_{account}.json`

**macOS:**
- Primary: Keychain (secure)
- Fallback: `~/.config/workspace-cli/tokens_{account}.json`
- Cache: `~/.config/workspace-cli/token_cache_{account}.json`

**Linux:**
- Primary: Secret Service / GNOME Keyring (secure)
- Fallback: `~/.config/workspace-cli/tokens_{account}.json`
- Cache: `~/.config/workspace-cli/token_cache_{account}.json`

### Current Account Tracking

The currently active account is stored in:
```
~/.config/workspace-cli/config.toml
```

**Example config:**
```toml
[auth]
current_account = "work@company.com"

[auth.accounts]
"work@company.com" = "/path/to/credentials.json"
"personal@gmail.com" = "/path/to/credentials.json"
```

---

## Common Workflows

### Workflow 1: Work and Personal Email Management

```bash
# Setup (do once)
workspace-cli auth login --credentials creds.json --account work@company.com
workspace-cli auth login --credentials creds.json --account personal@gmail.com

# Daily usage - check work emails
workspace-cli auth switch work@company.com
workspace-cli gmail list --query "is:unread" --limit 10

# Check personal emails
workspace-cli auth switch personal@gmail.com
workspace-cli gmail list --query "is:unread" --limit 10
```

### Workflow 2: Multiple Client Projects

```bash
# Setup accounts for each client
workspace-cli auth login --credentials creds.json --account client-acme@agency.com
workspace-cli auth login --credentials creds.json --account client-globex@agency.com
workspace-cli auth login --credentials creds.json --account client-initech@agency.com

# Work on Acme project
workspace-cli auth switch client-acme@agency.com
workspace-cli drive list --query "name contains 'Acme Project'"

# Switch to Globex project
workspace-cli auth switch client-globex@agency.com
workspace-cli drive list --query "name contains 'Globex Proposal'"
```

### Workflow 3: Scripting with Multiple Accounts

```bash
#!/bin/bash

# Process emails from multiple accounts
for account in "sales@company.com" "support@company.com" "admin@company.com"; do
    echo "Processing $account..."
    workspace-cli auth switch "$account"
    workspace-cli gmail list --query "is:unread" --format jsonl > "${account}_unread.jsonl"
done

# Switch back to default
workspace-cli auth switch sales@company.com
```

---

## Migration from Single Account

If you were using workspace-cli before this update, your existing tokens are automatically available as the `default` account:

```bash
# Your old tokens are now the "default" account
workspace-cli auth status
# Output: "current_account": "default"

# Rename it to something meaningful
workspace-cli auth logout  # Logout default
workspace-cli auth login --credentials creds.json --account myemail@gmail.com
```

Or simply add new accounts and the old one continues working:

```bash
# Your existing "default" account still works
workspace-cli gmail list --limit 5

# Add a new account
workspace-cli auth login --credentials creds.json --account second@gmail.com

# List shows both
workspace-cli auth accounts
# Output:
# {
#   "accounts": [
#     {"name": "default", "current": true},
#     {"name": "second@gmail.com", "current": false}
#   ]
# }
```

---

## Troubleshooting

### Issue: "Account not found"

**Problem:** Trying to switch to an account that doesn't exist.

**Solution:**
```bash
# List available accounts
workspace-cli auth accounts

# Login if the account doesn't exist
workspace-cli auth login --credentials creds.json --account newaccount@gmail.com
```

### Issue: Token expired

**Problem:** Token expired for a specific account.

**Solution:**
```bash
# Just login again with the same account name
workspace-cli auth login --credentials creds.json --account expired@gmail.com
```

### Issue: Want to clean up old accounts

**Problem:** Have too many old accounts.

**Solution:**
```bash
# List accounts
workspace-cli auth accounts

# Remove specific accounts
workspace-cli auth logout --account oldaccount@gmail.com

# Or clean up everything and start fresh
workspace-cli auth logout --all
```

---

## Best Practices

1. **Use Descriptive Account Names**
   ```bash
   # Good
   workspace-cli auth login --credentials creds.json --account work-john@company.com
   workspace-cli auth login --credentials creds.json --account personal-john@gmail.com
   
   # Avoid
   workspace-cli auth login --credentials creds.json  # Creates "account_1234567890"
   ```

2. **Keep Track of Your Accounts**
   ```bash
   # Periodically check what accounts you have
   workspace-cli auth accounts
   ```

3. **Clean Up Unused Accounts**
   ```bash
   # Remove accounts you no longer use
   workspace-cli auth logout --account oldproject@company.com
   ```

4. **Use Config File for Automation**
   - Store your current account preference in scripts
   - Use environment variables for CI/CD pipelines

---

## Advanced: Account-Specific Credentials

You can use different OAuth credentials for different accounts:

```bash
# Work account with work credentials
workspace-cli auth login --credentials work-creds.json --account work@company.com

# Personal account with personal credentials
workspace-cli auth login --credentials personal-creds.json --account personal@gmail.com
```

Each account remembers which credentials file it used, stored in `config.toml`.

---

## Command Reference

### auth login

Login with a Google account:

```bash
workspace-cli auth login --credentials <path> [--account <name>]
```

**Options:**
- `--credentials`: Path to OAuth2 credentials JSON (required)
- `--account`: Account identifier/email (optional, auto-generated if not specified)

### auth accounts

List all authenticated accounts:

```bash
workspace-cli auth accounts
```

**Output:** JSON array of accounts with current account indicator.

### auth switch

Switch to a different account:

```bash
workspace-cli auth switch <account>
```

**Arguments:**
- `account`: Account name to switch to (must exist)

### auth status

Show current authentication status:

```bash
workspace-cli auth status
```

**Output:** JSON object with current account, storage type, and token path.

### auth logout

Logout one or all accounts:

```bash
# Logout current account
workspace-cli auth logout

# Logout specific account
workspace-cli auth logout --account <name>

# Logout all accounts
workspace-cli auth logout --all
```

**Options:**
- `--account`: Specific account to logout
- `--all`: Logout all accounts

---

## What's Next?

With multi-account support, you can:

- ✅ Manage work and personal emails separately
- ✅ Handle multiple client projects efficiently
- ✅ Test with different Google Workspace organizations
- ✅ Switch contexts without re-authentication delays

**Happy multi-accounting!** 🚀
