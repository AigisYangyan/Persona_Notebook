# Releasing

This repository uses GitHub Releases as the public distribution channel for Windows installers and release binaries.

## Versioning

- stable releases use Semantic Versioning
- tag format is `vX.Y.Z`
- optional prerelease tags should use clear suffixes such as `v0.2.0-beta.1`

## Release Source of Truth

- app version: `src-tauri/tauri.conf.json`
- release history: `CHANGELOG.md`
- release workflow: `.github/workflows/release.yml`
- release notes categories: `.github/release.yml`

## Pre-Release Checklist

1. Confirm the app builds locally.
2. Run:

```bash
npm test
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
npm run tauri build
```

3. Update `CHANGELOG.md`.
4. Ensure `README.md` reflects current behavior.
5. Verify installer outputs launch correctly on Windows.

## First-Time GitHub Bootstrap

If this local directory has not been published before:

1. Create an empty public repository on GitHub.
2. Confirm your commit email is appropriate for a public repository.
3. Stage and commit locally.
4. Add the remote.
5. Push `main`.

Recommended commands:

```bash
git add .
git commit -m "chore: prepare public GitHub release"
git remote add origin https://github.com/<YOUR_USERNAME>/personal-growth-rpg-notebook.git
git push -u origin main
```

Important:
The commit author name and email recorded by Git come from your local Git configuration unless you override them. If you do not want to expose a personal email publicly, switch to a GitHub `noreply` address before the first public commit.

## GitHub Release Rule

The recommended release flow is:

1. Merge release-ready changes into `main`.
2. Bump the version in `src-tauri/tauri.conf.json` and `package.json` if needed.
3. Create an annotated git tag using `vX.Y.Z`.
4. Push `main` and the tag.
5. Let GitHub Actions build the release assets.
6. Review the generated draft release.
7. Publish the draft only after asset verification.

## Why Draft Releases

Draft releases reduce the risk of immediately publishing a broken installer, incorrect notes, or incomplete assets.

## Current Platform Scope

This repository currently ships a Windows-first release workflow.

That is intentional:

- Windows packaging is already validated locally
- helper scripts are Windows-oriented
- cross-platform release claims should not be made before platform-specific verification is in place
