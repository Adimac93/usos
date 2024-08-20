use std::{cell::LazyCell, ops::Deref};

pub struct UsosUri;

impl UsosUri {
    pub const DOMAIN: &'static str = "apps.usos.pwr.edu.pl";

    pub fn origin() -> String {
        format!("https://{}/", Self::DOMAIN)
    }

    pub fn with_path(path: impl AsRef<str>) -> String {
        format!("{}{}", Self::origin(), path.as_ref())
    }
}

pub struct Client {
    base_url: &'static str,
    client: reqwest::Client,
}

impl Client {
    fn new(base_url: &'static str) -> Self {
        let client = reqwest::Client::builder()
            .user_agent(format!(
                "{}/{}",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION")
            ))
            .cookie_store(true)
            .build()
            .unwrap();

        Self { base_url, client }
    }
}

impl Deref for Client {
    type Target = reqwest::Client;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

pub const CLIENT: LazyCell<Client> = LazyCell::new(|| Client::new("https://apps.usos.pwr.edu.pl/"));
