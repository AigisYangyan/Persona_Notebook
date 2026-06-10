# Contributing

Thanks for considering a contribution to Personal Growth RPG Notebook.

## Ground Rules

- keep changes focused and reviewable
- prefer small pull requests
- do not commit secrets, API keys, or local database files
- preserve the confirm-before-write scoring flow
- keep business rules explicit and testable

## Development Setup

```bash
npm install
npm run tauri dev
```

Recommended validation before opening a pull request:

```bash
npm test
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
```

## Branch and Commit Conventions

- default branch: `main`
- feature branches: `feat/<topic>`, `fix/<topic>`, `docs/<topic>`
- commit style: Conventional Commits when practical

Examples:

- `feat: add calendar analyzed-state badges`
- `fix: normalize rules_api setting persistence`
- `docs: document GitHub release flow`

## Pull Requests

Please include:

- what changed
- why it changed
- how it was tested
- screenshots for UI changes when useful

## Release Expectations

- public releases use Semantic Versioning
- release tags must use `vX.Y.Z`
- user-facing changes should be reflected in `CHANGELOG.md`

See [docs/RELEASING.md](docs/RELEASING.md) for the full release checklist.
