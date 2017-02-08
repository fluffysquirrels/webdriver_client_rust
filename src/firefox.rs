use super::*;

use std::time::Duration;
use std::process::{Command, Child, Stdio};
use std::thread;

use rand::distributions::{IndependentSample, Range};

pub struct GeckoDriver {
    child: Child,
    url: String,
}

impl GeckoDriver {
    pub fn new() -> Result<Self, Error> {
        Self::with_binary("firefox")
    }

    pub fn with_binary(bin: &str) -> Result<Self, Error> {
        let mut rng = rand::thread_rng();
        let range = Range::new(30000, 60000);

        // FIXME loop a few times
        let port: u16 = range.ind_sample(&mut rng);
        let cmd = try!(Command::new("geckodriver")
                        .arg("-b")
                        .arg(bin)
                        .arg("--webdriver-port")
                        .arg(format!("{}", port))
                        .stdin(Stdio::null())
                        .stderr(Stdio::null())
                        .stdout(Stdio::null())
                        .spawn());

        // FIXME make this configurable
        thread::sleep(Duration::new(1, 900));
        Ok(GeckoDriver {
            child: cmd,
            url: format!("http://localhost:{}", port),
        })
    }
}

impl Drop for GeckoDriver {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}

impl Driver for GeckoDriver {
    fn url(&self) -> &str {
        &self.url
    }
}

