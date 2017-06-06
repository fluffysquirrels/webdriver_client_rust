# WebDriver client library in Rust

`webdriver_client` on crates.io

[![crates.io](https://img.shields.io/crates/v/webdriver_client.svg)](https://crates.io/crates/webdriver_client)

[![Travis](https://img.shields.io/travis/fluffysquirrels/webdriver_client_rust.svg)](https://travis-ci.org/fluffysquirrels/webdriver_client_rust)

[![docs.rs](https://docs.rs/webdriver_client/badge.svg)](https://docs.rs/webdriver_client)

## Getting started

Currently only [geckodriver](https://github.com/mozilla/geckodriver) (WebDriver proxy for Firefox) is supported as a WebDriver backend.

This crate expects `geckodriver` to be on your path.

### On Linux

The script `bin/download_geckodriver` downloads the Linux x64 geckodriver binary release from the [geckodriver Github releases page](https://github.com/mozilla/geckodriver/releases) to `bin/geckodriver`.

This snippet will download geckodriver and place it on your current shell's path:
```sh
bin/download_geckodriver
export PATH=$PATH:$PWD/bin
```

This fork is based on equalsraf's excellent work from <https://github.com/equalsraf/webdriver>.
