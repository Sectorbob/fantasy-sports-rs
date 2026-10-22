# Contributing

Thanks for your interest. Issues and pull requests are welcome.

## Getting set up

You need Rust 1.88 or newer (`rustup update stable`). Then:

```shell
cargo test --workspace      # hermetic: no network and no credentials needed
```

## Before you open a pull request

The CI runs these; running them locally saves a round trip:

```shell
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
cargo deny check            # needs cargo-deny
```

Please add a test with a behaviour change or a bug fix. Tests must not need the network or credentials:
HTTP clients are tested against `wiremock` servers replaying fixtures. A test that really needs a live
service should be `#[ignore]`d with the reason and the required environment in its message.

## Test data

Fixtures must not contain real people's data. Use synthetic data, or anonymize recorded responses
consistently (replace names, avatars and ids everywhere so cross references still resolve). Do not commit
third-party content (tweets, videos, rankings) as fixtures.

## Commits

Small, focused commits with a message that explains *why*. Sign your commits if you can.

## License

By contributing you agree that your work is licensed under the same terms as the project,
`MIT OR Apache-2.0`.
