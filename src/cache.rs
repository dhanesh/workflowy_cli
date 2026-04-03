// Satisfies: RT-8 (target caching), U1 (transparent + bypassable), S2 (600 perms)

use crate::error::CliError;
use crate::models::TargetOutput;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// Cache TTL: 1 hour
const CACHE_TTL: Duration = Duration::from_secs(3600);

/// Returns cache directory: ~/.cache/workflowy-cli/
fn cache_dir() -> Result<PathBuf, CliError> {
    let home = std::env::var("HOME")
        .map_err(|_| CliError::Api("HOME environment variable not set".into()))?;
    Ok(PathBuf::from(home).join(".cache").join("workflowy-cli"))
}

fn targets_cache_path() -> Result<PathBuf, CliError> {
    Ok(cache_dir()?.join("targets.json"))
}

/// Read cached targets if the cache exists and is within TTL.
pub fn read_targets_cache() -> Result<Option<Vec<TargetOutput>>, CliError> {
    let path = targets_cache_path()?;
    if !path.exists() {
        return Ok(None);
    }

    // Check TTL based on file modification time
    let metadata = fs::metadata(&path)
        .map_err(|e| CliError::Api(format!("Failed to read cache metadata: {}", e)))?;
    let modified = metadata
        .modified()
        .map_err(|e| CliError::Api(format!("Failed to read cache mtime: {}", e)))?;

    if SystemTime::now()
        .duration_since(modified)
        .unwrap_or(CACHE_TTL)
        >= CACHE_TTL
    {
        tracing::debug!("targets cache expired");
        return Ok(None);
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| CliError::Api(format!("Failed to read cache: {}", e)))?;
    let targets: Vec<TargetOutput> = serde_json::from_str(&content).map_err(|e| {
        tracing::warn!("corrupt cache file, ignoring: {}", e);
        CliError::Api(format!("Corrupt cache: {}", e))
    })?;

    Ok(Some(targets))
}

/// Write targets to cache with 600 permissions (S2).
pub fn write_targets_cache(targets: &[TargetOutput]) -> Result<(), CliError> {
    let dir = cache_dir()?;
    fs::create_dir_all(&dir)
        .map_err(|e| CliError::Api(format!("Failed to create cache directory: {}", e)))?;

    let path = dir.join("targets.json");
    let content = serde_json::to_string(targets)?;
    fs::write(&path, &content)
        .map_err(|e| CliError::Api(format!("Failed to write cache: {}", e)))?;

    // Satisfies: S2 — cache files use 600 permissions
    #[cfg(unix)]
    {
        let perms = fs::Permissions::from_mode(0o600);
        fs::set_permissions(&path, perms)
            .map_err(|e| CliError::Api(format!("Failed to set cache permissions: {}", e)))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;

    // Validates: RT-8.3 — cache TTL is 1 hour
    #[test]
    fn cache_ttl_is_one_hour() {
        assert_eq!(CACHE_TTL, Duration::from_secs(3600));
    }

    // Validates: RT-8.1 — cache path is in standard location
    #[test]
    fn cache_path_is_in_standard_location() {
        let path = targets_cache_path().unwrap();
        let path_str = path.to_string_lossy();
        assert!(path_str.contains(".cache/workflowy-cli/targets.json"));
    }

    // Validates: RT-8 — returns None when no cache exists
    #[test]
    #[serial]
    fn read_cache_returns_none_when_missing() {
        let orig_home = env::var("HOME").unwrap();
        // Use a path that definitely doesn't have a .cache/workflowy-cli/ dir
        let fake_home = env::temp_dir().join(format!("wfcli-nocache-{}", std::process::id()));
        env::set_var("HOME", &fake_home);
        let result = read_targets_cache().unwrap();
        env::set_var("HOME", orig_home);
        let _ = fs::remove_dir_all(&fake_home);
        assert!(result.is_none());
    }

    // Validates: RT-8, S2 — write and read round-trip works
    #[test]
    #[serial]
    fn write_then_read_cache_round_trip() {
        let tmp = env::temp_dir().join("workflowy-cli-cache-test-rt");
        let orig_home = env::var("HOME").unwrap();
        env::set_var("HOME", &tmp);

        let targets = vec![TargetOutput {
            key: "home".into(),
            target_type: "default".into(),
            name: Some("Home".into()),
        }];

        write_targets_cache(&targets).unwrap();
        let cached = read_targets_cache()
            .unwrap()
            .expect("should have cached data");
        assert_eq!(cached.len(), 1);
        assert_eq!(cached[0].key, "home");

        // Cleanup
        let _ = fs::remove_dir_all(tmp.join(".cache").join("workflowy-cli"));
        env::set_var("HOME", orig_home);
    }

    // Validates: S2 — cache file has 600 permissions
    #[cfg(unix)]
    #[test]
    #[serial]
    fn cache_file_has_600_permissions() {
        let tmp = env::temp_dir().join("workflowy-cli-cache-test-perms");
        let orig_home = env::var("HOME").unwrap();
        env::set_var("HOME", &tmp);

        let targets = vec![TargetOutput {
            key: "test".into(),
            target_type: "default".into(),
            name: None,
        }];

        write_targets_cache(&targets).unwrap();

        let cache_path = tmp
            .join(".cache")
            .join("workflowy-cli")
            .join("targets.json");
        let perms = fs::metadata(&cache_path).unwrap().permissions();
        assert_eq!(perms.mode() & 0o777, 0o600);

        // Cleanup
        let _ = fs::remove_dir_all(tmp.join(".cache").join("workflowy-cli"));
        env::set_var("HOME", orig_home);
    }
}
