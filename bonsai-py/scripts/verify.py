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

    assert bt.Behavior.__module__ == "bonsai_py", bt.Behavior.__module__
    print("  OK: Behavior.__module__ == 'bonsai_py'")

    assert repr(bt.Action(None)) == "Action(...)"
    print("  OK: Action(None) is a legal leaf")

    # Identity-based equality holds for every variant. Two distinct
    # constructions of the same shape must compare as distinct objects.
    builders = [
        ("Action",        lambda: bt.Action("x")),
        ("Wait",          lambda: bt.Wait(1.0)),
        ("WaitForever",   lambda: bt.WaitForever()),
        ("Invert",        lambda: bt.Invert(bt.Action("x"))),
        ("AlwaysSucceed", lambda: bt.AlwaysSucceed(bt.Action("x"))),
        ("Sequence",      lambda: bt.Sequence([bt.Action("x")])),
        ("Select",        lambda: bt.Select([bt.Action("x")])),
        ("WhenAll",       lambda: bt.WhenAll([bt.Action("x")])),
        ("WhenAny",       lambda: bt.WhenAny([bt.Action("x")])),
        ("After",         lambda: bt.After([bt.Action("x")])),
        ("Race",          lambda: bt.Race([bt.Action("x")])),
        ("If",            lambda: bt.If(bt.Action("c"), bt.Action("s"), bt.Action("f"))),
        ("While",         lambda: bt.While(bt.Action("c"), [bt.Action("b")])),
        ("WhileAll",      lambda: bt.WhileAll(bt.Action("c"), [bt.Action("b")])),
    ]
    for label, make in builders:
        a, b = make(), make()
        assert a is not b, f"{label}: distinct constructions returned same object"
        assert (a == b) is False, f"{label}: structural == returned True"
        assert a != b, f"{label}: != returned False"
    print(f"  OK: identity-based equality across all {len(builders)} variants")


def test_bt_tick_and_telemetry() -> None:
    """BT class: tick callback semantics, blackboard, reset, telemetry."""
    import socket

    import bonsai_py as bt

    # Doctest equivalent: 5 ticks of 0.5s, expect count == 1.
    tree = bt.Sequence([
        bt.Wait(1.0), bt.Action("inc"),
        bt.Wait(1.0), bt.Action("inc"),
        bt.Wait(0.5), bt.Action("dec"),
    ])
    bb = {"count": 0}
    tree_bt = bt.BT(tree, bb)
    acc = 0

    def cb(args, _blackboard):
        nonlocal acc
        if args.action == "inc":
            acc += 1
            return (bt.Status.Success, args.dt)
        if args.action == "dec":
            acc -= 1
            return (bt.Status.Success, args.dt)
        return bt.RUNNING

    for _ in range(5):
        tree_bt.tick(0.5, cb)
    blackboard = tree_bt.blackboard()
    blackboard["count"] = acc
    assert bb["count"] == 1, bb["count"]
    assert tree_bt.tick_count() == 5
    print("  OK: doctest equivalent — count == 1 after 5 ticks")

    assert bt.RUNNING == (bt.Status.Running, 0.0)
    print("  OK: bt.RUNNING == (Status.Running, 0.0)")

    # Callback exception propagates with traceback / message intact.
    bad_bt = bt.BT(bt.Sequence([bt.Action("boom")]), None)

    def raising_cb(_args, _bb):
        raise ValueError("boom")

    try:
        bad_bt.tick(0.5, raising_cb)
    except ValueError as e:
        assert str(e) == "boom", str(e)
        print("  OK: callback ValueError propagates with message intact")
    else:
        raise AssertionError("expected ValueError from tick")

    # Callback returning a non-tuple is rejected by extract.
    bad_shape = bt.BT(bt.Sequence([bt.Action("x")]), None)

    def bad_return_cb(_args, _bb):
        return "not a tuple"

    try:
        bad_shape.tick(0.5, bad_return_cb)
    except Exception as e:
        msg = str(e).lower()
        assert "extract" in msg or "tuple" in msg or isinstance(e, TypeError), e
        print(f"  OK: bad return shape rejected ({type(e).__name__})")
    else:
        raise AssertionError("expected error from malformed return")

    # is_finished + reset_bt cycle.
    quick = bt.BT(bt.Action("done"), None)

    def done_cb(_args, _bb):
        return (bt.Status.Success, 0.0)

    assert not quick.is_finished()
    quick.tick(0.0, done_cb)
    assert quick.is_finished()
    assert quick.tick(0.0, done_cb) is None
    quick.reset_bt()
    assert not quick.is_finished()
    print("  OK: is_finished + reset_bt cycle")

    # with_telemetry: chainable + bind-failure raises OSError.
    with socket.socket() as s:
        s.bind(("127.0.0.1", 0))
        port = s.getsockname()[1]
    chained = bt.BT(bt.Action("x"), None).with_telemetry(port)
    assert chained is not None, "with_telemetry should return self for chaining"
    print(f"  OK: with_telemetry({port}) chainable")

    try:
        bt.BT(bt.Action("x"), None).with_telemetry(port)
    except OSError as e:
        print(f"  OK: with_telemetry on bound port raises OSError ({type(e).__name__})")
    else:
        raise AssertionError("expected OSError on already-bound port")

    assert bt.BT.__module__ == "bonsai_py", bt.BT.__module__
    print("  OK: BT.__module__ == 'bonsai_py'")

    bb_in = {"x": 1}
    identity_bt = bt.BT(bt.Action("noop"), bb_in)
    assert identity_bt.blackboard() is bb_in
    print("  OK: blackboard() preserves Python object identity")

    sentinel_action = {"k": 42}
    sentinel_bb = {"y": 1}
    seen = []

    def identity_cb(args, bbref):
        seen.append((args.action is sentinel_action, bbref is sentinel_bb))
        return (bt.Status.Success, 0.0)

    bt.BT(bt.Action(sentinel_action), sentinel_bb).tick(0.0, identity_cb)
    assert seen == [(True, True)], seen
    print("  OK: action + bb identity preserved through tick callback")

    # WhenAll short-circuit: callback raise on child[1] must NOT invoke child[2].
    order = []

    def short_circuit_cb(args, _bb):
        order.append(args.action)
        if args.action == "b":
            raise ValueError("stop")
        return (bt.Status.Success, 0.0)

    parallel = bt.BT(
        bt.WhenAll([bt.Action("a"), bt.Action("b"), bt.Action("c")]),
        None,
    )
    try:
        parallel.tick(0.0, short_circuit_cb)
    except ValueError:
        pass
    assert order == ["a", "b"], f"expected ['a','b'] (short-circuit), got {order}"
    print("  OK: WhenAll short-circuits sibling callbacks after a raise")

    # Poisoned BT (after with_telemetry bind failure) refuses subsequent calls.
    with socket.socket() as s:
        s.bind(("127.0.0.1", 0))
        port2 = s.getsockname()[1]
    _holder = bt.BT(bt.Action("x"), None).with_telemetry(port2)  # keep alive
    victim = bt.BT(bt.Action("y"), None)
    try:
        victim.with_telemetry(port2)
    except OSError:
        pass
    try:
        victim.tick(0.0, lambda _a, _b: (bt.Status.Success, 0.0))
    except RuntimeError as e:
        assert "invalidated" in str(e), str(e)
        print("  OK: poisoned BT raises RuntimeError on subsequent tick")
    else:
        raise AssertionError("expected RuntimeError on poisoned BT")


def test_type_stubs() -> None:
    """Type stub is present, valid syntax, declares every public symbol."""
    import ast
    from pathlib import Path

    import bonsai_py

    stub_path = Path(bonsai_py.__file__).parent / "__init__.pyi"
    assert stub_path.exists(), f"missing stub: {stub_path}"
    print(f"  OK: stub present at {stub_path.name}")

    tree = ast.parse(stub_path.read_text())
    print("  OK: stub parses as valid Python syntax")

    declared: set[str] = set()
    for node in ast.walk(tree):
        if isinstance(node, (ast.ClassDef, ast.FunctionDef)):
            declared.add(node.name)
        elif isinstance(node, ast.AnnAssign) and isinstance(node.target, ast.Name):
            declared.add(node.target.id)

    expected = set(bonsai_py.__all__)
    missing = expected - declared
    assert not missing, f"stub missing names: {sorted(missing)}"
    print(f"  OK: all {len(expected)} __all__ names present in stub")

    # The stub-side __all__ must agree with the package __all__ on the
    # full public surface. Drift between the two is the most common cause
    # of "mypy sees Status but `from bonsai_py import *` doesn't".
    stub_all: set[str] = set()
    for node in ast.walk(tree):
        if isinstance(node, ast.Assign) and any(
            isinstance(t, ast.Name) and t.id == "__all__" for t in node.targets
        ):
            if isinstance(node.value, ast.List):
                stub_all = {
                    elt.value for elt in node.value.elts if isinstance(elt, ast.Constant)
                }
    if stub_all:
        diff = expected.symmetric_difference(stub_all)
        assert not diff, f"package and stub __all__ disagree: {sorted(diff)}"
        print("  OK: package and stub __all__ agree")


# ---------------------------------------------------------------------------
# Registry — append new test functions above and add them to this list.
# Do not reorder or remove existing entries.
# ---------------------------------------------------------------------------
TESTS = [
    test_module_scaffolding,
    test_status_and_action_args,
    test_behavior_factories,
    test_bt_tick_and_telemetry,
    test_type_stubs,
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
