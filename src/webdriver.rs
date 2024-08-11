pub mod geckodriver;
use std::time::Duration;

use geckodriver::{GeckoDriver, WebDriverClientBuilder};
use tokio::time::sleep;

pub struct WebDriverClient {
    client: fantoccini::Client,
    driver: tokio::process::Child,
}

impl WebDriverClient {
    pub async fn new(driver: GeckoDriver) -> Self {
        let (driver, port) = driver.spawn().await.unwrap();

        let client = fantoccini::ClientBuilder::native()
            .connect(&format!("http://127.0.0.1:{port}"))
            .await
            .unwrap();

        Self { client, driver }
    }

    pub fn builder() -> WebDriverClientBuilder {
        WebDriverClientBuilder::new()
    }

    pub fn client(&self) -> &fantoccini::Client {
        &self.client
    }

    pub async fn close(mut self) {
        self.client.close().await.unwrap();
        self.driver.kill().await.unwrap();
    }
}

#[tokio::test]
#[ignore = "requires geckodriver to be installed"]
async fn test_webdriver() {
    let web_driver = WebDriverClient::builder()
        .port(4444)
        .log_level(geckodriver::LogLevel::Debug)
        .build()
        .await;
}
