# WebDriver client library in Rust

`webdriver_client` on crates.io

[![crates.io](https://img.shields.io/crates/v/webdriver_client.svg)](https://crates.io/crates/webdriver_client)

[![docs.rs](https://docs.rs/webdriver_client/badge.svg)](https://docs.rs/webdriver_client)

Source code and issues on GitHub:
[![GitHub last commit](https://img.shields.io/github/last-commit/fluffysquirrels/webdriver_client_rust.svg)][github]

   [github]: https://github.com/fluffysquirrels/webdriver_client_rust

CI build on Travis CI: [![Build Status](https://travis-ci.org/fluffysquirrels/webdriver_client_rust.svg)](https://travis-ci.org/fluffysquirrels/webdriver_client_rust)

Pull requests welcome.

## Getting started

[GeckoDriver] and [ChromeDriver] are fully supported as WebDriver backends by the `webdriver_client::firefox::GeckoDriver` and `webdriver_client::chrome::ChromeDriver` structs. This crate expects the driver to be on your path.

However HttpDriver will accept any WebDriver server's HTTP URL, so
[Microsoft WebDriver for Edge][ms-wd], `safaridriver` for Apple
Safari, and [OperaDriver] for Opera should all work if you start the
server yourself.

[GeckoDriver]: https://github.com/mozilla/geckodriver
[ChromeDriver]: https://sites.google.com/a/chromium.org/chromedriver/getting-started
[ms-wd]: https://docs.microsoft.com/en-us/microsoft-edge/webdriver
[OperaDriver]: https://github.com/operasoftware/operachromiumdriver

### On Linux

The scripts `bin/download_geckodriver` and `bin/download_chromedriver` download the Linux x64 binary releases for geckodriver and chromedriver.

This snippet will download the drivers and place it on your current shell's path:
```sh
bin/download_geckodriver
bin/download_chromedriver
export PATH=$PATH:$PWD/bin
```

## Tests

`cargo test` runs a few tests. Integration tests require geckodriver and chromedriver to be installed.

## Changelog

### master from v0.2.3

* Added screenshot support: `DriverSession::screenshot()` and `Element::screenshot()`.

### v0.2.0

* Added ChromeDriver.
* `www` bin has new commands: `frames`, `switchframe`.
* Breaking change: `Driver::session and DriverSession::create_session` take
  a `NewSessionCmd` argument that specifies the session capabilities.
* New method on `DriverSession`: `browser_name`
* New methods on `Element`: `property`, `clear`, `find_element`, `find_elements`
  `raw_reference`.
* Integration tests: many more of them, more assertions and a built-in HTTP server.

---------------

This fork is based on equalsraf's excellent work from <https://github.com/equalsraf/webdriver>.
