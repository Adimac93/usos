use std::{
    fmt::Display,
    future::Future,
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};

use tokio::{
    io::{stdin, stdout, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt},
    time::sleep,
};

use super::WebDriverClient;

const DEFAULT_PORT: u16 = 4444;

pub struct GeckoDriver {
    cmd: tokio::process::Command,
    port: u16,
}

impl GeckoDriver {
    pub fn new() -> Self {
        let driver = tokio::process::Command::new("geckodriver");

        Self {
            cmd: driver,
            port: DEFAULT_PORT,
        }
    }

    fn port(&mut self, port: u16) {
        self.cmd.arg("--port").arg(port.to_string());
        self.port = port;
    }

    fn log_level(&mut self, level: LogLevel) {
        self.cmd.arg("--log").arg(level.to_string());
    }

    fn firefox_binary(&mut self, path: &Path) {
        self.cmd.arg("--binary").arg(path);
    }

    fn websocket_port(&mut self, port: u16) {
        self.cmd.arg("--websocket-port").arg(port.to_string());
    }

    pub async fn spawn(mut self) -> tokio::io::Result<(tokio::process::Child, u16)> {
        let mut child = self.cmd.kill_on_drop(true).spawn().unwrap();
        // FIXME
        sleep(Duration::from_secs(1)).await;
        // let stdout = child.stdout.take().unwrap();
        // let mut reader = tokio::io::BufReader::new(stdout)
        // let line = reader.lines().next_line().await.unwrap();
        // reader.
        // println!("{line:?}");
        // if reader.next_line().await.unwrap().is_some() {
        Ok((child, self.port))
        // } else {
        //     Err(tokio::io::Error::new(
        //         tokio::io::ErrorKind::Other,
        //         "Failed to read geckodriver output",
        //     ))
        // }
    }
}

#[derive(Debug)]
pub enum LogLevel {
    Fatal,
    Error,
    Warn,
    Info,
    Config,
    Debug,
    Trace,
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fatal => write!(f, "fatal"),
            Self::Error => write!(f, "error"),
            Self::Warn => write!(f, "warn"),
            Self::Info => write!(f, "info"),
            Self::Config => write!(f, "config"),
            Self::Debug => write!(f, "debug"),
            Self::Trace => write!(f, "trace"),
        }
    }
}

pub struct WebDriverClientBuilder {
    port: Option<u16>,
    log_level: Option<LogLevel>,
    firefox_binary: Option<PathBuf>,
}

impl WebDriverClientBuilder {
    pub fn new() -> Self {
        Self {
            port: None,
            log_level: None,
            firefox_binary: None,
        }
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn log_level(mut self, level: LogLevel) -> Self {
        self.log_level = Some(level);
        self
    }

    pub fn firefox_binary(mut self, path: PathBuf) -> Self {
        self.firefox_binary = Some(path);
        self
    }

    pub fn build(self) -> impl Future<Output = WebDriverClient> {
        let mut driver = GeckoDriver::new();
        if let Some(port) = self.port {
            driver.port(port);
        }
        if let Some(level) = self.log_level {
            driver.log_level(level);
        }
        if let Some(path) = self.firefox_binary {
            driver.firefox_binary(&path);
        }
        WebDriverClient::new(driver)
    }
}
