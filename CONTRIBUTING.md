# Contributing

Thanks for helping. Issues and pull requests are welcome, in English or German.

## Setup

```bash
npm install
npx tauri dev          # app with hot reload
```

Needs Rust (stable), Node.js 24 and the [Tauri prerequisites](https://tauri.app/start/prerequisites/).

## Before you open a pull request

The same checks run in CI; a pull request can only be merged when they pass.

```bash
npm run check                      # svelte-check (types, a11y)
npm test                           # vitest
cd src-tauri
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## Commit messages

Releases and the changelog are generated from commit messages
([Conventional Commits](https://www.conventionalcommits.org/)):

| Prefix | Meaning | Version |
|---|---|---|
| `feat: …` | new feature | minor (0.1.0 → 0.2.0) |
| `fix: …` | bug fix, also dependency updates (`fix(deps): …`) | patch (0.1.0 → 0.1.1) |
| `feat!: …` or `BREAKING CHANGE:` in the body | incompatible change | minor before 1.0, major after |
| `docs:`, `ci:`, `test:`, `refactor:`, `chore:` | no release on their own | none |

## Releases

Maintainers merge the "Release PR" opened by release-please. That tags the version, builds the tarball, `.deb` and `.rpm`, attaches them to the GitHub release and updates the AUR packages.

## License

By contributing you agree that your contribution is licensed under [GPL-3.0-or-later](LICENSE).
