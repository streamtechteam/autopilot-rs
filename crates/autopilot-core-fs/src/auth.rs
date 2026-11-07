use std::{
    env, fs,
    io::Write,
    path::{Path, PathBuf},
};

use crate::{
    config,
    error::{FsError, Result},
};

/// Environment variable that overrides the persisted API token.
pub const API_TOKEN_ENV: &str = "AUTOPILOT_API_TOKEN";
/// File name under the AutoPilot config root that stores the generated token.
pub const API_TOKEN_FILE: &str = "api-token";

/// Path to the persisted API token for the active config root.
pub fn api_token_path() -> Result<PathBuf> {
    Ok(config::current()?.api_token_path())
}

fn env_token() -> Option<String> {
    env::var(API_TOKEN_ENV)
        .ok()
        .map(|token| token.trim().to_string())
        .filter(|token| !token.is_empty())
}

fn read_file_token(path: &Path) -> Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(contents) => {
            let token = contents.trim();
            if token.is_empty() {
                return Err(FsError::Init(format!(
                    "API token file {} is empty",
                    path.display()
                )));
            }
            Ok(Some(token.to_string()))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(FsError::Init(format!(
            "Failed to read API token {}: {error}",
            path.display()
        ))),
    }
}

/// Read the configured API token without creating one.
///
/// `AUTOPILOT_API_TOKEN` wins. Otherwise the token file under the active
/// config root is used when it exists.
pub fn read_api_token() -> Result<Option<String>> {
    if let Some(token) = env_token() {
        return Ok(Some(token));
    }

    read_file_token(&api_token_path()?)
}

/// Read the API token, generating and persisting one when it does not exist.
pub fn load_or_create_api_token() -> Result<String> {
    if let Some(token) = env_token() {
        return Ok(token);
    }

    let path = api_token_path()?;
    if let Some(token) = read_file_token(&path)? {
        return Ok(token);
    }

    let token = generate_token()?;
    write_token_file(&path, &token)?;

    read_file_token(&path)?.ok_or_else(|| {
        FsError::Init(format!(
            "API token file {} disappeared after creation",
            path.display()
        ))
    })
}

fn generate_token() -> Result<String> {
    let mut bytes = [0_u8; 32];
    getrandom::fill(&mut bytes)
        .map_err(|error| FsError::Init(format!("Failed to generate API token: {error}")))?;

    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

fn write_token_file(path: &Path, token: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            FsError::Init(format!(
                "Failed to create API token directory {}: {error}",
                parent.display()
            ))
        })?;
    }

    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }

    match options.open(path) {
        Ok(mut file) => {
            file.write_all(token.as_bytes()).map_err(|error| {
                FsError::Init(format!(
                    "Failed to write API token {}: {error}",
                    path.display()
                ))
            })?;
            file.write_all(b"\n").map_err(|error| {
                FsError::Init(format!(
                    "Failed to write API token {}: {error}",
                    path.display()
                ))
            })?;
            file.sync_all().map_err(|error| {
                FsError::Init(format!(
                    "Failed to sync API token {}: {error}",
                    path.display()
                ))
            })?;
            Ok(())
        }
        // Another process won the race; the caller re-reads the file.
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
        Err(error) => Err(FsError::Init(format!(
            "Failed to create API token {}: {error}",
            path.display()
        ))),
    }
}

/// Compare secrets without short-circuiting on the first differing byte.
pub fn constant_time_eq(left: &str, right: &str) -> bool {
    let left = left.as_bytes();
    let right = right.as_bytes();
    if left.len() != right.len() {
        return false;
    }

    left.iter()
        .zip(right)
        .fold(0_u8, |difference, (left, right)| {
            difference | (left ^ right)
        })
        == 0
}

#[cfg(test)]
#[path = "auth.spec.rs"]
mod spec;
