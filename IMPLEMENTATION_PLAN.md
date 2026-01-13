# youtube-summary Implementation Plan

This document captures the planned implementation work discussed on 2026-01-12.

---

## Immediate Priority: Credential File Creation

### Current State
- Location defined: `~/.config/youtube-summary/credentials`
- Reading works: `Credentials::load()` in `src/config.rs:119-152`
- **Missing**: Cannot create/write the credentials file
- **Bug**: Error message at `src/config.rs:32-33` says `credentials.toml` but actual file is `credentials`

### What Needs to Be Done

#### 1. Fix the error message typo
**File**: `src/config.rs:32-33`

Change:
```
"No API key found. Set ANTHROPIC_API_KEY env var, use --api-key, or add to ~/.config/youtube-summary/credentials.toml"
```
To:
```
"No API key found. Set ANTHROPIC_API_KEY env var or add to ~/.config/youtube-summary/credentials"
```
(Also removing `--api-key` mention since we're removing that option)

#### 2. Add `save_api_key()` function
**File**: `src/config.rs`

Add this new function to the `Credentials` impl block:

```rust
impl Credentials {
    // Existing load() function stays...

    /// Saves an API key to the credentials file
    /// Creates the config directory if it doesn't exist
    /// Sets secure file permissions (0600 - owner read/write only)
    pub fn save_api_key(api_key: &str) -> Result<()> {
        let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let config_dir = PathBuf::from(home).join(".config/youtube-summary");

        // Create directory if it doesn't exist
        fs::create_dir_all(&config_dir)
            .map_err(|e| Error::Config(format!("Failed to create config directory: {}", e)))?;

        let credentials_path = config_dir.join("credentials");
        let content = format!("ANTHROPIC_API_KEY={}\n", api_key);

        fs::write(&credentials_path, content)
            .map_err(|e| Error::Config(format!("Failed to write credentials file: {}", e)))?;

        // Set file permissions to 0600 (owner read/write only) on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            fs::set_permissions(&credentials_path, perms)
                .map_err(|e| Error::Config(format!("Failed to set file permissions: {}", e)))?;
        }

        Ok(())
    }
}
```

#### 3. Make `Credentials` struct public
**File**: `src/config.rs:114`

Change:
```rust
struct Credentials {
```
To:
```rust
pub struct Credentials {
```

This allows `main.rs` to use `Credentials::save_api_key()` in the future for the API key UX upgrade.

---

## Next Priority: Remove CLI `--api-key` Option

### Why
- Security risk: API key visible in process list (`ps aux`)
- Not needed: ENV variable and credentials file are sufficient and more secure

### What to Keep
- `ANTHROPIC_API_KEY` environment variable support (secure, convenient)
- `~/.config/youtube-summary/credentials` file support (secure, persistent)

### What Needs to Be Done

#### 1. Remove from Args struct
**File**: `src/cli.rs:35`

Remove this line:
```rust
pub api_key: Option<String>,
```

#### 2. Remove argument parsing
**File**: `src/cli.rs:81-87`

Remove this entire match arm:
```rust
"-k" | "--api-key" => {
    i += 1;
    if i >= args.len() {
        return Err("--api-key requires a value".to_string());
    }
    api_key = Some(args[i].clone());
}
```

Also remove the `let mut api_key = None;` declaration at line 56.

#### 3. Remove from Args construction
**File**: `src/cli.rs:114`

Remove `api_key,` from the `Ok(Args { ... })` return statement.

#### 4. Update help text
**File**: `src/cli.rs:129`

Remove this line from the help string:
```
  -k, --api-key <KEY>      Anthropic API key (overrides env/config)
```

#### 5. Update Config::load() precedence
**File**: `src/config.rs:24-25`

Change from:
```rust
let api_key = args
    .api_key
    .clone()
    .or_else(|| env::var("ANTHROPIC_API_KEY").ok())
    .or(credentials.anthropic_api_key)
    .or(file_config.api_key)
```

To:
```rust
let api_key = env::var("ANTHROPIC_API_KEY")
    .ok()
    .or(credentials.anthropic_api_key)
    .or(file_config.api_key)
```

---

## Files Summary

| File | Changes |
|------|---------|
| `src/config.rs` | Add `save_api_key()`, fix error message, make `Credentials` public, update precedence |
| `src/cli.rs` | Remove `--api-key` argument (4 locations) |
| `src/main.rs` | No changes needed |

---

## Future Features (For Reference)

These were discussed but are NOT part of the immediate implementation:

### API Key UX Upgrade (Can Wait)
- Prompt user for API key if not configured
- Ask if they want to persist it to credentials file
- Use `Credentials::save_api_key()` if yes

### Security Libraries (Can Wait)
- `rpassword` / `termrw` - Hidden terminal input when prompting for API key
- `zeroize` - Secure memory handling (clear sensitive data from memory)
- `dirs` - Better home directory detection (cross-platform)

### Loading Spinners with indicatif (Soon)
Stages to display:
1. Validating inputs
2. Fetching transcript
3. Sending transcript to analysis
4. Receiving summary
5. Making sure everything is alright

Library: `indicatif` (user's choice)

### Config File Versioning (First Release)
- Add `version=1` to config/credentials files
- Check version on load for compatibility

### Prompt Templates (Low Hanging Fruit)
- User-customizable prompt templates
- Template storage and selection
- History of recent prompts

### Local LLM Support (Big Feature - Last)
- Ollama integration
- Provider selection mechanism

### Cache User Preferences (End of Project)
- Remember last template choice
- Remember preferred model
- Other persistent preferences

---

## Testing After Implementation

1. **Test credential file creation**:
   ```bash
   # Remove existing credentials if any
   rm -f ~/.config/youtube-summary/credentials

   # The save function should:
   # - Create ~/.config/youtube-summary/ directory
   # - Create credentials file with ANTHROPIC_API_KEY=<key>
   # - Set permissions to 600
   ```

2. **Test --api-key removal**:
   ```bash
   # This should now fail with "Unknown argument"
   youtube-summary "https://youtube.com/watch?v=xxx" -k "sk-ant-xxx"

   # This should still work
   ANTHROPIC_API_KEY="sk-ant-xxx" youtube-summary "https://youtube.com/watch?v=xxx"
   ```

3. **Verify help text**:
   ```bash
   youtube-summary --help
   # Should NOT show -k, --api-key option
   ```

---

## Notes

- Current API key precedence: CLI > ENV > credentials > config
- New API key precedence: ENV > credentials > config
- Credentials file format: `ANTHROPIC_API_KEY=<key>` (simple key=value, like .env)
- Config file format: `api_key=<key>` and `default_model=<haiku|sonnet|opus>`
- Both files support `#` comments and empty lines
