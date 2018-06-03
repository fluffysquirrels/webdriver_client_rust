# Release process

## `webdriver_client` crate

1.  Push changes to [GitHub][github].
1.  Check build locally with `bin/build_local`.
1.  Check Travis build: [![Build Status](https://travis-ci.org/fluffysquirrels/webdriver_client_rust.svg)][travis]

1.  Increment version number in Cargo.toml (major version if breaking changes).
1.  Commit to update the version number:

    `git commit -m "Update version number"`

1.  Add a git tag for the new version number:

    `git tag v1.2.3`

1.  Push to [GitHub][github]:

    `git push origin master && git push --tags`

1.  Publish to [crates.io][crates] with `bin/publish`.
1.  Check new version appears on
    [![Crate](https://img.shields.io/crates/v/webdriver_client.svg)][crates]
    and
    [![Documentation](https://docs.rs/webdriver_client/badge.svg)][docs]

[travis]: https://travis-ci.org/fluffysquirrels/webdriver_client_rust
[github]: https://github.com/fluffysquirrels/webdriver_client_rust
[crates]: https://crates.io/crates/webdriver_client
[docs]: https://docs.rs/webdriver_client
