# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog, and this project intends to follow Semantic Versioning for public releases.

## [Unreleased]

## [0.2.0] - 2026-06-11

### Added

- cyberpunk UI theme assets and shared global styling primitives
- GitHub publishing documentation and release workflows
- Apache-2.0 license metadata and repository health files

### Changed

- redesigned Dashboard, Today, Calendar, Ledger, and Settings with the new sidebar-first UI
- enforced a minimum desktop window size so the refactored layout stays usable while resizing
- README rewritten for public GitHub distribution
- release policy formalized around semantic version tags and GitHub Releases

### Fixed

- calendar today badge no longer implies a record already exists
- out-of-month calendar cells no longer present themselves as interactive
- removed release-noisy import error logging from Settings

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
