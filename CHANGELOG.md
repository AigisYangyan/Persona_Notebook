# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog, and this project intends to follow Semantic Versioning for public releases.

## [Unreleased]

### Added

- GitHub publishing documentation and release workflows
- Apache-2.0 license metadata and repository health files

### Changed

- README rewritten for public GitHub distribution
- release policy formalized around semantic version tags and GitHub Releases

## [0.1.0] - 2026-06-11

### Added

- five-page desktop workflow: Dashboard, Today, Calendar, Ledger, Settings
- SQLite-backed local task recording and growth ledger
- rollback support and daily review recalculation
- CSV/JSON import and JSON export
- Windows release bundles with NSIS and MSI outputs

### Changed

- scoring pipeline unified into `Rules + API feedback`
- API keys are no longer exposed back to the frontend
- release entry standardized to `src-tauri/target/release/pgrn.exe`
