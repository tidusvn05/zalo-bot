# Releasing

1. Update `version` in `Cargo.toml` and move the `[Unreleased]` notes in `CHANGELOG.md` under
   `## [X.Y.Z] - YYYY-MM-DD`. Merge that to `main`.
2. Tag and push: `git tag vX.Y.Z && git push origin vX.Y.Z`.
3. The `Release` workflow checks the tag and changelog, runs the tests, publishes to crates.io, and
   creates the GitHub release.

Publishing uses crates.io **Trusted Publishing**: the crate trusts this repository, `release.yml` and
the GitHub environment `crates-io`. The workflow gets a 30-minute token through
`rust-lang/crates-io-auth-action`, so no crates.io token is stored in GitHub. The crate's
configuration is under Settings → Trusted Publishing on crates.io. To change it from a terminal,
run okbot's `scripts/release.py trustpub tidusvn05/zalo-bot --crates zalo-bot --only`.

Publishing by hand (`cargo publish` after `cargo login`) before pushing the tag also works: the
workflow then skips the publish step and only creates the GitHub release.

A published version cannot be deleted from crates.io, only yanked (`cargo yank --version X.Y.Z`).
