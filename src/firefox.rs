use super::*;

use std::process::{Command, Child, Stdio};
use std::thread;
use std::time::Duration;

pub struct GeckoDriverBuilder {
    port: u16,
    ff_binary: String,
    kill_on_drop: bool,
}

impl GeckoDriverBuilder {
    pub fn new() -> Self {
        GeckoDriverBuilder {
            port: 4444,
            ff_binary: "firefox".to_owned(),
            kill_on_drop: true,
        }
    }
    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }
    pub fn firefox_binary(mut self, binary: &str) -> Self {
        self.ff_binary = binary.to_owned();
        self
    }
    pub fn kill_on_drop(mut self, kill: bool) -> Self {
        self.kill_on_drop = kill;
        self
    }
    pub fn spawn(self) -> Result<GeckoDriver, Error> {
        let child = Command::new("geckodriver")
            .arg("-b")
            .arg(self.ff_binary)
            .arg("--webdriver-port")
            .arg(format!("{}", self.port))
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .stdout(Stdio::null())
            .spawn()?;

        // TODO: parameterize this
        thread::sleep(Duration::new(1, 500));
        Ok(GeckoDriver {
            child: child,
            url: format!("http://localhost:{}", self.port),
            kill_on_drop: self.kill_on_drop,
        })
    }
}

/// A geckodriver process
pub struct GeckoDriver {
    pub child: Child,
    url: String,
    kill_on_drop: bool,
}

impl GeckoDriver {
    pub fn spawn() -> Result<Self, Error> {
        GeckoDriverBuilder::new().spawn()
    }
    pub fn build() -> GeckoDriverBuilder {
        GeckoDriverBuilder::new()
    }
}

impl Drop for GeckoDriver {
    fn drop(&mut self) {
        if self.kill_on_drop {
            let _ = self.child.kill();
        }
    }
}

impl Driver for GeckoDriver {
    fn url(&self) -> &str {
        &self.url
    }
}

