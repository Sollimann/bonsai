//! Stub-generator binary. Builds the `.pyi` from the `#[gen_stub_*]`
//! annotations sprinkled across the binding crate.
//!
//! Run with `cargo run --bin stub_gen -p bonsai-py`. pyo3-stub-gen reads
//! `pyproject.toml` to determine the package layout and writes the stub
//! to `python/bonsai_bt/__init__.pyi`. The companion `scripts/regen-stubs.sh`
//! appends the manual `RUNNING` constant declaration afterwards.

use pyo3_stub_gen::Result;

fn main() -> Result<()> {
    let stub = bonsai_py::stub_info()?;
    stub.generate()?;
    Ok(())
}
