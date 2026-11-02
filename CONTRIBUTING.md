# Contributing

## Communication

Ask directly. Include the exact command, error, and what you already tried; "it doesn't work" is not actionable. Describe the problem you are trying to solve, not only the change you have in mind.

## Bug reports

Use the bug report template. The best report is a minimal, reproducible example: exact steps, expected and actual result, logs, environment, and what you already tried.

## Branches

`main` is always releasable. Work on a short-lived branch and open a pull request into `main`:

- `feat/<short-name>`
- `fix/<short-name>`
- `docs/<short-name>`
- `refactor/<short-name>`
- `chore/<short-name>`

Releases are tagged `v<version>` from `main`; the tag must match `Cargo.toml` and `assets/manifest.json`.

## Commits

Use [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/):

```
feat: add battery image
fix: refresh title after key creation
docs: describe the device picker
```

Common types: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`, `ci`, `build`, `perf`. Mark breaking changes with `!` (`feat!:`) or a `BREAKING CHANGE:` footer. Use the same convention for pull request titles.

## Checks

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
node --check assets/pi/main.js
scripts/package.sh
```

## License

OpenG is free software under the [GNU Affero General Public License, version 3](LICENSE). By contributing, you agree that your changes are licensed under the same terms.
