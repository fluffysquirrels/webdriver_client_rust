//! Support for the Chrome browser.

use super::*;

use std::process::{Command, Child, Stdio};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use super::util;

pub struct ChromeDriverBuilder {
    port: Option<u16>,
    kill_on_drop: bool,
}

impl ChromeDriverBuilder {
    pub fn new() -> Self {
        ChromeDriverBuilder {
            port: None,
            kill_on_drop: true,
        }
    }
    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }
    pub fn kill_on_drop(mut self, kill: bool) -> Self {
        self.kill_on_drop = kill;
        self
    }
    pub fn spawn(self) -> Result<ChromeDriver, Error> {
        let port = util::check_tcp_port(self.port)?;

        let child = Command::new("chromedriver")
            .arg(format!("--port={}", port))
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .stdout(Stdio::null())
            .spawn()?;

        // TODO: parameterize this
        thread::sleep(Duration::new(1, 500));
        Ok(ChromeDriver {
            inner: Arc::new(ChromeDriverInner {
                child: child,
                url: format!("http://localhost:{}", port),
                kill_on_drop: self.kill_on_drop,
            }),
        })
    }
}

/// A chromedriver process
#[derive(Clone)]
pub struct ChromeDriver {
    inner: Arc<ChromeDriverInner>,
}

struct ChromeDriverInner {
    child: Child,
    url: String,
    kill_on_drop: bool,
}

impl ChromeDriver {
    pub fn spawn() -> Result<Self, Error> {
        ChromeDriverBuilder::new().spawn()
    }
    pub fn build() -> ChromeDriverBuilder {
        ChromeDriverBuilder::new()
    }

    /// Start a session for this driver
    pub fn session(&self, params: &NewSessionCmd) -> Result<DriverSession, Error> where Self : Sized + 'static {
        DriverSession::create_session(Box::new(self.clone()), params)
    }
}

impl Drop for ChromeDriverInner {
    fn drop(&mut self) {
        if self.kill_on_drop {
            let _ = self.child.kill();
        }
    }
}

impl Driver for ChromeDriver {
    fn url(&self) -> &str {
        &self.inner.url
    }
}
