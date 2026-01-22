# CLAUDE.md - workspace-cli Development Guide

## Project Overview

**workspace-cli** is a high-performance Rust CLI for Google Workspace APIs, optimized for AI agent integration. It provides structured JSON output for Gmail, Drive, Calendar, Docs, Sheets, Slides, and Tasks.

**Author:** Majid Manzarpour
**License:** MIT
**Rust Edition:** 2021

## Quick Command Reference

### Build & Test
```bash
cargo build --release          # Release build (optimized, ~4MB binary)
cargo build                    # Debug build
cargo test                     # Run all tests (9 tests)
cargo clippy                   # Lint check
```

### Binary Location
```
./target/release/workspace-cli   # Release binary
./target/debug/workspace-cli     # Debug binary
```

### Authentication
```bash
workspace-cli auth login --credentials /path/to/credentials.json
workspace-cli auth status
workspace-cli auth logout
```

## Architecture

```
src/
├── main.rs              # CLI entry point, clap command definitions (~2000 lines)
├── lib.rs               # Library exports
├── cli.rs               # CLI context utilities
├── auth/                # OAuth2 & token management
│   ├── oauth.rs         # WorkspaceAuthenticator, SCOPES
│   ├── token.rs         # TokenManager (get_access_token, ensure_authenticated)
│   └── keyring_storage.rs  # OS keyring integration
├── client/              # API client infrastructure
│   ├── api_client.rs    # ApiClient with rate limiting & retry
│   ├── rate_limiter.rs  # Per-API rate limiters
│   ├── retry.rs         # Exponential backoff retry logic
│   └── batch.rs         # BatchClient for multipart/mixed batch requests
├── commands/            # Service implementations
│   ├── gmail/           # list, get, send (cc/bcc), reply, forward, delete, trash, labels, filters
│   ├── drive/           # list, upload, download, mkdir, share, watch, changes
│   ├── calendar/        # list, get, create (attendees, recurrence, reminders), update, delete, free-busy
│   ├── docs/            # get, append, create, replace, delete
│   ├── sheets/          # get, update, append, create, clear, list-sheets, delete
│   ├── slides/          # get, create, add-slide, add-text
│   ├── tasks/           # lists, list, create, update, delete
│   └── batch/           # CLI wrapper for batch API requests
├── config/              # Config file handling (~/.config/workspace-cli/)
├── error/               # Structured error types (CliError, WorkspaceError)
├── output/              # Formatter (JSON/JSONL/CSV), field filtering
└── utils/               # base64, field_mask, html_to_md
```

## Key Components

### ApiClient (`src/client/api_client.rs`)
Factory methods create service-specific clients with appropriate rate limiters:
```rust
ApiClient::gmail(token_manager)    // Gmail API client
ApiClient::drive(token_manager)    // Drive API client
ApiClient::calendar(token_manager) // Calendar API client
ApiClient::docs(token_manager)     // Docs API client
ApiClient::sheets(token_manager)   // Sheets API client
ApiClient::slides(token_manager)   // Slides API client
ApiClient::tasks(token_manager)    // Tasks API client
```

### API Endpoints (`src/client/api_client.rs:11-19`)
```rust
GMAIL:    "https://gmail.googleapis.com/gmail/v1"
DRIVE:    "https://www.googleapis.com/drive/v3"
CALENDAR: "https://www.googleapis.com/calendar/v3"
DOCS:     "https://docs.googleapis.com/v1"
SHEETS:   "https://sheets.googleapis.com/v4"
SLIDES:   "https://slides.googleapis.com/v1"
TASKS:    "https://tasks.googleapis.com/tasks/v1"
```

### Output Formatter (`src/output/formatter.rs`)
Handles JSON, JSON-compact, JSONL, and CSV output with field filtering:
- `--format json|json-compact|jsonl|csv`
- `--fields "id,name,mimeType"` - Filter output fields
- `--quiet` - Suppress output
- `--output file.json` - Write to file

**Important:** Field filtering handles wrapper objects (`files`, `messages`, `items`, `labels`, `permissions`) by filtering array items, not the wrapper itself.

### Error Handling (`src/error/types.rs`)
Structured errors for agent consumption:
```rust
ErrorCode::AuthenticationFailed
ErrorCode::TokenExpired
ErrorCode::RateLimitExceeded
ErrorCode::QuotaExceeded
ErrorCode::NotFound
ErrorCode::PermissionDenied
ErrorCode::InvalidRequest
ErrorCode::NetworkError
ErrorCode::ServerError
```

## CLI Command Structure

### Global Options (all commands)
```
-f, --format <FORMAT>    Output format: json, jsonl, csv [default: json]
--fields <FIELDS>        Comma-separated fields to include
-o, --output <FILE>      Write output to file
-q, --quiet              Suppress non-essential output
```

### Gmail Commands
```bash
gmail list [--query "is:unread"] [--limit 20] [--label INBOX]
gmail get <id> [--full]                    # Minimal by default (headers + plain text body)
gmail send --to <email> --subject <text> --body <text> [--cc <emails>] [--bcc <emails>] [--body-file <path>] [--html] [--attachment <path>...]
gmail draft --to <email> --subject <text> [--body <text>] [--cc <emails>] [--bcc <emails>] [--html] [--attachment <path>...]
gmail attachments <id>                     # List attachments in a message
gmail attachment <message-id> <attachment-id> --output <path>  # Download attachment
gmail reply <id> --body <text> [--body-file <path>] [--all] [--html]
gmail reply-draft <id> --body <text> [--all] [--html]
gmail delete <id>
gmail trash <id>
gmail untrash <id>
gmail labels
gmail modify <id> [--add-labels L1,L2] [--remove-labels L3] [--mark-read] [--mark-unread] [--star] [--unstar] [--archive]
gmail forward <id> --to <email> [--cc <emails>] [--bcc <emails>] [--message <text>]
gmail filters                                # List all email filters
gmail create-filter [--from X] [--to Y] [--subject Z] [--query Q] [--add-labels L] [--skip-inbox] [--mark-read] [--star]
gmail delete-filter <filter-id>
```

**CC/BCC:** Use comma-separated emails: `--cc "user1@example.com,user2@example.com"`

**Attachments:** Add files to emails with `--attachment`:
```bash
# Single attachment
gmail send --to X --subject "Report" --body "See attached" --attachment report.pdf

# Multiple attachments
gmail send --to X --subject "Files" --body "Attached" --attachment doc.pdf --attachment image.png

# Download attachment from email
gmail attachments <message-id>    # First, list to get attachment IDs
gmail attachment <message-id> <attachment-id> --output downloaded-file.pdf
```

**Token Optimization (defaults to minimal output):**
- `gmail get` returns essential headers (from, to, subject, date) + plain text body (~88% reduction). Use `--full` for raw message structure.
- `gmail send/reply/draft` return only `{success, id, threadId}` (~90% reduction)
- `gmail modify` returns only `{success, id, labels}` (~99% reduction)

**Forward emails:**
```bash
# Simple forward
gmail forward <message-id> --to recipient@example.com

# Forward with note
gmail forward <message-id> --to user@example.com --message "FYI - see below"

# Forward with CC
gmail forward <message-id> --to user@example.com --cc "team@example.com"
```

**Email filters:**
```bash
# List all filters
gmail filters

# Auto-archive emails from newsletters
gmail create-filter --from "newsletter@example.com" --skip-inbox

# Star and label emails from boss
gmail create-filter --from "boss@company.com" --star --add-labels "IMPORTANT"

# Mark emails with "urgent" in subject as read and star
gmail create-filter --subject "urgent" --mark-read --star

# Filter with Gmail search query
gmail create-filter --query "from:support@vendor.com has:attachment" --add-labels "CATEGORY_UPDATES"

# Delete a filter
gmail delete-filter <filter-id>
```

### Drive Commands
```bash
drive list [--query <q>] [--limit 20] [--parent <folder-id>]
drive get <id>
drive upload <file> [--parent <folder-id>] [--name <name>]
drive download <id> [--output <path>]
drive delete <id>
drive trash <id>
drive untrash <id>
drive mkdir <name> [--parent <folder-id>]
drive move <id> --to <folder-id>
drive copy <id> [--name <name>] [--parent <folder-id>]
drive rename <id> <new-name>
drive share <id> --email <email> --role reader|writer|commenter
drive share <id> --anyone --role reader
drive permissions <id>
drive unshare <id> <permission-id>
drive start-page-token                     # Get token for watching changes
drive watch --page-token <token> --webhook <url>  # Watch Drive changes
drive watch-file <id> --webhook <url>      # Watch specific file
drive stop-watch --channel-id <id> --resource-id <id>  # Stop watching
drive changes --page-token <token>         # List recent changes
```

**Watch for Drive changes:**
```bash
# Step 1: Get a start page token
drive start-page-token
# Returns: {"startPageToken": "12345"}

# Step 2: Set up a watch channel (requires HTTPS webhook)
drive watch --page-token "12345" --webhook "https://your-server.com/webhook"
# Returns channel info with id and resourceId (save these!)

# Step 3: Your webhook receives notifications when files change

# Step 4: List actual changes when notified
drive changes --page-token "12345"

# Stop watching when done
drive stop-watch --channel-id "uuid-from-watch" --resource-id "resource-id-from-watch"
```

**Watch specific file:**
```bash
drive watch-file <file-id> --webhook "https://your-server.com/file-webhook"
```

### Calendar Commands
```bash
calendar list [--calendar primary] [--time-min 2025-01-01T00:00:00Z] [--time-max ...] [--limit 20] [--sync-token <token>] [--full]
calendar get <id> [--calendar primary]     # Get specific event by ID
calendar create --summary <title> --start <datetime> --end <datetime> [--description <text>] [--attendees <emails>] [--calendar primary] [--recurrence <rule>] [--reminders <spec>]
calendar update <id> [--summary <title>] [--start <datetime>] [--end <datetime>] [--calendar primary]
calendar delete <id> [--calendar primary]
calendar free-busy --time-min <datetime> --time-max <datetime> [--calendars "primary,other@example.com"] [--timezone "America/New_York"]
```

**Attendees:** Use comma-separated emails: `--attendees "user1@example.com,user2@example.com"`

**Check availability (free/busy):**
```bash
# Check your own availability
calendar free-busy --time-min "2025-01-20T09:00:00Z" --time-max "2025-01-20T17:00:00Z"

# Check multiple calendars
calendar free-busy --time-min "2025-01-20T09:00:00Z" --time-max "2025-01-20T17:00:00Z" \
  --calendars "primary,colleague@company.com,room-a@company.com"

# With timezone
calendar free-busy --time-min "2025-01-20T09:00:00-05:00" --time-max "2025-01-20T17:00:00-05:00" \
  --timezone "America/New_York"
```
Returns busy time slots for each calendar, useful for finding meeting times.

**Recurring Events:** Use RRULE format (RFC 5545):
```bash
# Daily for 5 days
calendar create --summary "Standup" --start ... --end ... --recurrence "RRULE:FREQ=DAILY;COUNT=5"

# Weekly on Mon, Wed, Fri
calendar create --summary "Team Sync" --start ... --end ... --recurrence "RRULE:FREQ=WEEKLY;BYDAY=MO,WE,FR"

# Monthly on the 15th
calendar create --summary "Monthly Review" --start ... --end ... --recurrence "RRULE:FREQ=MONTHLY;BYMONTHDAY=15"
```

**Event Reminders:** Format is `method:minutes` (comma-separated for multiple):
```bash
# Email 30 minutes before
calendar create --summary "Meeting" --start ... --end ... --reminders "email:30"

# Email 30 min + popup 10 min before
calendar create --summary "Meeting" --start ... --end ... --reminders "email:30,popup:10"

# Popup 1 hour (60 min) before
calendar create --summary "Call" --start ... --end ... --reminders "popup:60"
```

**Token Optimization:** `calendar list` returns minimal event fields (id, summary, start, end, status) by default (~50% reduction). Use `--full` for attendees, organizer, description, recurrence, etc.

### Docs Commands
```bash
docs get <id> [--markdown] [--text]        # --text for plain text output
docs create <title>
docs append <id> <text>
docs replace <id> --find <text> --with <replacement> [--match-case]
docs delete <id>                           # Move document to trash
docs insert-image <id> --uri <url> [--index N] [--width P] [--height P]  # Insert image
docs insert-table <id> --rows N --columns M [--index N]                  # Insert table
docs export <id> -o <path> [-f <format>]   # Export to pdf, docx, txt, html, odt, rtf, epub
```

**Insert image:** URI must be publicly accessible. Omit `--index` to append at end.
```bash
docs insert-image <id> --uri "https://example.com/image.png"             # Append image
docs insert-image <id> --uri "https://example.com/image.png" --index 1   # Insert at start
docs insert-image <id> --uri "https://..." --width 200 --height 150      # With size (points)
```

**Insert table:** Creates an empty table with specified rows and columns.
```bash
docs insert-table <id> --rows 3 --columns 4           # Append 3x4 table
docs insert-table <id> --rows 2 --columns 3 --index 1 # Insert at start
```

**Export formats:** `pdf` (default), `docx`, `txt`, `html`, `odt`, `rtf`, `epub`
```bash
docs export <id> -o report.pdf              # Export as PDF (default)
docs export <id> -o report.docx -f docx     # Export as Word document
docs export <id> -o report.txt -f txt       # Export as plain text
```

**Token Optimization:** Use `--text` for plain text extraction (~70% reduction vs JSON structure). Use `--markdown` for formatted text.

### Sheets Commands
```bash
sheets get <id> --range "Sheet1!A1:C10" [--full]  # Values array by default
sheets create <title>
sheets update <id> --range "Sheet1!A1:B2" --values '[["Name","Value"],["A","1"]]'
sheets append <id> --range "Sheet1!A1" --values '[["Row1","Data"]]'
sheets clear <id> --range "Sheet1!A1:C10"
sheets list-sheets <id>                    # List all tabs in spreadsheet
sheets add-sheet <id> --title "New Tab" [--index N]         # Add new tab
sheets rename-sheet <id> --sheet-id N --title "New Name"    # Rename tab
sheets delete <id>                         # Move spreadsheet to trash
sheets export <id> -o <path> [-f <format>] [--sheet <name>]  # Export to csv, xlsx, pdf, ods, tsv, html
```

**Manage tabs/sheets:**
```bash
# List sheets to get sheet IDs
sheets list-sheets <spreadsheet-id>

# Add new sheet at end
sheets add-sheet <id> --title "Q2 Data"

# Add new sheet at beginning
sheets add-sheet <id> --title "Summary" --index 0

# Rename a sheet (use sheet ID from list-sheets)
sheets rename-sheet <id> --sheet-id 123456789 --title "Q1 Data (Archived)"
```

**Export formats:** `csv` (default), `xlsx`, `pdf`, `ods`, `tsv`, `html`
```bash
sheets export <id> -o data.csv                    # Export as CSV (default, first sheet)
sheets export <id> -o data.xlsx -f xlsx           # Export as Excel workbook
sheets export <id> -o data.pdf -f pdf             # Export as PDF
sheets export <id> -o sheet2.csv --sheet "Sheet2" # Export specific sheet as CSV
```

**Token Optimization:** `sheets get` returns just the values array by default (~50% reduction). Use `--full` for range metadata wrapper.

### Slides Commands
```bash
slides get <id> [--full]                   # Text extraction by default
slides page <id> --page 0 [--full]         # Text extraction by default
slides export <id> -o <path> [-f <format>] # Export to pdf, pptx, odp, txt
slides create --title <name>               # Create new presentation
slides add-slide <id> [--index N] [--layout BLANK]   # Add slide to presentation
slides add-text <id> --page-id <pid> --text "..." [--x 100] [--y 100] [--width 400] [--height 50]
```

**Create presentations:**
```bash
# Create a new blank presentation
slides create --title "Q1 Report"

# Create and get the presentation ID from the response
slides create --title "Team Deck" --fields "presentationId"
```

**Add slides with layouts:**
```bash
# Add blank slide at end
slides add-slide <presentation-id>

# Add slide at specific position (0 = first)
slides add-slide <id> --index 0

# Add slide with layout
slides add-slide <id> --layout TITLE_AND_BODY
slides add-slide <id> --layout TITLE_ONLY
slides add-slide <id> --layout SECTION_HEADER
```

**Available layouts:** `BLANK`, `TITLE`, `TITLE_AND_BODY`, `TITLE_AND_TWO_COLUMNS`, `TITLE_ONLY`, `SECTION_HEADER`, `SECTION_TITLE_AND_DESCRIPTION`, `ONE_COLUMN_TEXT`, `MAIN_POINT`, `BIG_NUMBER`, `CAPTION_ONLY`

**Add text to slides:**
```bash
# Add text box at position (x=100pt, y=100pt from top-left)
slides add-text <id> --page-id p1 --text "Hello World"

# Custom position and size (all in points)
slides add-text <id> --page-id p1 --text "Title" --x 50 --y 50 --width 600 --height 80

# Get page-id from slides get response (objectId field in slides array)
slides get <id> --fields "slides"
```

**Export formats:** `pdf` (default), `pptx`, `odp`, `txt`
```bash
slides export <id> -o presentation.pdf           # Export as PDF (default)
slides export <id> -o presentation.pptx -f pptx  # Export as PowerPoint
slides export <id> -o slides.txt -f txt          # Export text content only
```

**Token Optimization:** Returns extracted text content by default (~93% reduction). Use `--full` for complete presentation structure (masters, layouts, transforms, colors).

### Tasks Commands
```bash
tasks lists                           # List all task lists
tasks list [--list @default] [--limit 20] [--show-completed] [--full]
tasks create <title> [--list @default] [--due 2025-01-20T12:00:00Z] [--notes <text>]
tasks update <id> [--list @default] [--title <text>] [--complete]
tasks delete <id> [--list @default]
```

**Token Optimization:** `tasks list` returns minimal task fields (id, title, status, due, notes, completed) by default (~40% reduction). Use `--full` for etag, selfLink, links, parent, position, etc.

### Batch Commands
Execute up to 100 API requests in a single HTTP call:
```bash
batch gmail --requests '<json-array>'     # Batch Gmail API requests
batch gmail --file requests.json          # Read requests from file
batch drive --requests '<json-array>'     # Batch Drive API requests
batch calendar --requests '<json-array>'  # Batch Calendar API requests
echo '<json>' | batch gmail               # Read from stdin
```

**Request format:**
```json
[
  {"id": "req1", "method": "GET", "path": "/gmail/v1/users/me/messages/abc123"},
  {"id": "req2", "method": "POST", "path": "/gmail/v1/users/me/messages/xyz/modify", "body": {"addLabelIds": ["STARRED"]}}
]
```

**Response format:**
```json
{
  "status": "success|partial|error",
  "results": [{"id": "req1", "status": 200, "body": {...}}],
  "errors": [{"id": "req2", "status": 400, "message": "..."}]
}
```

**Path prefixes by service:**
- Gmail: `/gmail/v1/...`
- Drive: `/drive/v3/...`
- Calendar: `/calendar/v3/...`

## Interpreting User Requests

### Common Patterns

| User Says | Command |
|-----------|---------|
| "list my emails" / "show inbox" | `gmail list --limit 20` |
| "unread emails" | `gmail list --query "is:unread"` |
| "emails from X" | `gmail list --query "from:X"` |
| "read email <id>" | `gmail get <id>` |
| "send email to X" | `gmail send --to X --subject "..." --body "..."` |
| "send email with CC" | `gmail send --to X --cc "Y,Z" --subject "..." --body "..."` |
| "send email with BCC" | `gmail send --to X --bcc "Y" --subject "..." --body "..."` |
| "reply to email" | `gmail reply <id> --body "..."` |
| "reply all" | `gmail reply <id> --body "..." --all` |
| "draft a reply" | `gmail reply-draft <id> --body "..."` |
| "list files" / "my drive" | `drive list --limit 20` |
| "files in folder" | `drive list --parent <folder-id>` |
| "search for X" | `drive list --query "name contains 'X'"` |
| "upload file" | `drive upload <path>` |
| "download file" | `drive download <id> --output <path>` |
| "share with X" | `drive share <id> --email X --role writer` |
| "who has access" | `drive permissions <id>` |
| "my calendar" / "events" | `calendar list --time-min <today>` |
| "get event details" | `calendar get <event-id>` |
| "schedule meeting" | `calendar create --summary "..." --start ... --end ...` |
| "invite people to meeting" | `calendar create --summary "..." --start ... --end ... --attendees "a@x.com,b@x.com"` |
| "read document" | `docs get <id> --markdown` |
| "add to doc" | `docs append <id> "text"` |
| "spreadsheet data" | `sheets get <id> --range "Sheet1!A:Z"` |
| "list sheets/tabs" | `sheets list-sheets <id>` |
| "delete spreadsheet" | `sheets delete <id>` |
| "delete document" | `docs delete <id>` |
| "send email with attachment" | `gmail send --to X --subject "..." --body "..." --attachment file.pdf` |
| "download attachment" | `gmail attachments <id>` then `gmail attachment <id> <att-id> -o file.pdf` |
| "export doc as PDF" | `docs export <id> -o doc.pdf` |
| "export doc as Word" | `docs export <id> -o doc.docx -f docx` |
| "export sheet as CSV" | `sheets export <id> -o data.csv` |
| "export sheet as Excel" | `sheets export <id> -o data.xlsx -f xlsx` |
| "export slides as PDF" | `slides export <id> -o slides.pdf` |
| "export slides as PowerPoint" | `slides export <id> -o slides.pptx -f pptx` |
| "insert image in doc" | `docs insert-image <id> --uri "https://..."` |
| "add table to doc" | `docs insert-table <id> --rows 3 --columns 4` |
| "add new tab/sheet" | `sheets add-sheet <id> --title "New Sheet"` |
| "rename sheet/tab" | `sheets rename-sheet <id> --sheet-id N --title "New Name"` |
| "my tasks" / "todo list" | `tasks list` |
| "add task" | `tasks create "title"` |
| "complete task" | `tasks update <id> --complete` |
| "batch request" / "bulk operation" | `batch gmail/drive/calendar --requests '[...]'` |
| "get multiple emails at once" | `batch gmail --requests '[{"id":"1","method":"GET","path":"/gmail/v1/users/me/messages/id1"},...]'` |
| "star all these messages" | `batch gmail --requests '[{"id":"1","method":"POST","path":"/gmail/v1/users/me/messages/id1/modify","body":{"addLabelIds":["STARRED"]}}]'` |
| "create recurring event" | `calendar create --summary "..." --start ... --end ... --recurrence "RRULE:FREQ=WEEKLY"` |
| "weekly meeting" | `calendar create --summary "..." --start ... --end ... --recurrence "RRULE:FREQ=WEEKLY;BYDAY=MO"` |
| "set event reminder" | `calendar create --summary "..." --start ... --end ... --reminders "popup:10"` |
| "create presentation" | `slides create --title "..."` |
| "add slide" | `slides add-slide <id> --layout TITLE_AND_BODY` |
| "add text to slide" | `slides add-text <id> --page-id p1 --text "..."` |
| "forward this email" | `gmail forward <id> --to user@example.com` |
| "create email filter" | `gmail create-filter --from X --skip-inbox` |
| "list my filters" | `gmail filters` |
| "check availability" | `calendar free-busy --time-min ... --time-max ...` |
| "when is X free" | `calendar free-busy --time-min ... --time-max ... --calendars "X@example.com"` |
| "watch for file changes" | `drive start-page-token` then `drive watch --page-token T --webhook URL` |
| "list recent drive changes" | `drive changes --page-token T` |

### ID Extraction
Google Workspace IDs are found in URLs:
- Drive: `https://drive.google.com/file/d/<ID>/view`
- Docs: `https://docs.google.com/document/d/<ID>/edit`
- Sheets: `https://docs.google.com/spreadsheets/d/<ID>/edit`
- Slides: `https://docs.google.com/presentation/d/<ID>/edit`

### Date/Time Format
All datetime parameters use RFC3339 format:
```
2025-01-15T14:00:00Z      # UTC
2025-01-15T14:00:00-08:00 # With timezone
```

## Configuration

### Config File Location
```
~/.config/workspace-cli/config.toml
```

### Config Structure
```toml
[auth]
credentials_path = "/path/to/credentials.json"
service_account_path = "/path/to/service-account.json"

[output]
format = "json"
compact = false

[api]
timeout_seconds = 30
max_retries = 3
```

### Environment Variables
```bash
WORKSPACE_CREDENTIALS_PATH      # OAuth credentials JSON path
GOOGLE_APPLICATION_CREDENTIALS  # Service account JSON path
WORKSPACE_OUTPUT_FORMAT         # Default output format
WORKSPACE_OUTPUT_COMPACT        # true/false
WORKSPACE_API_TIMEOUT           # Timeout in seconds
WORKSPACE_API_MAX_RETRIES       # Max retry attempts
RUST_LOG                        # Logging level (debug, info, warn, error)
```

## Known Patterns & Gotchas

### List Response Wrappers
API list responses wrap items in arrays:
- Drive: `{ "files": [...], "nextPageToken": "..." }`
- Gmail: `{ "messages": [...], "resultSizeEstimate": N }`
- Tasks: `{ "items": [...] }`
- Calendar: `{ "items": [...] }`

The `--fields` flag filters within these arrays, not at root level.

### Gmail Query Syntax
Uses Gmail's search syntax:
```
is:unread
from:user@example.com
subject:keyword
has:attachment
after:2025/01/01
before:2025/12/31
label:INBOX
```

### Drive Query Syntax
Uses Drive's query syntax:
```
name contains 'report'
mimeType = 'application/vnd.google-apps.folder'
'folder-id' in parents
trashed = false
modifiedTime > '2025-01-01T00:00:00'
```

### Tasks API Limits
- `maxResults`: 1-100 (default 20)
- List ID `@default` refers to primary task list

### Email Subject Encoding
Non-ASCII subjects (emojis, special characters) are RFC 2047 Base64 encoded automatically.

## Development Notes

### Adding New Commands
1. Add subcommand enum variant in `src/main.rs`
2. Create handler in appropriate `src/commands/<service>/` module
3. Wire up in `run()` function match statement
4. Add types in `types.rs` if needed

### Testing Against Live API
```bash
# Enable debug logging
RUST_LOG=debug ./target/release/workspace-cli gmail list --limit 1

# Test read-only operations
./target/release/workspace-cli drive list --limit 3
./target/release/workspace-cli gmail labels
./target/release/workspace-cli tasks lists
./target/release/workspace-cli calendar list --time-min "2025-01-01T00:00:00Z"
```

### Common Build Issues
- Keyring issues on Linux: May need `gnome-keyring` or `libsecret`
- SSL issues: Ensure `openssl-dev` is installed

## Dependencies (Key)

| Crate | Purpose |
|-------|---------|
| tokio | Async runtime |
| clap | CLI parsing |
| reqwest | HTTP client |
| serde/serde_json | JSON serialization |
| yup-oauth2 | Google OAuth2 |
| keyring | OS credential storage |
| base64 | Encoding |
| chrono | Date/time handling |
| thiserror | Error types |
