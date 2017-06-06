use super::*;

use std::process::{Command, Child, Stdio};
use std::thread;
use std::time::Duration;
use std::net::TcpListener;


/// Find a TCP port number to use. This is racy but see
/// https://bugzilla.mozilla.org/show_bug.cgi?id=1240830
///
/// If port is Some, check if we can bind to the given port. Otherwise
/// pick a random port.
fn check_tcp_port(port: Option<u16>) -> io::Result<u16> {
    TcpListener::bind(&("localhost", port.unwrap_or(0)))
        .and_then(|stream| stream.local_addr())
        .map(|x| x.port())
}

pub struct GeckoDriverBuilder {
    port: Option<u16>,
    ff_binary: String,
    kill_on_drop: bool,
}

impl GeckoDriverBuilder {
    pub fn new() -> Self {
        GeckoDriverBuilder {
            port: None,
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
        let port = check_tcp_port(self.port)?;

        let child = Command::new("geckodriver")
            .arg("-b")
            .arg(self.ff_binary)
            .arg("--webdriver-port")
            .arg(format!("{}", port))
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .stdout(Stdio::null())
            .spawn()?;

        // TODO: parameterize this
        thread::sleep(Duration::new(1, 500));
        Ok(GeckoDriver {
            child: child,
            url: format!("http://localhost:{}", port),
            kill_on_drop: self.kill_on_drop,
        })
    }
}

/// A geckodriver process
pub struct GeckoDriver {
    child: Child,
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

