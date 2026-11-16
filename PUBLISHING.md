# Publishing mathcat_c to crates.io

[`.github/workflows/publish-crates.yml`](.github/workflows/publish-crates.yml) publishes `mathcat_c` to crates.io when a GitHub release is published.

## What the workflow does

1. Skips prereleases and drafts. The rolling `latest` development build is a prerelease and is never published.
2. Checks that the release tag, minus a leading `v`, matches `version` in `Cargo.toml`. For example, tag `v0.7.8` requires `version = "0.7.8"`.
3. Checks that `mathcat` at the same version is on crates.io and not yanked. `mathcat_c` tracks the MathCAT version, so publish MathCAT first.
4. Replaces the local `mathcat = {path = "../MathCAT/" ...}` dependencies with `=<version>` crates.io dependencies, because crates.io doesn't accept path dependencies.
5. Builds the crate, runs `cargo publish --dry-run`, then publishes using a short-lived token from crates.io Trusted Publishing. Packaging uses `--no-verify` because `build.rs` writes headers into `Example/`, which cargo's tarball verification rejects; the preceding build step validates the crate instead.

## One-time setup

Trusted Publishing uses GitHub OIDC, so no long-lived crates.io API token is stored in the repository.

1. A `mathcat_c` crate owner signs in to crates.io and opens **mathcat_c → Settings → Trusted Publishing → Add**:
   - Repository owner: `daisy`
   - Repository name: `MathCATForC`
   - Workflow filename: `publish-crates.yml`
   - Environment: `crates-io`
2. In the GitHub repository, create an environment named `crates-io` (**Settings → Environments**). Optionally, add required reviewers so a maintainer must approve each publish.

## Releasing

1. Make sure `mathcat` `X.Y.Z` is published on crates.io.
2. Set `version = "X.Y.Z"` in `Cargo.toml` and merge.
3. Publish a GitHub release with tag `vX.Y.Z`, and leave **Set as a pre-release** unchecked.
