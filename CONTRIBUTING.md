# Contributing

Thanks for helping. Bug reports with the API answer you got (remove the token!) are especially useful,
because the Zalo Bot API is young and its documentation is incomplete.

## Development

```sh
cargo fmt --all
cargo clippy --all-features --all-targets -- -D warnings
cargo test --all-features
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
cargo deny check
```

Live tests are opt-in: `ZALO_BOT_TOKEN=… cargo test --test live -- --ignored`.

## Rules

- Never log or format a request URL: it contains the token.
- Every API field Zalo may extend is `#[non_exhaustive]` or has an `Other` variant.
- New API behaviour needs a mock-server test in `tests/api.rs`, with the real answer as fixture.
- Conventional commits (`feat: …`, `fix: …`); update `CHANGELOG.md`.
- By contributing you agree to license your work under MIT OR Apache-2.0.

## Code of Conduct

This project follows the [Contributor Covenant](CODE_OF_CONDUCT.md).
