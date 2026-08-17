# npm distribution

`npm i -g ssh-clipboard` installs a JavaScript launcher plus four native Rust executables:

```text
vendor/darwin-arm64/ssh-clipboard
vendor/darwin-amd64/ssh-clipboard
vendor/linux-arm64/ssh-clipboard
vendor/linux-amd64/ssh-clipboard
```

The launcher maps Node's `process.platform` and `process.arch` to one executable, inherits the terminal unchanged for Ratatui, and forwards the native exit status. It also exports the vendor directory to the Rust process. First-run setup can therefore upload the correct native executable to a peer with a different OS or architecture. Installation and ordinary use never download executable code.

## Building a package

The release workflow builds all four targets on their native GitHub-hosted runners. To reproduce the packaging step with a directory of flat release artifacts:

```sh
npm ci
npm test
npm run stage:binaries -- /path/to/release-binaries
npm pack
```

Staging records every binary's size and SHA-256 digest in `vendor/manifest.json`. The `prepack` hook rejects missing, non-executable, corrupt, or incorrectly formatted Mach-O/ELF payloads before npm can create the tarball.

## Publishing

The `ssh-clipboard` package name is reserved in the public npm registry by the `0.0.0-dev.ffffff` placeholder release. Replace that placeholder with the first complete release before advertising the global install command.

Configure npm trusted publishing for this GitHub repository and `.github/workflows/release.yml`, then set the GitHub repository variable `NPM_PUBLISH=true`. Version tags will build, verify, package, and publish through short-lived OIDC credentials with npm provenance; no long-lived npm token is stored in GitHub.

The repository metadata in `package.json` must exactly match the public GitHub repository for npm provenance. It is set to `justin-schroeder/ssh-clipboard`. The version in `package.json` and `Cargo.toml` must match the release tag.

## macOS signing

Apple Developer ID signing and notarization are not required for this npm-installed command-line package. The npm route does not launch a downloaded `.app` bundle, installer package, or disk image through Finder. The release workflow therefore does not access an Apple developer account.

If a future release adds a standalone macOS installer or GUI, sign the final Mach-O binaries with a dedicated Developer ID Application identity and notarize that distribution artifact. Treat signing credentials as release secrets; do not put them in the npm package or repository.
