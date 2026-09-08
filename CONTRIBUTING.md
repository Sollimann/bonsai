# Contributing to Bonsai

Thanks for taking the time to contribute! Bonsai is a small, embeddable behavior-tree
library, and we want to keep it that way. This document collects the guidelines and
workflow to help your contribution land smoothly.

## Table of contents

- [Reporting issues](#reporting-issues)
- [Design guidelines](#design-guidelines)
- [Development setup](#development-setup)
- [Code style](#code-style)
- [Python bindings](#python-bindings)
- [Running the tests](#running-the-tests)
- [Pull request workflow](#pull-request-workflow)

## Reporting issues

Before opening a new issue, search the existing
[issues](https://github.com/Sollimann/bonsai/issues) to avoid duplicates. A good bug
report includes:

- a short description of what you expected to happen and what actually happened,
- a minimal reproduction (a small Rust or Python snippet is ideal),
- your environment (OS, Rust toolchain, bonsai-bt version).

## Design guidelines

Bonsai deliberately stays lean. Please keep these principles in mind when proposing a
change:

- **Only add a new node primitive if the target behavior cannot be composed from the
  existing primitives** (`Sequence`, `Select`, `While`, `WhenAll`, `WhenAny`, `Race`,
  `After`, `Invert`, `AlwaysSucceed`, `If`, `Wait`, `WaitForever`, `Action`, ...) *and*
  there is real, demonstrated demand for it. Composing existing nodes is almost always
  preferable to growing the public API.

- **Keep the crate lean with as few dependencies as possible.** Fewer dependencies mean
  smaller binaries, relevance for embedded/WASM targets, and less chance of downstream
  breaking changes. In particular:
  - avoid I/O-heavy libraries (e.g. `tokio`) — Bonsai compiles to WASM and should stay
    usable in embedded contexts,
  - prefer standard-library solutions whenever one is good enough,
  - justify any new dependency in the PR description.

When in doubt, open an issue first to discuss the design before writing code.

## Development setup

You need a Rust toolchain (MSRV is 1.80.0) and the system libraries used by the
visualization feature:

```sh
make install-dev-deps   # installs libudev-dev, pkg-config, librust-alsa-sys-dev
```

Install and enable [pre-commit](https://pre-commit.com/):

```sh
pip install pre-commit
pre-commit install
```

The pre-commit hooks run `rustfmt`, `cargo clippy`, and (for changes under `bonsai-py/`)
regenerate the Python type stubs. Details are in [`DEVELOPMENT.md`](DEVELOPMENT.md).

## Code style

Rust code is formatted with `rustfmt` (see [`.rustfmt.toml`](.rustfmt.toml): edition 2021,
max width 120). Run:

```sh
cargo fmt --check
```

Clippy must pass cleanly:

```sh
cargo clippy --all-targets --all-features -- -D warnings
```

## Python bindings

The Python bindings live in [`bonsai-py/`](bonsai-py/). Type stubs are generated from
`#[gen_stub_*]` annotations and must be kept in sync with the Rust sources:

```sh
bash bonsai-py/scripts/regen-stubs.sh
```

If you change anything under `bonsai-py/src/`, regenerate the stub and commit the result.
To build and test the bindings locally:

```sh
cd bonsai-py
maturin develop --release
pytest -v bonsai-py/tests/
```

## Running the tests

```sh
cargo build --verbose
cargo build --examples
cargo test --verbose
```

## Pull request workflow

1. Fork the repository and create a feature branch from `main`.
2. Make your changes, keeping commits focused and well described.
3. Run the checks above (format, clippy, tests) locally.
4. Push your branch and open a pull request against `main`.
5. Describe *what* changed and *why*, and reference any issue it closes
   (e.g. `Closes #123`).

The CI (`rust-pr.yml`) will run the build, tests, and (for Python changes) `pytest`. Keep
it green and respond to review feedback.

## License

By contributing, you agree that your contributions will be licensed under the
[MIT License](LICENSE).
