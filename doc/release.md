# Release process

## `webdriver_client` crate

1. Push changes to [GitHub][github].
1. Check build locally with `bin/build_local`.
1. Check Travis build: [![Build Status](https://travis-ci.org/fluffysquirrels/webdriver_client_rust.svg)][travis]

   [travis]: https://travis-ci.org/fluffysquirrels/webdriver_client_rust
1. Increment version number in Cargo.toml (major version if breaking changes).
1. Commit to update the version number.
1. Add a git tag for the new version number. Push it to [GitHub][github].
1. Publish with `bin/publish`.
1. Check new version appears on
   [![Crate](https://img.shields.io/crates/v/webdriver_client.svg)][crates]
   and
   [![Documentation](https://docs.rs/webdriver_client/badge.svg)][docs]

   [github]: https://github.com/fluffysquirrels/webdriver_client_rust
   [crates]: https://crates.io/crates/webdriver_client
   [docs]: https://docs.rs/webdriver_client
