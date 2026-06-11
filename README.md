# Personal Growth RPG Notebook

Local-first desktop notebook for structured self-tracking, daily review, and growth scoring.

## Overview

Personal Growth RPG Notebook (`PGRN`) is a Tauri desktop application built around a five-dimension growth model:

- `knowledge` / 学识
- `willpower` / 觉悟
- `expression` / 表达
- `physique` / 体魄
- `bond` / 羁绊

The current scoring pipeline is intentionally constrained:

- local deterministic `Rules` generate a first-pass scoring cache
- the API receives those `Rules` hints as structured constraints
- the user confirms the preview before anything is written to the ledger

This means the app no longer uses a pure offline scorer as the final authority, and it also avoids unconstrained API scoring.

## Current Scope

- local-first desktop app with SQLite persistence
- daily task capture with `title + minutes + difficulty_star`
- hybrid `Rules + API feedback` scoring preview
- explicit confirm-before-write ledger flow
- rollback and daily review recalculation
- calendar, dashboard, ledger, import/export, and settings pages
- Windows release bundles via NSIS / MSI

## Privacy Model

- task data is stored locally
- API keys are persisted locally and only read by the Rust side
- the frontend only receives `api_key_configured: boolean`
- scoring previews are validated before they can be confirmed

## Screens and Features

| Page | Purpose |
|------|---------|
| Dashboard | levels, EXP, radar chart, streaks, badges |
| Today | record tasks, generate scoring preview, confirm write |
| Calendar | browse recorded / analyzed days |
| Ledger | inspect growth changes and rollback |
| Settings | API config, API key management, import/export |

## Tech Stack

- Tauri v2
- Rust
- Vue 3
- Vite
- TypeScript
- Naive UI
- Pinia
- rusqlite / SQLite
- Zod
- Vitest

## Local Development

Install dependencies:

```bash
npm install
```

Start the desktop app in development mode:

```bash
npm run tauri dev
```

Or use the helper script:

```bash
run-dev.bat
```

## Build and Run Release

Build a local release:

```bash
build-release.bat
```

Standard outputs:

```text
src-tauri\target\release\pgrn.exe
src-tauri\target\release\bundle\nsis\Personal Growth RPG Notebook_0.2.0_x64-setup.exe
src-tauri\target\release\bundle\msi\Personal Growth RPG Notebook_0.2.0_x64_en-US.msi
```

Run the built release:

```bash
run-release.bat
```

Do not use any manually copied root-level `pgrn.exe`. The supported release entry is `src-tauri\target\release\pgrn.exe` or the generated installer packages.

## Test Commands

```bash
npm test
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
npm run tauri build
```

## Data Location

The default local database path is:

```text
%LOCALAPPDATA%\com.pgrn.app\data.db
```

Note:
The current Tauri identifier is intentionally kept as `com.pgrn.app` for compatibility with existing local data.

## Release Policy

This repository follows:

- Semantic Versioning for public releases
- tag format `vX.Y.Z`
- `CHANGELOG.md` for human-maintained release history
- GitHub Releases for downloadable installers and release notes
- GitHub Actions for CI and tagged release builds

Detailed publishing rules:

- [docs/RELEASING.md](docs/RELEASING.md)
- [.github/workflows/release.yml](.github/workflows/release.yml)
- [.github/release.yml](.github/release.yml)

## Documentation

- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
- [docs/RELEASING.md](docs/RELEASING.md)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- [SECURITY.md](SECURITY.md)

## License

Apache-2.0. See [LICENSE](LICENSE).
