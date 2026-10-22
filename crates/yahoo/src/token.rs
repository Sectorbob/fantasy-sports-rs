use std::io::{self, ErrorKind};

use tokio::{fs, io::AsyncWriteExt};

use crate::Token;

pub async fn load_token_file(token_file: &String) -> Result<Token, Error> {
    let file_contents = match fs::read(token_file).await {
        Ok(contents) => contents,
        Err(err) => {
            if matches!(err.kind(), ErrorKind::NotFound) {
                return Ok(Token::default());
            }
            return Err(Error::FileError(err));
        }
    };
    let mut token: Token = serde_json::from_slice(&file_contents)?;
    if token.ts.is_some() {
        let metadata = tokio::fs::metadata(token_file).await?;
        let modified = metadata.modified()?;
        token.ts = Some(modified.into())
    }
    Ok(token)
}

/// Writes the token as JSON. The file holds the access and refresh tokens, so on unix it is
/// created readable by its owner only (0600), including when it already existed with wider
/// permissions.
pub async fn save_token_file(file: &String, token: &Token) -> Result<(), Error> {
    let mut options = fs::OpenOptions::new();
    options
        .write(true)
        .create(true) // creates if not exists or opens
        .truncate(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut handle = options.open(file).await?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        handle
            .set_permissions(std::fs::Permissions::from_mode(0o600))
            .await?;
    }
    let token_json = serde_json::to_vec_pretty(&token)?;
    handle.write_all(&token_json).await?;
    handle.flush().await?;
    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to se/de json token: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("failed to perform file op: {0}")]
    FileError(#[from] io::Error),
    #[error("token filename is required")]
    FilenameRequired,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn token() -> Token {
        Token {
            access_token: "access".into(),
            token_type: "bearer".into(),
            expires_in: 3600,
            refresh_token: "refresh".into(),
            scope: Some("fspt-r".into()),
            ts: Some(chrono::Utc::now()),
        }
    }

    fn file(dir: &TempDir) -> String {
        dir.path().join("token.json").to_string_lossy().to_string()
    }

    #[tokio::test]
    async fn a_saved_token_loads_back() {
        let dir = TempDir::new().unwrap();
        let file = file(&dir);
        save_token_file(&file, &token()).await.unwrap();

        let loaded = load_token_file(&file).await.unwrap();

        assert_eq!("access", loaded.access_token);
        assert_eq!("refresh", loaded.refresh_token);
        assert_eq!(Some("fspt-r".to_string()), loaded.scope);
        assert!(loaded.ts.is_some());
    }

    #[tokio::test]
    async fn a_missing_token_file_is_an_empty_token_so_the_first_run_can_authorize() {
        let dir = TempDir::new().unwrap();
        let token = load_token_file(&file(&dir)).await.unwrap();
        assert!(token.access_token.is_empty());
        assert!(token.is_expired());
    }

    #[tokio::test]
    async fn a_corrupt_token_file_is_a_json_error() {
        let dir = TempDir::new().unwrap();
        let file = file(&dir);
        std::fs::write(&file, "{ not json").unwrap();
        assert!(matches!(
            load_token_file(&file).await,
            Err(Error::JsonError(_))
        ));
    }

    #[tokio::test]
    async fn a_token_in_an_unwritable_place_is_a_file_error() {
        let dir = TempDir::new().unwrap();
        let file = dir
            .path()
            .join("missing-dir")
            .join("token.json")
            .to_string_lossy()
            .to_string();
        assert!(matches!(
            save_token_file(&file, &token()).await,
            Err(Error::FileError(_))
        ));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn the_token_file_is_private_to_its_owner() {
        use std::os::unix::fs::PermissionsExt;
        let dir = TempDir::new().unwrap();
        let file = file(&dir);

        save_token_file(&file, &token()).await.unwrap();
        let mode = std::fs::metadata(&file).unwrap().permissions().mode() & 0o777;
        assert_eq!(0o600, mode, "freshly created: {mode:o}");

        // a file that already existed with wider permissions is tightened too
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o644)).unwrap();
        save_token_file(&file, &token()).await.unwrap();
        let mode = std::fs::metadata(&file).unwrap().permissions().mode() & 0o777;
        assert_eq!(0o600, mode, "pre-existing: {mode:o}");
    }

    #[tokio::test]
    async fn a_client_needs_a_token_file_name_from_somewhere() {
        if std::env::var("YAHOO_TOKEN_FILE").is_ok() {
            return; // the environment supplies one, which is the other branch
        }
        let err = crate::Client::new("id", "uri")
            .with_token_file::<String>(None)
            .await
            .map(|_| ())
            .unwrap_err();
        assert!(matches!(err, Error::FilenameRequired));
    }

    #[tokio::test]
    async fn a_client_loads_its_token_from_the_named_file() {
        let dir = TempDir::new().unwrap();
        let file = file(&dir);
        save_token_file(&file, &token()).await.unwrap();

        let client = crate::Client::new("id", "uri")
            .with_token_file(Some(file.clone()))
            .await
            .unwrap();

        assert_eq!(Some(file), client.token_file);
        assert_eq!(
            "access",
            client.refresh_token(false).await.unwrap().access_token
        );
    }
}
