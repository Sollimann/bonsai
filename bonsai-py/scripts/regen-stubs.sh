#!/usr/bin/env bash
# Regenerate the type stub and apply manual touch-ups (RUNNING constant).
# Run after editing any #[gen_stub_*] annotation.
#
# Usage: ./bonsai-py/scripts/regen-stubs.sh
set -euo pipefail

cd "$(dirname "$0")/.."   # cd into bonsai-py/
cargo run --quiet --bin stub_gen -p bonsai-py

# pyo3-stub-gen 0.22 reads pyproject.toml and writes the stub to
# python/bonsai_py/__init__.pyi automatically.
STUB=python/bonsai_py/__init__.pyi
if [ ! -f "$STUB" ]; then
    echo "ERROR: $STUB was not generated. Did the binary fail silently?" >&2
    exit 1
fi

# pyo3-stub-gen doesn't introspect m.add() module-level constants, so
# the RUNNING declaration must be appended manually (idempotent).
if ! grep -q "^RUNNING: " "$STUB"; then
    {
        echo ""
        echo "RUNNING: typing.Final[tuple[Status, builtins.float]]"
        echo 'r"""Convenience constant: ``(Status.Running, 0.0)`` - return from a tick callback to keep the action running."""'
    } >> "$STUB"
fi

# Add RUNNING to __all__ if not already present.
if ! grep -q '"RUNNING"' "$STUB"; then
    # Insert before the closing ']' of the __all__ list.
    sed -i '/^__all__ = \[/,/^]/{/^]/i\    "RUNNING",
}' "$STUB"
fi

# Strip trailing whitespace defensively.
sed -i 's/[[:space:]]*$//' "$STUB"
echo "Regenerated $STUB"
