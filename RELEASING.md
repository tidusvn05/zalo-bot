# Releasing

1. Update `version` in `Cargo.toml` and move the `[Unreleased]` notes in `CHANGELOG.md` under
   `## [X.Y.Z] - YYYY-MM-DD`. Merge that to `main`.
2. Tag and push: `git tag vX.Y.Z && git push origin vX.Y.Z`.
3. The `Release` workflow checks the tag and changelog, runs the tests, publishes to crates.io with
   the `CARGO_REGISTRY_TOKEN` secret of the `crates-io` environment, and creates the GitHub release.

A published version cannot be deleted from crates.io, only yanked (`cargo yank --version X.Y.Z`).
