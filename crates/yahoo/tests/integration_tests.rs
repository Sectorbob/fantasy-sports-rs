#[allow(dead_code)]
#[cfg(test)]
mod integration_tests {
    use dotenv;
    use yahoo_fantasy_rs::{Client, Game, GameCode};

    // #[tokio::test]
    async fn test_something() {
        dotenv::from_filename("notes/.env").expect("should work");

        let fixture = Client::new_from_env()
            .expect("unnable to load from env")
            .with_token_file::<String>(None)
            .await
            .expect("failed to load token file");
        assert_eq!(
            "MH5MD53HFUWDT774Z3SKZV3CIJVFPRA5KO4ZKH6OE4A5DWD3TBXQLF5PYRFYT3SDEEWQ62GUNQZN43ZPJT6XKRJSWCZVUBLM",
            fixture.client_id
        );
        assert_eq!(
            "https://example.invalid/ccc3ad3b6d6cd350.png",
            fixture.redirect_uri
        );
        assert_eq!(
            Some("../fantasy-cli/.token.json".to_string()),
            fixture.token_file
        );

        let _refreshed_token = fixture
            .refresh_token()
            .await
            .expect("failed to update token");
        // debug_assert_eq!("foo", refreshed_token.access_token, "token is unknoiwn");

        let _dest_dir = "./tests/data";

        let games = fixture
            .get_games(vec![GameCode::NFL], vec!["2019"])
            .await
            .expect("things");

        assert_eq!(vec![] as Vec<Game>, games);
    }
}
