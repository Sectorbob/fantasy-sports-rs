mod client;
mod data;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let yahoo_client_id = std::env::var("YAHOO_CLIENT_ID").unwrap();
    let yahoo_redirect_uri = std::env::var("YAHOO_REDIRECT_URI").unwrap();
    let mut client = client::Client::new(yahoo_client_id, yahoo_redirect_uri);

    let auth_request_url = client.get_request_auth_url().await.unwrap();
    println!("Auth request URL: {}", auth_request_url);

    let auth_code = receive_auth_code_from_stdin();
    println!("Auth code: {}", auth_code);
    let token = client.get_token(&auth_code).await.unwrap();
    println!("Token: {:#?}", token);

    let users = client.get_users(true).await.unwrap();
    println!("Users: {:#?}", users);

    Ok(())
}

fn receive_auth_code_from_stdin() -> String {
    println!("Please enter the auth code:");
    let mut auth_code = String::new();
    std::io::stdin().read_line(&mut auth_code).unwrap();
    auth_code.trim().to_string()
}
