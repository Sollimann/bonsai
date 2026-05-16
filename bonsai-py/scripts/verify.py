r"""
Verification script for bonsai-py.

WHAT THIS IS
    Python-side assertions for the installed `bonsai_py` extension module.
    Every test runs on every invocation; the script exits 0 on full pass,
    1 on the first failure, 2 on a usage error.

USAGE
    From the repository root, with a venv that has the wheel installed:

        source .venv/bin/activate          # macOS / Linux / WSL
        .\.venv\Scripts\Activate.ps1       # Windows PowerShell

    Then:

        python bonsai-py/scripts/verify.py

PREREQUISITES
    cargo check -p bonsai-py
    cd bonsai-py && maturin develop

    `maturin develop` produces the abi3 wheel and installs it editable into
    the active venv. This script imports the installed `bonsai_py` module —
    if it fails with `ImportError`, you forgot to `maturin develop`.

WHAT THIS IS NOT
    Not a pytest suite. Long-term behavior coverage lives under
    `bonsai-py/tests/` and is run via pytest separately.
"""

from __future__ import annotations

import sys


def test_module_scaffolding() -> None:
    """Module imports cleanly; version metadata and docstring present."""
    import bonsai_py

    assert bonsai_py.__version__ == "0.12.0", bonsai_py.__version__
    print("  OK: __version__ == 0.12.0")

    assert bonsai_py.__doc__, "missing module docstring"
    print("  OK: module docstring present")


def test_status_and_action_args() -> None:
    """Status enum + ActionArgs class: semantics, pickle, copy, identity."""
    import copy
    import pickle

    import bonsai_py as bt

    assert hasattr(bt, "Status") and hasattr(bt, "ActionArgs")
    print("  OK: Status and ActionArgs exported")

    s = bt.Status
    assert s.Success == s.Success
    assert s.Success != s.Failure
    assert s.Success == 0 and s.Failure == 1 and s.Running == 2
    assert int(s.Success) == 0
    assert hash(s.Success) == hash(s.Success)
    assert {s.Success: "ok"}[s.Success] == "ok"
    assert s.Success is s.Success
    assert repr(s.Success) == "Status.Success"
    print("  OK: Status semantics (eq, eq_int, hash, identity, repr)")

    assert bt.Status.__module__ == "bonsai_py", bt.Status.__module__
    assert bt.ActionArgs.__module__ == "bonsai_py", bt.ActionArgs.__module__
    print("  OK: __module__ == 'bonsai_py' on both classes")

    for v in (s.Success, s.Failure, s.Running):
        assert pickle.loads(pickle.dumps(v)) is v
    for v in (s.Success, s.Failure, s.Running):
        assert copy.copy(v) is v
        assert copy.deepcopy(v) is v
    print("  OK: pickle / copy.copy / copy.deepcopy preserve Status singleton identity")

    a = bt.ActionArgs(0.5, "inc")
    assert a.dt == 0.5
    assert a.action == "inc"
    assert repr(a) == "ActionArgs(dt=0.5, action='inc')"
    try:
        a.dt = 0.7
    except AttributeError:
        pass
    else:
        raise AssertionError("ActionArgs should be frozen (got mutation)")
    print("  OK: ActionArgs construct + readonly + repr")

    bb = {"k": 1}
    a2 = bt.ActionArgs(0.0, bb)
    assert a2.action is bb, "action identity not preserved through FFI"
    a2.action["k"] = 2
    assert bb["k"] == 2, "mutation via args.action did not persist"
    print("  OK: action identity preserved across FFI boundary")


def test_behavior_factories() -> None:
    """14 tree-construction factories: exported, build trees, validate inputs."""
    import bonsai_py as bt

    factories = (
        "Action", "Wait", "WaitForever",
        "Invert", "AlwaysSucceed",
        "Sequence", "Select", "WhenAll", "WhenAny", "After", "Race",
        "If", "While", "WhileAll",
    )
    for name in factories:
        assert hasattr(bt, name), f"missing factory: {name}"
        assert callable(getattr(bt, name)), f"not callable: {name}"
    print(f"  OK: all {len(factories)} factories exported")

    # 27-node tree mirroring the Rust visualizer-smoke example.
    tree = bt.Sequence([
        bt.If(
            bt.Action("low_hp"),
            bt.AlwaysSucceed(bt.Action("flee")),
            bt.Action("regroup"),
        ),
        bt.Select([
            bt.Sequence([
                bt.Action("acquire_target"),
                bt.WhenAll([bt.Action("aim"), bt.Action("track")]),
            ]),
            bt.Race([bt.Action("dodge"), bt.Wait(2.0)]),
            bt.Invert(bt.Action("enemy_visible")),
        ]),
        bt.While(bt.Action("has_ammo"), [bt.Action("fire"), bt.Wait(0.3)]),
        bt.After([bt.Action("cooldown"), bt.Action("ready_signal")]),
        bt.WhenAny([bt.Action("victory_check"), bt.Action("retreat_signal")]),
    ])
    assert repr(tree) == "Sequence(5)"
    print("  OK: 27-node nested tree constructs (5 top-level children)")

    wait = bt.Wait(1.0)
    reuse = bt.Sequence([wait, wait, wait])
    assert repr(reuse) == "Sequence(3)"
    assert repr(wait) == "Wait(1)"
    print("  OK: subtree reuse (same Behavior referenced 3x)")

    for label, call in (
        ("While(cond, [])", lambda: bt.While(bt.Action("x"), [])),
        ("WhileAll(cond, [])", lambda: bt.WhileAll(bt.Action("x"), [])),
        ("Wait(NaN)", lambda: bt.Wait(float("nan"))),
    ):
        try:
            call()
        except ValueError:
            pass
        else:
            raise AssertionError(f"{label} should raise ValueError")
    print("  OK: validation guards (empty While/WhileAll body, NaN Wait)")

    # Pass-through cases — must NOT raise.
    bt.Sequence([]); bt.Select([])
    bt.WhenAll([]); bt.WhenAny([]); bt.After([]); bt.Race([])
    bt.Wait(-1.0); bt.Wait(float("inf")); bt.Wait(1); bt.Wait(0.0)
    print("  OK: empty composites + negative/inf/int/zero Wait pass through")

    tree2 = bt.If(
        cond=bt.Action("c"),
        on_success=bt.Action("s"),
        on_failure=bt.Action("f"),
    )
    assert repr(tree2) == "If(...)"
    print("  OK: keyword arguments accepted on If")

    assert repr(bt.Action({"k": 1})) == "Action(...)"
    assert repr(bt.Invert(bt.Action("x"))) == "Invert(...)"
    assert repr(bt.WaitForever()) == "WaitForever"
    print("  OK: __repr__ is one-line bounded summary")


# ---------------------------------------------------------------------------
# Registry — append new test functions above and add them to this list.
# Do not reorder or remove existing entries.
# ---------------------------------------------------------------------------
TESTS = [
    test_module_scaffolding,
    test_status_and_action_args,
    test_behavior_factories,
]


def main() -> int:
    if len(sys.argv) > 1:
        print(
            f"usage: {sys.argv[0]}    (no arguments; runs every test)",
            file=sys.stderr,
        )
        return 2

    for fn in TESTS:
        print(f"=== {fn.__name__} ===")
        try:
            fn()
        except AssertionError as e:
            print(f"  FAIL: {e}", file=sys.stderr)
            return 1
        except ImportError as e:
            print(
                f"  FAIL: import error ({e}) — did you run `maturin develop`?",
                file=sys.stderr,
            )
            return 1
        except Exception as e:  # noqa: BLE001 — surface anything unexpected
            print(f"  FAIL: {type(e).__name__}: {e}", file=sys.stderr)
            return 1

    print(f"\nAll {len(TESTS)} tests passed.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
