# Plan: Python bindings for `bonsai-bt` via PyO3

## Context

`bonsai-bt` is a pure-Rust behavior-tree library ([bonsai/src/lib.rs](bonsai/src/lib.rs)). Its core surface is small — a recursive `Behavior<A>` enum, a `BT<A, B>` executor with a `tick()` method that takes a user closure, plus an optional WebSocket visualizer attached via `BT::with_telemetry(port)` (see [bonsai/src/bt_telemetry.rs:94](bonsai/src/bt_telemetry.rs#L94)). There are no existing bindings, no C deps, and `BT<A, B>` carries no lifetimes — a clean fit for PyO3.

Goal: expose the full library to Python so users can build behavior trees, tick them with a Python callback, and open the same browser visualizer. Action and blackboard types will be arbitrary `PyObject`s so Python users aren't constrained to a fixed shape. The crate will be a new workspace member and ship as a published wheel on PyPI.

## Resolved decisions

| Decision | Choice |
|---|---|
| Action type in Python | Arbitrary `PyObject` (strings, dicts, dataclasses — anything) |
| Blackboard type | Arbitrary `PyObject` (live reference, mutations persist) |
| Visualizer (`with_telemetry`) | Included via `visualize` feature |
| Crate location | New workspace member `bonsai-py/` |
| Distribution | Full PyPI pipeline (wheels + sdist on tag) |
| PyPI dist name | `bonsai-py` (Python import is `bonsai_py` — PyPI's automatic hyphen→underscore mapping) |
| `BT.with_telemetry(port)` | Returns `self` (chainable); two-line usage also documented |
| `ActionArgs` fields | `dt: float`, `action: Any` only (no `event`) |
| Execution mode | Sequential — each section completes & is reviewed before the next |
| Python version floor | 3.10+ via `abi3-py310` |

## Crate layout

Add a third workspace member `bonsai-py/` (sibling of `bonsai/` and `examples/`). Update [Cargo.toml](Cargo.toml) workspace `members` to `["bonsai", "examples", "bonsai-py"]`.

```
bonsai-py/
├── Cargo.toml            # cdylib, pyo3 + bonsai-bt deps
├── pyproject.toml        # maturin build backend, package metadata
├── README.md
├── src/
│   ├── lib.rs            # #[pymodule] root, registers classes/functions
│   ├── behavior.rs       # PyBehavior + factory functions
│   ├── bt.rs             # PyBT class wrapping BT<PyObject, PyObject>
│   ├── status.rs         # PyStatus enum
│   └── action_args.rs    # PyActionArgs passed to user callback
├── python/bonsai_py/
│   ├── __init__.py       # re-export from the compiled module
│   └── py.typed          # PEP 561 marker
├── stubs/bonsai_py.pyi   # type stubs for IDE/mypy
├── tests/test_bt.py      # pytest suite
└── examples/visualizer_smoke.py
```

Key dependency choices in `bonsai-py/Cargo.toml`:
- `pyo3 = { version = "0.28", features = ["extension-module", "abi3-py310"] }`
- `bonsai-bt = { path = "../bonsai", version = "0.12", features = ["visualize"] }`
- `[lib] crate-type = ["cdylib"]`

## Rust → Python type mapping

| Rust type                            | Python exposure                                                   |
|--------------------------------------|-------------------------------------------------------------------|
| `Behavior<PyObject>`                 | `Behavior` opaque `#[pyclass]`; built via factory funcs           |
| `BT<PyObject, PyObject>`             | `BT` `#[pyclass(unsendable)]` — PyObject is not `Send`            |
| `Status` (Success/Failure/Running)   | `Status` `#[pyclass(eq, eq_int)]` enum                            |
| `ActionArgs<'_, Event, PyObject>`    | `ActionArgs` `#[pyclass]` with `.dt`, `.action` (no `.event`)     |
| `with_telemetry(port) -> io::Result` | `BT.with_telemetry(port)` chainable, raises `OSError` on bind fail |

**Why `unsendable`:** `PyObject` isn't `Send`. `#[pyclass(unsendable)]` tells PyO3 to panic if `BT` crosses thread boundaries — safe because all calls happen under the GIL.

## Python API surface (target shape)

```python
import bonsai_py as bt

tree = bt.Sequence([
    bt.Wait(1.0),
    bt.Action("inc"),
    bt.If(bt.Action("low_hp"),
          bt.AlwaysSucceed(bt.Action("flee")),
          bt.Action("regroup")),
    bt.While(bt.Action("has_ammo"),
             [bt.Action("fire"), bt.Wait(0.3)]),
])

# Both styles work — with_telemetry returns self:
tree_bt = bt.BT(tree, {"count": 0}).with_telemetry(8910)
# or:
tree_bt = bt.BT(tree, {"count": 0})
tree_bt.with_telemetry(8910)

def on_action(args, bb):
    if args.action == "inc":
        bb["count"] += 1
        return (bt.Status.Success, args.dt)
    return (bt.Status.Running, 0.0)

status, remaining_dt = tree_bt.tick(dt=0.5, callback=on_action)
print(tree_bt.blackboard()["count"], tree_bt.is_finished())
tree_bt.reset_bt()
```

---

# Agent-ready sections

Sections are designed for sequential execution. Each is self-contained: a downstream agent reading only its own section plus the **shared overview above** has everything needed. Acceptance criteria define the handoff to the next section.

---

## Section 1 — Scaffolding & build setup

**Goal.** Stand up `bonsai-py/` so that `maturin develop` produces an importable (empty) Python module.

**Depends on:** nothing.

### Resolved decisions (Section-1 scope)

| Decision | Choice | Rationale |
|---|---|---|
| Python version floor | `abi3-py310` | Python 3.8/3.9 EOL by 2026; py310 lets us use modern typing in stubs in §5. |
| PyO3 version | `0.28` (caret pin, latest stable confirmed via `cargo search pyo3` on 2026-05-15) | Mature `Bound<>` API; matches Section 4's tick-bridge sketch verbatim. |
| bonsai-bt dep style | Dual `path = "../bonsai", version = "0.12"` | Local builds use path; sdist consumers resolve from crates.io. **Requires `bonsai-bt 0.12.x` published to crates.io before any sdist upload.** |
| `f32` feature | Not exposed | Python floats are 64-bit; enabling f32 would silently lose precision. |
| `visualize` feature | Always enabled in `bonsai-py/Cargo.toml` | Resolved upstream; keeps the Python wheel self-contained. |
| Single name across layers | Cargo package = `bonsai-py`; PyPI distribution = `bonsai-py`; Python import = `bonsai_py` | The PyPI↔import hyphen↔underscore mapping is automatic across the Python ecosystem; users still type one name. |
| `[lib].name` | `bonsai_py` | Controls the produced `.so`/`.pyd` filename; must match the `#[pymodule]` function name and pyproject `module-name`. |
| License | MIT | Mirrors the parent crate. |
| Publish to crates.io? | No (`publish = false`) | The binding crate is built into wheels, not distributed via crates.io. |

### Files to create

Exact file contents follow — paste as-is.

#### `bonsai-py/Cargo.toml`

```toml
[package]
name = "bonsai-py"
version = "0.12.0"
edition = "2021"
rust-version = "1.80.0"
description = "Python bindings for the bonsai-bt behavior tree library"
license = "MIT"
authors = [
    "Kristoffer Solberg Rakstad <kristoffer.solberg@cognite.com>",
    "Anmol Kathail <anmolkathail@gmail.com>",
]
repository = "https://github.com/sollimann/bonsai.git"
homepage = "https://github.com/sollimann/bonsai"
publish = false

[lib]
name = "bonsai_py"
crate-type = ["cdylib"]

[dependencies]
pyo3 = { version = "0.28", features = ["extension-module", "abi3-py310"] }
bonsai-bt = { path = "../bonsai", version = "0.12", features = ["visualize"] }
```

Notes on each line worth understanding:
- `name = "bonsai-py"` is the **Cargo package name** and also the **PyPI distribution name** (set in `pyproject.toml` below). Python imports replace the hyphen with an underscore, giving `import bonsai_py`. This is the universal PyPI↔import convention — users only learn one name.
- `[lib].name = "bonsai_py"` controls the compiled artifact filename. PyO3 requires this to match the `#[pymodule]` function name; maturin requires it to match `tool.maturin.module-name`. All three must be **`bonsai_py`** (underscored) for `import bonsai_py` to find `PyInit_bonsai_py`.
- `crate-type = ["cdylib"]` is what produces the dynamic library Python loads. Not `rlib`, not `dylib`.
- `pyo3` features:
  - `extension-module` tells PyO3 we're building a Python C extension (skips linking libpython, needed on Linux for portable wheels).
  - `abi3-py310` enables the stable ABI floor at Python 3.10. One built wheel works for 3.10, 3.11, 3.12, 3.13, and future 3.x versions.
- `bonsai-bt = { path = ..., version = "0.12" }` — the **dual** form is load-bearing. With path only, Cargo refuses to package (sdist build fails). With version only, local dev wouldn't pick up uncommitted changes in `../bonsai/`. Both together: local cargo uses path; published sdist falls back to crates.io.

#### `bonsai-py/pyproject.toml`

```toml
[build-system]
requires = ["maturin>=1.7,<2.0"]
build-backend = "maturin"

[project]
name = "bonsai-py"
description = "Behavior trees in Python, powered by the bonsai-bt Rust crate."
readme = "README.md"
license = { text = "MIT" }
requires-python = ">=3.10"
authors = [
  { name = "Kristoffer Solberg Rakstad", email = "kristoffer.solberg@cognite.com" },
  { name = "Anmol Kathail", email = "anmolkathail@gmail.com" },
]
classifiers = [
  "Development Status :: 4 - Beta",
  "Intended Audience :: Developers",
  "License :: OSI Approved :: MIT License",
  "Programming Language :: Python :: 3",
  "Programming Language :: Python :: 3 :: Only",
  "Programming Language :: Python :: 3.10",
  "Programming Language :: Python :: 3.11",
  "Programming Language :: Python :: 3.12",
  "Programming Language :: Python :: 3.13",
  "Programming Language :: Rust",
  "Topic :: Scientific/Engineering :: Artificial Intelligence",
  "Topic :: Games/Entertainment",
  "Typing :: Typed",
]
dynamic = ["version"]

[project.urls]
Homepage = "https://github.com/sollimann/bonsai"
Repository = "https://github.com/sollimann/bonsai"
Issues = "https://github.com/sollimann/bonsai/issues"

[tool.maturin]
module-name = "bonsai_py"
python-source = "python"
features = ["pyo3/extension-module"]
strip = true
```

Notes:
- `dynamic = ["version"]` — maturin auto-populates the project version from `Cargo.toml`'s `package.version`. Don't duplicate it here, or you risk drift between the wheel filename and the package metadata.
- `python-source = "python"` — required so maturin recognizes `python/bonsai_py/` as the package layout and places the compiled `.so` *inside* the `bonsai_py/` Python package (not at the top level). Without this, `from .bonsai_py import *` in `__init__.py` fails.
- `strip = true` — strips debug symbols from the built `cdylib`, shrinking the wheel by ~5-10 MB.
- No `include = [...]` for `../bonsai/**` — superseded by the dual-version dep. Adding it would bundle bonsai's source into the sdist redundantly.

#### `bonsai-py/src/lib.rs`

```rust
use pyo3::prelude::*;

/// Python bindings for the bonsai-bt behavior-tree library.
///
/// Construct trees with the factory functions (Sequence, Action, Wait, …),
/// wrap one in `BT(tree, blackboard)`, and drive it with `bt.tick(dt, callback)`.
#[pymodule]
fn bonsai_py(_py: Python<'_>, _m: &Bound<'_, PyModule>) -> PyResult<()> {
    Ok(())
}
```

Notes:
- Function name `bonsai_py` **must** match `[lib].name` and `tool.maturin.module-name`. PyO3 emits `PyInit_bonsai_py`; the dynamic loader looks for that symbol in `bonsai_py.so`.
- The `&Bound<'_, PyModule>` parameter shape is the PyO3 ≥0.22 signature. If pinning to an older PyO3, switch to `&PyModule`.
- The module docstring shows up under `help(bonsai_py)` and on PyPI — keep it short but useful even at the scaffolding stage.

#### `bonsai-py/python/bonsai_py/__init__.py`

```python
"""bonsai-py — behavior trees in Python, powered by the Rust bonsai-bt crate."""

from importlib.metadata import PackageNotFoundError, version as _version

try:
    __version__ = _version("bonsai-py")
except PackageNotFoundError:  # editable install before metadata is in place
    __version__ = "0.0.0+unknown"

from .bonsai_py import *  # noqa: F401,F403  (re-export the compiled module)
```

Notes:
- `importlib.metadata.version("bonsai-py")` reads from the installed wheel's METADATA. This keeps `__version__` in lockstep with the PyPI name without hard-coding.
- The `from .bonsai_py import *` line is what makes `bonsai_py.BT`, `bonsai_py.Status`, etc. resolve at top level once Sections 2–4 land. `# noqa: F401,F403` is for linters; remove if the project doesn't use ruff/flake8.

#### `bonsai-py/python/bonsai_py/py.typed`

Empty file. PEP 561 marker — type checkers (`mypy`, `pyright`) only honor inline types or stub files if this marker is present.

#### `bonsai-py/README.md`

````markdown
# bonsai-py — Python bindings

Python bindings for the [bonsai-bt](https://github.com/sollimann/bonsai)
behavior-tree library.

> Scaffolding only — public API arrives in subsequent commits. See the
> repository for the full design plan.

## Installation (dev)

```bash
python -m venv .venv
source .venv/bin/activate   # Windows: .venv\Scripts\Activate.ps1
pip install maturin
cd bonsai-py
maturin develop
python -c "import bonsai_py; print(bonsai_py.__version__)"
```

## License

MIT — see [LICENSE](../LICENSE).
````

Real placeholder (not blank), so a PyPI render won't break if a release is cut before Section 9 fills it in.

### Files to modify

#### Root [Cargo.toml](Cargo.toml)

```toml
[workspace]
resolver = "2"
members = ["bonsai", "examples", "bonsai-py"]
```

Append `"bonsai-py"` to the existing two-entry list. Keep `resolver = "2"` (already there per commit `341b7a7`). No other change.

### Edge cases & pitfalls

The implementer should read this list **before** typing anything — most pitfalls produce confusing errors hours into debugging.

1. **Hyphen vs underscore drift.** One conceptual name (`bonsai-py`) flows through every layer, but Python module identifiers can't contain hyphens — so the *import* name is `bonsai_py` (underscore). Concretely: PyPI distribution `bonsai-py` ↔ Cargo package `bonsai-py` ↔ `[lib].name` `bonsai_py` ↔ `#[pymodule]` fn `bonsai_py` ↔ `tool.maturin.module-name` `bonsai_py` ↔ `import bonsai_py`. If the underscored form drifts (e.g. `[lib].name = "bonsai_bt"`), the import fails at runtime with `ImportError: dynamic module does not define module export function (PyInit_*)`. Quick check: grep for `bonsai_py` and `bonsai-py` and confirm those are the only two forms in the binding crate.

2. **`python-source` layout is exact.** `python/bonsai_py/__init__.py` — both directory names matter. If you name the inner dir `python/bonsai/`, maturin will install the package as `bonsai`, not `bonsai_py`, and the C extension import inside `__init__.py` will mismatch.

3. **Path dependency without version.** `bonsai-bt = { path = "../bonsai" }` (no `version`) works for `cargo build` but causes `cargo package` (and hence `maturin sdist`) to fail with `cannot package a workspace member with a path dependency without a version`. The dual form solves this.

4. **bonsai-bt not on crates.io yet?** Check `https://crates.io/crates/bonsai-bt`. If version 0.12.0 isn't there, the dual dep still works for local builds but `maturin sdist` will refuse to package because the version isn't resolvable from a registry. **Action:** before publishing any Python sdist (Section 8/9), confirm `cargo search bonsai-bt` shows ≥0.12.0. If it doesn't, run `cargo publish -p bonsai-bt` from the workspace root first. Wheels are unaffected — only sdist needs registry resolution.

5. **abi3 feature drift.** Some PyO3 features (custom `__hash__`, certain magic methods) need conditional `#[cfg(...)]` to compile under abi3. Section 1 won't hit this since the module is empty, but document it: when Section 2 adds `#[pyclass(eq, eq_int)]`, verify it compiles with `abi3-py310` (it should — `eq_int` is abi3-safe).

6. **`f32` feature must NOT be enabled.** Python's `float` is always 64-bit (IEEE-754 double). Compiling bonsai-bt with `f32` and passing `f64` from Python down to a `f32`-typed `dt` would either fail to compile or silently truncate. Keep `features = ["visualize"]` only.

7. **`cargo build` from workspace root pulls Python headers.** Building `bonsai-py` requires the C Python development headers. On Linux CI without them, `cargo build` (no `-p`) will fail. For Section 1's verification this isn't a blocker (we test `-p bonsai-py` explicitly and `-p bonsai-bt`), but Section 8 will need to either (a) configure CI to install Python dev headers before `cargo build`, or (b) avoid workspace-wide `cargo build` in Rust-only CI. Flag this in the section so the implementer mentions it in the PR description.

8. **Windows wheel build needs MSVC.** Not Section 1's job, but document: developers on Windows must have the MSVC build tools installed (`rustup default stable-msvc` and Visual Studio Build Tools). `maturin develop` will fail with a linker error otherwise.

9. **`maturin develop` requires an active venv.** It refuses to install into the system Python. The verification steps below activate one explicitly.

10. **README path resolution.** `readme = "README.md"` in `pyproject.toml` is relative to `pyproject.toml`, not to the workspace root. The placeholder README **must exist** at `bonsai-py/README.md` before `maturin develop` runs, or the build fails before reaching Rust compilation.

11. **`publish = false` on the Cargo package** prevents accidental `cargo publish` from pushing the binding crate to crates.io (where it would never resolve correctly). The Python wheel is the only published artifact.

12. **`Cargo.lock` churn.** Adding `bonsai-py` will materially expand the workspace `Cargo.lock` (pyo3 pulls a sizeable dep graph). Expect the lockfile diff in the PR to be large. This is normal and not a regression.

### Python-usage considerations baked into the scaffold

Even though Section 1 ships no API, several scaffolding choices were made specifically to make later sections' Python ergonomics work:

- **`py.typed` from day one** — once Section 5 lands stubs, IDE autocomplete works in editable installs without any additional config.
- **`python-source = "python"`** — lets us add pure-Python helpers later (e.g. context managers, dataclass adapters) without rebuilding the wheel.
- **`__version__` via `importlib.metadata`** — common Python convention; matches what users expect from `pip show bonsai-py`.
- **`abi3-py310`** — users on 3.10/3.11/3.12/3.13 install the *same* wheel; no per-Python-minor fanout. Saves CI time and disk for users.
- **Module docstring in `lib.rs`** — `help(bonsai_py)` will print something meaningful from day one, which sets expectations for the API doc style in later sections.

### Verification (sequential — each step must pass before continuing)

Run from the repository root unless noted. **Each step is a complete command the implementer can copy-paste.**

1. **Rust crate compiles in isolation:**
   ```bash
   cargo check -p bonsai-py
   ```
   Expect: exit 0, warnings allowed but no errors. First run will download pyo3 and friends — may take 1–2 min.

2. **Workspace still builds end-to-end (core crate unaffected):**
   ```bash
   cargo build -p bonsai-bt
   cargo test -p bonsai-bt
   ```
   Expect: existing tests still pass. If they don't, Section 1 has corrupted workspace settings — investigate before continuing.

3. **Create a fresh venv:**
   ```bash
   python -m venv .venv
   source .venv/bin/activate           # macOS/Linux
   .\.venv\Scripts\Activate.ps1        # Windows PowerShell
   ```

4. **Install maturin:**
   ```bash
   pip install -U "maturin>=1.7,<2.0"
   ```

5. **Build & install the extension into the venv:**
   ```bash
   cd bonsai-py
   maturin develop
   ```
   Expect a line like `📦 Installed bonsai-py-0.12.0`. The compiled artifact name should contain `abi3` — confirm by inspecting `maturin develop -v` output or:
   ```bash
   python -c "import bonsai_py, os; print(os.path.basename(bonsai_py.__file__))"
   ```
   Expect a filename like `bonsai_py.abi3.so` (Linux/macOS) or `bonsai_py.pyd` (Windows — note Windows abi3 wheels embed the tag in the wheel filename, not the `.pyd`).

6. **Import works and version metadata is present:**
   ```bash
   python -c "import bonsai_py; print(bonsai_py.__version__); print(bonsai_py.__doc__)"
   ```
   Expect:
   ```
   0.12.0
   bonsai-py — behavior trees in Python, powered by the Rust bonsai-bt crate.
   ```

7. **Build a release wheel (no install) and verify the filename:**
   ```bash
   maturin build --release
   ls target/wheels/
   ```
   Expect a single file matching `bonsai_py-0.12.0-cp310-abi3-<platform>.whl`. The `cp310-abi3` substring confirms abi3-py310 is active. (PyPI normalizes the distribution name `bonsai-py` to `bonsai_py` in wheel filenames — this is expected.)

If all seven steps pass, Section 1 is complete and Section 2 may begin.

### Acceptance criteria (for the reviewer)

- All files listed in **Files to create** exist with the specified content (auto-verifiable: `git diff main --stat` shows the expected set).
- Root `Cargo.toml` `members` includes `"bonsai-py"`.
- Verification steps 1–7 all pass on a clean checkout.
- The PR description confirms `abi3-py310` resolves cleanly on `pyo3 = "0.28"` (see Open Question #1).
- No changes to `bonsai/` or `examples/` source — Section 1 is additive only. (`git diff main -- bonsai/ examples/` should be empty.)

### Open questions for the implementer (resolve before merging)

1. **`abi3-py310` feature on `pyo3 = "0.28"` — sanity check.** PyO3 has been pruning older `abi3-py3X` features over time. Confirm `0.28` still exposes `abi3-py310` before pinning by running `cargo add pyo3 --features extension-module,abi3-py310` in a scratch directory. If it doesn't resolve, drop one minor version of PyO3 at a time until it does, and note the resolved pair in the PR description.
2. **`bonsai-bt 0.12.0` on crates.io.** Not blocking for Section 1's verification (which uses path), but the implementer should confirm `cargo search bonsai-bt` returns a `0.12.x` entry. If not, file a follow-up ticket on Section 9 to publish bonsai-bt first.

### Handoff checklist

The implementer should treat this section as complete only after:

- [ ] All 6 files created with content matching this spec.
- [ ] Root `Cargo.toml` updated.
- [ ] Verification steps 1–7 captured in the PR description (screenshot or pasted output).
- [ ] Open questions 1–2 above resolved or filed as follow-up tickets.
- [ ] CI passes on the PR (note: existing `rust-ci.yml` may need a small tweak from item #7 in pitfalls — coordinate with the user if it fails).

---

## Section 2 — `PyStatus` and `PyActionArgs`

**Goal.** Wrap the two leaf value types that the user-facing API exposes.

**Depends on:** Section 1.

**Files to create:**
- [bonsai-py/src/status.rs](bonsai-py/src/status.rs)
- [bonsai-py/src/action_args.rs](bonsai-py/src/action_args.rs)

**Files to modify:**
- [bonsai-py/src/lib.rs](bonsai-py/src/lib.rs) — add `mod status; mod action_args;` and register `PyStatus`, `PyActionArgs` on the module.

**Key references:**
- Rust `Status` enum: [bonsai/src/status.rs](bonsai/src/status.rs).
- Rust `ActionArgs<'_, E, A>`: [bonsai/src/state.rs:18-27](bonsai/src/state.rs#L18-L27).

**Implementation notes:**
- `PyStatus` is a `#[pyclass(eq, eq_int, name = "Status")]` enum with three variants: `Success`, `Failure`, `Running`. Add `From<Status>` + `From<PyStatus>` conversions in this file.
- `PyActionArgs` is `#[pyclass(name = "ActionArgs")]` with two `#[pyo3(get)]` fields: `dt: f64`, `action: PyObject`. **No `event` field** (resolved decision).
- Provide a Rust-side helper `PyActionArgs::from_rust(args: &ActionArgs<Event, PyObject>, py: Python) -> Self` that clones the action `PyObject` for the Python user. This is what the tick bridge in Section 4 will call.

**Acceptance criteria:**
- `cargo build -p bonsai-py` passes.
- `maturin develop` succeeds; from Python: `bonsai_py.Status.Success == bonsai_py.Status.Success` and `bonsai_py.Status.Success != bonsai_py.Status.Failure`.
- An ad-hoc Rust unit test constructs a `PyActionArgs` and reads `dt`, `action`.

---

## Section 3 — `PyBehavior` and factory functions

**Goal.** Let Python construct a `Behavior<PyObject>` tree.

**Depends on:** Section 1.

**Files to create:**
- [bonsai-py/src/behavior.rs](bonsai-py/src/behavior.rs).

**Files to modify:**
- [bonsai-py/src/lib.rs](bonsai-py/src/lib.rs) — register `PyBehavior` and add `m.add_function(...)` for each factory.

**Key references:**
- Rust `Behavior<A>` enum (14 variants): [bonsai/src/behavior.rs:12](bonsai/src/behavior.rs#L12).
- The exact factory list is in [bonsai/src/lib.rs:121-124](bonsai/src/lib.rs#L121-L124).

**Implementation notes:**
- `PyBehavior` is opaque to Python: `#[pyclass(name = "Behavior")] struct PyBehavior { inner: Behavior<PyObject> }`. No fields exposed. (Power users can introspect via repr in a later iteration.)
- Implement 14 `#[pyfunction]` factories that build a `Behavior<PyObject>` and wrap it in `PyBehavior`:
  - **Leaves:** `Action(action: PyObject) -> PyBehavior`, `Wait(seconds: f64)`, `WaitForever()`
  - **Decorators:** `Invert(child: PyBehavior)`, `AlwaysSucceed(child: PyBehavior)`
  - **Composites:** `Sequence(children: Vec<PyBehavior>)`, `Select(children: Vec<PyBehavior>)`, `WhenAll(...)`, `WhenAny(...)`, `After(...)`, `Race(...)`
  - **Control flow:** `If(cond: PyBehavior, ok: PyBehavior, ko: PyBehavior)`, `While(cond: PyBehavior, body: Vec<PyBehavior>)`, `WhileAll(cond: PyBehavior, body: Vec<PyBehavior>)`
- Factories take `PyBehavior` by value (the user's variable is consumed). Internally we move `inner` out.
- Add `__repr__` on `PyBehavior` that prints the variant name + arity. Helps debugging.
- Watch the `While` / `WhileAll` panic-on-empty-body documented in [bonsai/src/behavior.rs:54-55](bonsai/src/behavior.rs#L54-L55): raise `ValueError` from Python *before* calling the Rust constructor. Don't let a Rust panic cross the FFI boundary.

**Acceptance criteria:**
- All 14 factories importable from Python.
- Construct the full tree from [examples/src/visualizer_smoke/main.rs:75-94](examples/src/visualizer_smoke/main.rs#L75-L94) from Python without panics.
- `bt.While(bt.Action("x"), [])` raises `ValueError`.

---

## Section 4 — `PyBT` and the callback bridge

**Goal.** The hot path: `BT(tree, blackboard).tick(dt, callback)` works end-to-end.

**Depends on:** Sections 2 and 3.

**Files to create:**
- [bonsai-py/src/bt.rs](bonsai-py/src/bt.rs).

**Files to modify:**
- [bonsai-py/src/lib.rs](bonsai-py/src/lib.rs) — register `PyBT`.

**Key references:**
- Rust `BT` struct + `tick`: [bonsai/src/bt.rs](bonsai/src/bt.rs).
- `with_telemetry`: [bonsai/src/bt_telemetry.rs:94](bonsai/src/bt_telemetry.rs#L94).
- Doctest example (target behavior to match): [bonsai/src/lib.rs:39-118](bonsai/src/lib.rs#L39-L118).

**API to expose** (all `#[pymethods]`):
- `__init__(behavior: PyBehavior, blackboard: PyObject)`
- `tick(dt: float, callback: Callable[[ActionArgs, Any], tuple[Status, float]]) -> Optional[tuple[Status, float]]`
- `blackboard() -> PyObject` (live reference — mutations to it from Python persist into the BT)
- `reset_bt() -> None`
- `tick_count() -> int`
- `is_finished() -> bool`
- `with_telemetry(port: int, host: str = "127.0.0.1") -> PyBT` (chainable; raises `OSError` if bind fails)

**The tick bridge.** This is the only non-trivial code in the binding. Sketch:

```rust
#[pymethods]
impl PyBT {
    fn tick(&mut self, py: Python<'_>, dt: f64, callback: PyObject)
        -> PyResult<Option<(PyStatus, f64)>>
    {
        let event: Event = UpdateArgs { dt }.into();
        let mut cb_err: Option<PyErr> = None;
        let result = self.inner.tick(&event, &mut |args, bb: &mut PyObject| {
            if cb_err.is_some() { return (Status::Failure, 0.0); } // short-circuit after error
            let py_args = PyActionArgs::from_rust(&args, py);
            match callback.call1(py, (py_args, bb.clone_ref(py))) {
                Ok(ret) => match ret.extract::<(PyStatus, f64)>(py) {
                    Ok((s, dt)) => (s.into(), dt),
                    Err(e) => { cb_err = Some(e); (Status::Failure, 0.0) }
                },
                Err(e) => { cb_err = Some(e); (Status::Failure, 0.0) }
            }
        });
        if let Some(e) = cb_err { return Err(e); }
        Ok(result.map(|(s, dt)| (s.into(), dt)))
    }
}
```

**Implementation notes:**
- **GIL strategy:** hold the GIL for the whole tick. The closure runs synchronously and is always Python-bound, so `allow_threads` + `with_gil` adds overhead without benefit. (We can revisit if a future async path appears.)
- **Error propagation:** Python exceptions from the callback can't unwind through Rust. Capture in `cb_err` and propagate. Short-circuit subsequent action calls in the same tick to avoid spurious sibling callbacks after an error.
- **Blackboard.** `inner.bb` is the actual `PyObject` — `blackboard()` returns `self.inner.blackboard().clone_ref(py)`. Same Python object, refcount bumped.
- **`with_telemetry`:** wrap the Rust `io::Result`. Use `pyo3::exceptions::PyOSError` for bind failures. Returning `Self` from a `#[pymethods]` consuming method requires careful PyO3 ownership — easier to take `&mut self`, call `with_telemetry_at` semantically (port/host stored, server started), and return a clone or `Py<Self>`. Look at [bonsai/src/bt_telemetry.rs:162](bonsai/src/bt_telemetry.rs#L162) — the underlying call takes `mut self` by value. Workaround: pop `inner` into a temporary, run `with_telemetry_at`, put it back. Or refactor to use `&mut self`. Document the approach in code.

**Acceptance criteria:**
- Python can run the doctest equivalent and observe `count == 1` after 5 ticks of 0.5s (matching [bonsai/src/lib.rs:99-113](bonsai/src/lib.rs#L99-L113)).
- A callback that `raise ValueError("boom")` causes `bt.tick()` to raise `ValueError("boom")` with traceback intact.
- `bt.with_telemetry(8910)` returns the same `BT` (chainable) and starts a listening socket on `127.0.0.1:8910`.
- `bt.with_telemetry(8910)` while the port is already bound raises `OSError`.

---

## Section 5 — Type stubs

**Goal.** Editor autocomplete + `mypy`-friendly types.

**Depends on:** Section 4 (final method signatures must be locked in).

### Approach decision (evaluate before writing code)

Two viable approaches — **the implementer should pick one at the start of Section 5 and document the choice in the PR description.**

**Option A — Hand-written stubs (default if B fails compatibility check).**
Write `bonsai-py/stubs/bonsai_py.pyi` by hand, mirroring the API from Section 4. Simple, no extra build-time dep, full control over the wording — but the stub file and the Rust signatures drift independently, so every API change in Sections 2–4 requires a parallel `.pyi` edit. Maintenance cost compounds.

**Option B — Auto-generate stubs via `pyo3-stub-gen` (preferred if compatible).**
[`pyo3-stub-gen`](https://crates.io/crates/pyo3-stub-gen) (`0.22.2` on crates.io as of 2026-05-15) reads `#[pyo3(...)]` annotations on `#[pyclass]`/`#[pyfunction]` items and emits a `.pyi` file. The stub generation runs as a small binary built into the crate and invoked by `cargo run --bin stub_gen` (typically wired into CI). Trade-offs:
- ✅ Stubs stay in lockstep with the Rust source automatically.
- ✅ Annotations live next to the functions they document.
- ⚠ Adds a build-time dep (`pyo3-stub-gen` + `pyo3-stub-gen-derive`).
- ⚠ **Compatibility check required:** `pyo3-stub-gen 0.22.x` was published before PyO3 `0.28`; the implementer must verify `pyo3-stub-gen` supports PyO3 `0.28` (check the crate's README and Cargo.toml's `pyo3 = "..."` line). If it lags, either wait for an update or fall back to Option A.

**Recommendation:** start by running `cargo info pyo3-stub-gen` and inspecting its current PyO3 compatibility. If it supports `0.28` (or a newer matching version exists), use Option B. Otherwise Option A.

### Files to create

- Option A: [bonsai-py/stubs/bonsai_py.pyi](bonsai-py/stubs/bonsai_py.pyi) (hand-written).
- Option B: [bonsai-py/src/bin/stub_gen.rs](bonsai-py/src/bin/stub_gen.rs) (small binary that calls the generator) + add `#[gen_stub_pyclass]` / `#[gen_stub_pyfunction]` macros throughout the existing Rust files from Sections 2–4. The generated `bonsai_py.pyi` lives at the same path under `stubs/` (or wherever maturin's `include` glob picks it up).

### Implementation notes (apply to either option)

- Use `Any` for `action` and the blackboard. Use `Callable[[ActionArgs, Any], tuple[Status, float]]` for the tick callback.
- `Status` is an `IntEnum`-like class — expose `Success`, `Failure`, `Running` as class attributes.
- Factory functions are stubbed at module level: `def Sequence(children: list[Behavior]) -> Behavior: ...` etc.
- In `bonsai-py/pyproject.toml` `[tool.maturin]` add an `include` glob so the `.pyi` and `py.typed` ship in the wheel.
- If Option B is chosen, wire `cargo run --bin stub_gen` into CI (Section 8) so generated stubs are regenerated and diffed on every PR — drift between annotations and committed `.pyi` should fail the build.

### Acceptance criteria

- `mypy --strict` on a small example script that imports `bonsai_py` passes with no errors.
- VS Code/PyCharm shows autocomplete on `bt.Status.` and `bt.BT(...).`.
- The PR description states which option was chosen and (for Option B) the exact `pyo3-stub-gen` version pinned alongside the PyO3 0.28 compatibility result.

---

## Section 6 — Test suite

**Goal.** Behavioral parity with the Rust test suite.

**Depends on:** Section 4.

**Files to create:**
- [bonsai-py/tests/test_bt.py](bonsai-py/tests/test_bt.py).
- [bonsai-py/tests/conftest.py](bonsai-py/tests/conftest.py) (optional, fixtures).

**Tests to write** (one pytest each):
1. **Doctest port** — translate [bonsai/src/lib.rs:39-118](bonsai/src/lib.rs#L39-L118) literally; assert counter values at each step. Covers `Sequence`, `Wait`, `Action`, blackboard mutation, `reset_bt`, `tick_count`, `is_finished`.
2. **Variant coverage** — one test per non-trivial variant. Mirror cases in [bonsai/tests/bt_tests.rs](bonsai/tests/bt_tests.rs):
   - `Select` first-success short-circuit
   - `If` true/false branches
   - `Invert` flips Success↔Failure
   - `AlwaysSucceed` swallows Failure
   - `While` repeats until cond fails
   - `WhileAll` checks cond only between iterations
   - `WhenAll` AND-parallel, `WhenAny` OR-parallel
   - `Race` first-to-complete (success OR failure)
   - `After` sequential success in parallel
3. **Callback error propagation** — callback raises `ValueError`; assert `tick()` propagates it.
4. **Empty `While` body validation** — `bt.While(bt.Action("x"), [])` raises `ValueError`.
5. **`with_telemetry` bind failure** — bind port 8910 twice; second call raises `OSError`.

**Acceptance criteria:**
- `cd bonsai-py && maturin develop && pytest -v` is green.
- Tests run in <5s total.

---

## Section 7 — Visualizer smoke example

**Goal.** Confirm the WebSocket visualizer works through Python end-to-end.

**Depends on:** Section 4.

**Files to create:**
- [bonsai-py/examples/visualizer_smoke.py](bonsai-py/examples/visualizer_smoke.py).

**Key references:**
- Rust original: [examples/src/visualizer_smoke/main.rs](examples/src/visualizer_smoke/main.rs).

**Implementation notes:**
- Direct line-for-line port. Same 27-node tree, same phase-offset color rotation, same 400 ms wall sleep.
- The Rust example uses string slices for actions; in Python use plain `str` — matches naturally.
- `bt.with_telemetry(8910)` chains: `bt = BT(build_tree(), {}).with_telemetry(8910)`.

**Manual verification protocol** (document in the script's module docstring):
1. `python examples/visualizer_smoke.py`
2. Open <http://127.0.0.1:8910/>
3. Confirm: 27 nodes render, leaves cycle through green/yellow/red every ~400 ms.
4. `Ctrl-C` and restart → browser reconnects within 1 s.

**Acceptance criteria:**
- Script runs without exceptions for at least 30 s.
- All manual checks above pass.

---

## Section 8 — CI wheel build matrix

**Goal.** Multi-platform wheels build on push and on tag.

**Depends on:** Sections 1, 6 (tests must exist to run in CI).

**Files to create:**
- [.github/workflows/python-wheels.yml](.github/workflows/python-wheels.yml).

**Implementation notes:**
- Use [`PyO3/maturin-action`](https://github.com/PyO3/maturin-action) (de-facto standard).
- Matrix:
  - Linux: `manylinux_2_17_x86_64`, `manylinux_2_17_aarch64` (cross-build via docker)
  - macOS: `universal2` (single wheel covers x86_64 + arm64)
  - Windows: `x86_64`
- One abi3 wheel per platform covers Python 3.8+ — don't fan out across Python minor versions.
- Triggers: push to `main` (test build, don't publish), tag matching `py-v*` (publish to PyPI).
- Add a step that runs `pytest` on the built wheel inside the matrix (install wheel into a fresh venv, then test).
- PyPI publish step uses `pypa/gh-action-pypi-publish` with `PYPI_API_TOKEN` secret. **Recommend a TestPyPI dry-run job triggered on `py-test-*` tags first.**

**Acceptance criteria:**
- A push to a feature branch produces green builds for all three platforms.
- Wheels are uploaded as GitHub Actions artifacts.
- A test tag pushes a wheel to TestPyPI; `pip install -i https://test.pypi.org/simple/ bonsai-py` works in a fresh venv.

---

## Section 9 — Docs & release process

**Goal.** Users can find and install the package.

**Depends on:** Sections 4, 7, 8.

**Files to modify:**
- [bonsai-py/README.md](bonsai-py/README.md) — quickstart, install, link to visualizer example, link back to the core Rust crate.
- [README.md](README.md) (root) — short Python section pointing at the wheel: `pip install bonsai-py`.

**Files to create:**
- [bonsai-py/CHANGELOG.md](bonsai-py/CHANGELOG.md) — initial entry: `0.1.0 — first PyO3 binding, parity with bonsai-bt 0.12.0`.

**Versioning policy** (document in `bonsai-py/README.md`):
- Python package version tracks the Rust crate version it wraps (e.g. Rust `bonsai-bt 0.12.0` → `bonsai-py 0.12.0` on PyPI).
- Hot-fix releases on the Python side only bump a `postN` suffix (PEP 440), e.g. `0.12.0.post1`.

**Acceptance criteria:**
- README renders correctly on PyPI (no broken Markdown after `maturin upload`).
- A tagged release pushes both a sdist and wheels for all platforms.

---

## Integration verification (final, after all sections)

Run once everything is in place:

1. Fresh venv: `python -m venv .venv && source .venv/bin/activate`
2. `pip install bonsai-py` (from PyPI, not the local path).
3. Run [bonsai-py/examples/visualizer_smoke.py](bonsai-py/examples/visualizer_smoke.py) and walk through Section 7's manual checks.
4. `pytest bonsai-py/tests/` against the installed wheel.
5. Verify `import bonsai_py; help(bonsai_py.BT)` shows the documented surface.

If all five pass, the binding is ready to announce.
