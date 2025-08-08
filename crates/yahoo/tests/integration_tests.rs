use std::{error::Error, io::BufWriter};

use yahoo_fantasy_rs::{self, Token};

#[cfg(test)]
mod integration_tests {
    use dotenv;
    use std::env;
    use yahoo_fantasy_rs::{Game, GameCode};

    use crate::{load_token_file, save_token_file};

    // #[tokio::test]
    async fn test_something() {
        dotenv::from_filename("notes/.env").expect("should work");

        let client_id =
            env::var("YAHOO_CLIENT_ID").expect("failed to load client_id from env file");
        let redirect_uri =
            env::var("YAHOO_REDIRECT_URI").expect("failed to load redirect_uri from env file");
        let token_file =
            env::var("YAHOO_TOKEN_FILE").expect("failed to load token_file from env file");

        assert_eq!(
            "MH5MD53HFUWDT774Z3SKZV3CIJVFPRA5KO4ZKH6OE4A5DWD3TBXQLF5PYRFYT3SDEEWQ62GUNQZN43ZPJT6XKRJSWCZVUBLM",
            client_id
        );
        assert_eq!(
            "https://example.invalid/ccc3ad3b6d6cd350.png",
            redirect_uri
        );
        assert_eq!("../fantasy-cli/.token.json", token_file);

        let mut fixture = yahoo_fantasy_rs::Client::with_token::<String>(
            client_id,
            redirect_uri,
            load_token_file(&token_file).expect("should work"),
            None,
            None,
        );

        match fixture.refresh_token().await {
            Ok(t) => {
                save_token_file(&token_file, t).expect("asdfasdfasfdas");
            }
            Err(err) => panic!("kaboom: {}", err),
        };

        let dest_dir = "./tests/data";

        let games = fixture
            .get_games(vec![GameCode::NFL], vec!["2019"])
            .await
            .expect("things");

        assert_eq!(vec![] as Vec<Game>, games);
    }
}

fn load_token_file(token_file: &String) -> Result<Token, Box<dyn Error>> {
    match std::fs::File::open(token_file) {
        // Read the JSON contents of the file as an instance of `Token`.
        Ok(file) => match serde_json::from_reader(std::io::BufReader::new(file)) {
            Ok(token) => Ok(token),
            Err(err) => Err(Box::new(err)),
        },
        Err(err) => Err(Box::new(err)),
    }
}

fn save_token_file(file: &String, token: Token) -> Result<(), Box<dyn Error>> {
    match std::fs::OpenOptions::new()
        .write(true)
        .create(true) // creates if not exists or opens
        .truncate(true)
        .open(file)
    {
        // Read the JSON contents of the file as an instance of `Token`.
        Ok(file) => match serde_json::to_writer_pretty(BufWriter::new(file), &token) {
            Ok(_) => Ok(()),
            Err(err) => Err(Box::new(err)),
        },
        Err(err) => Err(Box::new(err)),
    }
}
