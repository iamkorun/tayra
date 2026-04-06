# 🦡 tayra

[![Crates.io](https://img.shields.io/crates/v/tayra.svg)](https://crates.io/crates/tayra)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![GitHub Stars](https://img.shields.io/github/stars/iamkorun/tayra?style=flat)](https://github.com/iamkorun/tayra)
[![CI](https://github.com/iamkorun/tayra/actions/workflows/ci.yml/badge.svg)](https://github.com/iamkorun/tayra/actions/workflows/ci.yml)
[![Buy Me A Coffee](https://img.shields.io/badge/Buy%20Me%20A%20Coffee-support-yellow?style=flat&logo=buy-me-a-coffee)](https://buymeacoffee.com/iamkorun)

> Smart semver bumper — reads your git commits, suggests the next version.

---

## The Problem

Every release cycle, you open `git log`, squint at a wall of commits, try to remember what changed since last tag, and manually decide: is this a patch? A minor? A major?

It's tedious, it's error-prone, and it breaks CI when someone forgets.

## The Solution

tayra reads your git history since the last version tag, parses [Conventional Commits](https://www.conventionalcommits.org/), and tells you exactly what to bump — and why.

```
$ tayra
tayra v0.1.0
━━━━━━━━━━━━━━━━━━━━━
Current version: v1.4.2
Suggested bump:  minor → v1.5.0

Commits since v1.4.2:
  feat(auth): add OAuth2 support
  fix: handle empty token correctly
  chore: update dependencies
  docs: expand API reference

Breakdown: 1 feat, 1 fix, 1 chore, 1 docs → minor
```

No config. No setup. Just run it.

---

## Demo

<!-- ![demo](docs/demo.gif) -->

---

## Quick Start

```sh
cargo install tayra
cd your-repo
tayra
```

---

## Installation

### From crates.io (recommended)

```sh
cargo install tayra
```

### From source

```sh
git clone https://github.com/iamkorun/tayra
cd tayra
cargo install --path .
```

---

## Usage

### Basic usage

Run tayra in any git repository with version tags:

```sh
$ tayra
tayra v0.1.0
━━━━━━━━━━━━━━━━━━━━━
Current version: v2.0.1
Suggested bump:  patch → v2.0.2

Commits since v2.0.1:
  fix(parser): handle malformed input
  chore: bump serde to 1.0.197

Breakdown: 1 fix, 1 chore → patch
```

### Feature branch with breaking change

```sh
$ tayra
tayra v0.1.0
━━━━━━━━━━━━━━━━━━━━━
Current version: v1.3.0
Suggested bump:  major → v2.0.0

Commits since v1.3.0:
  feat!: redesign public API
  feat(core): add streaming support
  fix: correct edge case in parser

Breakdown: 2 feat, 1 fix → major
```

### New repository (no tags yet)

```sh
$ tayra
tayra v0.1.0
━━━━━━━━━━━━━━━━━━━━━
Current version: none (assuming 0.0.0)
Suggested bump:  minor → 0.1.0

Commits since beginning:
  feat: initial implementation
  chore: add CI workflow
  docs: write README

Breakdown: 1 feat, 1 chore, 1 docs → minor
```

### Create the tag automatically

```sh
$ tayra --tag
tayra v0.1.0
━━━━━━━━━━━━━━━━━━━━━
Current version: v1.2.3
Suggested bump:  minor → v1.3.0

Commits since v1.2.3:
  feat: add JSON output format

Breakdown: 1 feat → minor

Tag 'v1.3.0' created at HEAD.
```

### Custom prefix

```sh
# Use a custom tag prefix (e.g., "release-")
$ tayra --prefix release-
tayra v0.1.0
━━━━━━━━━━━━━━━━━━━━━
Current version: release-1.0.0
Suggested bump:  patch → release-1.0.1
...
```

### Run against a different repository path

```sh
$ tayra --path /path/to/other-repo
```

---

## CI/CD Integration

Use `--ci` for machine-readable output — just the version string, nothing else:

```sh
$ tayra --ci
v1.5.0
```

### GitHub Actions example

```yaml
jobs:
  release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0  # required: fetch all tags

      - uses: dtolnay/rust-toolchain@stable

      - name: Install tayra
        run: cargo install tayra

      - name: Get next version
        id: version
        run: echo "version=$(tayra --ci)" >> $GITHUB_OUTPUT

      - name: Tag and release
        run: |
          git tag ${{ steps.version.outputs.version }}
          git push origin ${{ steps.version.outputs.version }}
```

### GitLab CI example

```yaml
release:
  script:
    - cargo install tayra
    - NEXT_VERSION=$(tayra --ci)
    - git tag "$NEXT_VERSION"
    - git push origin "$NEXT_VERSION"
```

---

## How It Works

tayra uses [Conventional Commits](https://www.conventionalcommits.org/) to determine the bump level:

| Commit type | Bump |
|-------------|------|
| `fix:`, `chore:`, `docs:`, `refactor:`, `test:`, `ci:`, `style:`, `build:` | **patch** |
| `feat:` | **minor** |
| Any commit with `!` suffix or `BREAKING CHANGE:` in body | **major** |

The highest bump level across all commits since the last tag wins.

**Tag detection:** tayra auto-detects your tag prefix (e.g. `v` in `v1.2.3`, or none in `1.2.3`) and preserves it in the suggestion.

---

## Features

- Reads git history since the last semver tag automatically
- Parses Conventional Commits — feat, fix, chore, docs, refactor, test, ci, perf, style, build
- Detects breaking changes via `!` suffix or `BREAKING CHANGE:` in commit body
- Auto-detects tag prefix (`v` or bare)
- `--tag` flag creates the git tag for you
- `--ci` flag outputs only the version string — perfect for scripts and pipelines
- `--prefix` flag overrides the tag prefix
- `--path` flag for running against a different repository
- No config file — works with zero setup
- Pure Rust, single binary, blazing fast

---

## Contributing

Contributions are welcome. Please open an issue before submitting a large PR.

```sh
git clone https://github.com/iamkorun/tayra
cd tayra
cargo test
```

All tests must pass. Follow Conventional Commits for your commit messages (tayra uses itself to version its own releases).

---

## License

MIT — see [LICENSE](LICENSE)

---

## Star History

<a href="https://star-history.com/#iamkorun/tayra&Date">
  <img src="https://api.star-history.com/svg?repos=iamkorun/tayra&type=Date" alt="Star History Chart" width="600">
</a>

---

<p align="center">
  <a href="https://buymeacoffee.com/iamkorun"><img src="https://cdn.buymeacoffee.com/buttons/v2/default-yellow.png" alt="Buy Me a Coffee" width="200"></a>
</p>
