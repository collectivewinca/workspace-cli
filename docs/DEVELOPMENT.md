# Development & Update Guide

This guide explains how to build, test, and install workspace-cli after making code changes.

## Quick Reference

```bash
# After making code changes:
cd D:\Modules\workspace-cli
cargo build                          # Test compilation
cargo install --path .               # Install updated version
workspace-cli gmail send --help      # Verify changes
```

---

## Understanding the Installation System

### How Cargo Install Works

When you run `cargo install --path .`, here's what happens:

```
1. Source Code (D:\Modules\workspace-cli\src\)
   ↓
2. cargo install --path . compiles the code
   ↓
3. Binary is placed in: C:\Users\labhk\.cargo\bin\workspace-cli.exe
   ↓
4. Replaces any existing workspace-cli.exe in that location
   ↓
5. Windows PATH includes C:\Users\labhk\.cargo\bin
   ↓
6. You can run: workspace-cli (from anywhere in the terminal)
```

### Key Locations

| Location | Purpose |
|----------|---------|
| `D:\Modules\workspace-cli\` | Source code repository |
| `D:\Modules\workspace-cli\target\debug\` | Debug builds (fast compilation) |
| `D:\Modules\workspace-cli\target\release\` | Release builds (optimized) |
| `C:\Users\labhk\.cargo\bin\` | Installed binaries (in PATH) |

---

## Development Workflow

### 1. Make Code Changes

Edit files in `src/` directory:
```
src/
├── main.rs              # CLI argument parsing
├── commands/
│   └── gmail/
│       └── send.rs      # Email sending logic
└── ...
```

### 2. Check for Compilation Errors

```bash
# Quick check (debug build, faster)
cargo build

# Check for linting issues
cargo clippy

# Format code
cargo fmt
```

### 3. Test Your Changes

**Option A: Run without installing (for quick testing)**
```bash
cargo run -- gmail send --to test@example.com --subject "Test" --body "Hello" --html
```

**Option B: Install and test**
```bash
cargo install --path .
workspace-cli gmail send --to test@example.com --subject "Test" --body "Hello" --html
```

### 4. Verify Installation

```bash
# Check which executable is being used
Get-Command workspace-cli

# Expected output:
# Source: C:\Users\labhk\.cargo\bin\workspace-cli.exe

# Verify the new feature is available
workspace-cli gmail send --help
```

---

## Common Scenarios

### Scenario 1: Adding a New CLI Flag

**Example: Adding `--html` flag to gmail send**

1. **Update CLI definition** (`src/main.rs`):
   ```rust
   Send {
       #[arg(long)]
       html: bool,  // Add new flag
   }
   ```

2. **Update command handler** (`src/main.rs`):
   ```rust
   GmailCommands::Send { html, ... } => {
       // Use the new flag
       is_html: html,
   }
   ```

3. **Update business logic** (`src/commands/gmail/send.rs`):
   ```rust
   pub struct ComposeParams {
       pub is_html: bool,  // Add new field
   }
   
   fn build_raw_email(params: &ComposeParams) -> String {
       if params.is_html {
           email.push_str("Content-Type: text/html; charset=utf-8\r\n");
       }
   }
   ```

4. **Build and test**:
   ```bash
   cargo build
   cargo install --path .
   workspace-cli gmail send --help  # Verify --html appears
   ```

### Scenario 2: Fixing a Bug

1. Identify the bug in the code
2. Make the fix in appropriate file
3. Test with `cargo run --`:
   ```bash
   cargo run -- gmail list --limit 5
   ```
4. If working, install:
   ```bash
   cargo install --path .
   ```

### Scenario 3: Adding a New Command

1. Add command enum to `src/main.rs`
2. Implement command logic in `src/commands/`
3. Build and test
4. Install updated version

---

## Troubleshooting

### Problem: "unexpected argument '--html' found"

**Cause:** You're running an old version of workspace-cli

**Solution:**
```bash
cd D:\Modules\workspace-cli
cargo install --path .
```

**Verify:**
```bash
Get-Command workspace-cli  # Should show C:\Users\labhk\.cargo\bin\
workspace-cli --version
```

### Problem: Changes not appearing

**Solution:** Ensure you're testing the right version:
```bash
# Don't use: just "workspace-cli" (might be old version)
# Do use: cargo run -- or reinstall

# Reinstall to update
cargo install --path .

# Or test directly without installing
cargo run -- gmail send --help
```

### Problem: Compilation errors

**Solution:**
```bash
# Clean build artifacts
cargo clean

# Rebuild
cargo build

# Check for detailed errors
cargo build --verbose
```

---

## Build Types

### Debug Build (Fast, for development)
```bash
cargo build
# Output: target/debug/workspace-cli.exe
# Use with: cargo run --
```

### Release Build (Optimized, for production)
```bash
cargo build --release
# Output: target/release/workspace-cli.exe
# Smaller, faster, takes longer to compile
```

### Install (Builds release + copies to PATH)
```bash
cargo install --path .
# Compiles in release mode
# Copies to: C:\Users\labhk\.cargo\bin\
```

---

## Testing Checklist

After making changes, verify:

- [ ] Code compiles: `cargo build`
- [ ] No linting issues: `cargo clippy`
- [ ] Code formatted: `cargo fmt --check`
- [ ] Help text shows new features: `workspace-cli <command> --help`
- [ ] Feature works as expected (test with real data)
- [ ] No regression in existing features

---

## Version Management

### Check Installed Version
```bash
workspace-cli --version
```

### Update After Code Changes
```bash
# From workspace-cli directory
cargo install --path .

# Cargo will automatically:
# 1. Compile in release mode
# 2. Replace old version in .cargo/bin/
# 3. Show: "Replacing C:\Users\labhk\.cargo\bin\workspace-cli.exe"
```

### Multiple Versions
If you need to keep multiple versions:
```bash
# Rename current version
cd C:\Users\labhk\.cargo\bin\
copy workspace-cli.exe workspace-cli-backup.exe

# Install new version
cd D:\Modules\workspace-cli
cargo install --path .
```

---

## Example: The HTML Email Feature

### What Changed
```
Files Modified:
- src/main.rs (added --html flag to Send, Reply, Draft, ReplyDraft)
- src/commands/gmail/send.rs (added is_html field, updated Content-Type logic)

Files NOT Changed:
- All other files remain the same
```

### Update Process Used
```bash
1. Edit src/main.rs and src/commands/gmail/send.rs
2. cargo build (verify compilation)
3. cargo run -- gmail send --help (verify --html flag appears)
4. cargo run -- gmail send --to test@example.com --subject "Test" --body "<h1>Test</h1>" --html
5. cargo install --path . (install to system)
6. workspace-cli gmail send --help (verify installed version has --html)
7. workspace-cli gmail send --to user@example.com --subject "Test" --body "<h1>HTML</h1>" --html
```

---

## Quick Commands Reference

```bash
# Development
cargo build                   # Quick build (debug)
cargo build --release         # Optimized build
cargo run -- <args>           # Run without installing
cargo clippy                  # Lint code
cargo fmt                     # Format code
cargo clean                   # Clean build artifacts

# Installation
cargo install --path .        # Install to .cargo/bin/
Get-Command workspace-cli     # Check installed location (Windows)

# Testing
workspace-cli --help          # Show all commands
workspace-cli gmail send --help  # Show command-specific help
workspace-cli --version       # Show version
```

---

## Best Practices

1. **Always test with `cargo run --` first** before installing
2. **Use `cargo clippy`** to catch common issues
3. **Check help text** after adding new flags
4. **Test with real data** to verify functionality
5. **Keep this guide updated** when you discover new patterns

---

## Related Files

- `README.md` - User documentation
- `Cargo.toml` - Dependencies and project metadata
- `src/main.rs` - CLI entry point
- `src/commands/` - Command implementations

---

**Last Updated:** 2026-01-16 (HTML email feature added)
