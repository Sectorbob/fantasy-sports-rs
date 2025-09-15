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

pub async fn save_token_file(file: &String, token: &Token) -> Result<(), Error> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true) // creates if not exists or opens
        .truncate(true)
        .open(file)
        .await?;
    let token_json = serde_json::to_vec_pretty(&token)?;
    file.write_all(&token_json).await?;
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
