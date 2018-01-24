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

[`geckodriver`](https://github.com/mozilla/geckodriver)
(WebDriver proxy for Firefox) is fully supported as a WebDriver backend by the
`webdriver_client::firefox::GeckoDriver` struct. This crate expects `geckodriver` to be on your path.

However HttpDriver will accept any WebDriver server's HTTP URL, so [ChromeDriver] for Chrome, [Microsoft WebDriver for Edge][ms-wd], `safaridriver` for Apple Safari, and [OperaDriver] for Opera should all work if you start the server yourself.

[ChromeDriver]: https://sites.google.com/a/chromium.org/chromedriver/getting-started
[ms-wd]: https://docs.microsoft.com/en-us/microsoft-edge/webdriver
[OperaDriver]: https://github.com/operasoftware/operachromiumdriver

### On Linux

The script `bin/download_geckodriver` downloads the Linux x64 geckodriver binary release from the [geckodriver Github releases page](https://github.com/mozilla/geckodriver/releases) to `bin/geckodriver`.

This snippet will download geckodriver and place it on your current shell's path:
```sh
bin/download_geckodriver
export PATH=$PATH:$PWD/bin
```

--------

## Tests

`cargo test` runs a few tests. Integration tests currently require Firefox to be
installed to `/usr/lib/firefox`.

--------

This fork is based on equalsraf's excellent work from <https://github.com/equalsraf/webdriver>.
