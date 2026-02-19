# Contributing to Arb

## Setup

```bash
# Clone the repository
git clone https://github.com/tw93/Arb.git
cd Arb

# Install required tools (cargo-nextest, nightly rustfmt)
make install-tools
```

## Development

| Command | Purpose |
|---------|---------|
| `make fmt` | Auto-format code (requires nightly) |
| `make fmt-check` | Check formatting without modifying files |
| `make check` | Compile check, catch type/syntax errors |
| `make test` | Run unit tests |
| `make dev` | Fast local debug: build `arb-gui` and run from `target/debug` |
| `make build` | Compile binaries (no app bundle) |
| `make app` | Build debug app bundle → `dist/Arb.app` |

**Recommended workflow:**

```bash
make fmt        # format first
make check      # verify it compiles
make test       # run tests
make dev        # fast local run without packaging
```

You can override log level for `make dev`:

```bash
RUST_LOG=debug make dev
```

## Build Release

```bash
# Build application and DMG (release, universal binary)
./scripts/build.sh
# Outputs: dist/Arb.app and dist/Arb.dmg

# Build for current architecture only (faster, for local testing)
./scripts/build.sh --native-arch

# Build app bundle only (skip DMG creation)
./scripts/build.sh --native-arch --app-only

# Build and open the app automatically
./scripts/build.sh --native-arch --open
```

## Pull Requests

1. Fork and create a branch from `main`
2. Make changes
3. Run `make fmt && make check && make test`
4. Commit and push
5. Open PR targeting `main`

CI runs format check → unit tests → cargo check → universal build validation in order.
