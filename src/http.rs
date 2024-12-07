use reqwest::{Client, Error};

pub fn client() -> Result<Client, Error> {
    Client::builder()
        .user_agent(format!("umi-cli/{}", env!("CARGO_PKG_VERSION")))
        .build()
}
