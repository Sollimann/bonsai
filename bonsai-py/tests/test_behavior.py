"""Behavior class + 14 factory functions + ports of Rust behavior_tests."""
from __future__ import annotations

from typing import Any, Callable

import pytest

import bonsai_bt as bt

FACTORY_NAMES = (
    "Action", "Wait", "WaitForever",
    "Invert", "AlwaysSucceed",
    "Sequence", "Select", "WhenAll", "WhenAny", "After", "Race",
    "If", "While", "WhileAll",
)


class TestFactoriesPresent:
    @pytest.mark.parametrize("name", FACTORY_NAMES)
    def test_factory_exported(self, name: str) -> None:
        """Each of the 14 factory names is importable and callable."""
        assert hasattr(bt, name), f"missing factory {name}"
        assert callable(getattr(bt, name)), f"{name} not callable"

    def test_factory_count(self) -> None:
        """Exactly 14 factory names tracked — guards against silent additions."""
        assert len(FACTORY_NAMES) == 14


def _trivial(label: str) -> bt.Behavior:
    return bt.Action(label)


class TestFactoryConstruction:
    @pytest.mark.parametrize(
        "build, expected_repr",
        [
            (lambda: bt.Action("x"), "Action(...)"),
            (lambda: bt.Wait(1.0), "Wait(1)"),
            (lambda: bt.WaitForever(), "WaitForever"),
            (lambda: bt.Invert(_trivial("c")), "Invert(...)"),
            (lambda: bt.AlwaysSucceed(_trivial("c")), "AlwaysSucceed(...)"),
            (lambda: bt.Sequence([_trivial("a"), _trivial("b")]), "Sequence(2)"),
            (lambda: bt.Select([_trivial("a")]), "Select(1)"),
            (lambda: bt.WhenAll([_trivial("a")]), "WhenAll(1)"),
            (lambda: bt.WhenAny([_trivial("a"), _trivial("b")]), "WhenAny(2)"),
            (lambda: bt.After([_trivial("a")]), "After(1)"),
            (lambda: bt.Race([_trivial("a"), _trivial("b")]), "Race(2)"),
            (lambda: bt.If(_trivial("c"), _trivial("s"), _trivial("f")), "If(...)"),
            (lambda: bt.While(_trivial("c"), [_trivial("b")]), "While(1)"),
            (lambda: bt.WhileAll(_trivial("c"), [_trivial("b")]), "WhileAll(1)"),
        ],
    )
    def test_each_factory_builds(
        self, build: Callable[[], bt.Behavior], expected_repr: str
    ) -> None:
        """Each factory builds a Behavior with the expected bounded repr."""
        node = build()
        assert isinstance(node, bt.Behavior)
        assert repr(node) == expected_repr


class TestValidationGuards:
    def test_while_empty_body_raises(self) -> None:
        """While with empty body raises ValueError (would panic in Rust)."""
        with pytest.raises(ValueError, match="must not be empty"):
            bt.While(_trivial("c"), [])

    def test_whileall_empty_body_raises(self) -> None:
        """WhileAll with empty body raises ValueError (would panic in Rust)."""
        with pytest.raises(ValueError, match="must not be empty"):
            bt.WhileAll(_trivial("c"), [])

    def test_wait_nan_raises(self) -> None:
        """Wait(NaN) raises ValueError at the Python boundary, never reaches Rust."""
        with pytest.raises(ValueError, match="NaN"):
            bt.Wait(float("nan"))

    @pytest.mark.parametrize(
        "build",
        [
            lambda: bt.Sequence([]),
            lambda: bt.Select([]),
            lambda: bt.WhenAll([]),
            lambda: bt.WhenAny([]),
            lambda: bt.After([]),
            lambda: bt.Race([]),
        ],
    )
    def test_other_empty_composites_allowed(
        self, build: Callable[[], bt.Behavior]
    ) -> None:
        """Empty Sequence/Select/etc. are allowed (don't panic in Rust)."""
        node = build()
        assert isinstance(node, bt.Behavior)

    @pytest.mark.parametrize("value", [-1.0, 0.0, float("inf"), 1])
    def test_wait_passthrough_values(self, value: float) -> None:
        """Negative, zero, inf, int -- all accepted at the boundary."""
        assert isinstance(bt.Wait(value), bt.Behavior)


class TestSubtreeReuse:
    def test_leaf_reuse(self) -> None:
        """The same Behavior leaf can appear as a child of one parent multiple times."""
        wait = bt.Wait(1.0)
        tree = bt.Sequence([wait, wait, wait])
        assert repr(tree) == "Sequence(3)"

    def test_nested_subtree_reuse(self) -> None:
        """Nested subtrees are reusable too — the same composite can be a child multiple times."""
        inner = bt.Sequence([bt.Wait(0.1), bt.Action("x")])
        outer = bt.Sequence([inner, inner, inner])
        assert repr(outer) == "Sequence(3)"

    def test_subtree_reused_across_bts(self) -> None:
        """The same Behavior root can drive multiple independent BTs without interference."""
        subtree = bt.Sequence([bt.Action("a"), bt.Action("b")])
        calls1: list[Any] = []
        calls2: list[Any] = []

        def make_cb(out: list[Any]) -> Callable[[Any, Any], tuple[bt.Status, float]]:
            def cb(args: Any, _bb: Any) -> tuple[bt.Status, float]:
                out.append(args.action)
                return (bt.Status.Success, 0.0)
            return cb

        bt.BT(subtree, None).tick(0.0, make_cb(calls1))
        bt.BT(subtree, None).tick(0.0, make_cb(calls2))
        assert calls1 == ["a", "b"]
        assert calls2 == ["a", "b"]


class TestArgumentParsing:
    def test_kwargs_on_if(self) -> None:
        """If accepts cond / on_success / on_failure as keyword arguments."""
        tree = bt.If(
            cond=bt.Action("c"),
            on_success=bt.Action("s"),
            on_failure=bt.Action("f"),
        )
        assert repr(tree) == "If(...)"

    def test_tuple_accepted_for_children(self) -> None:
        """PyO3 Vec<T> extractor accepts indexable sequences, not just list."""
        tree = bt.Sequence((bt.Action("a"), bt.Action("b")))
        assert repr(tree) == "Sequence(2)"

    def test_generator_rejected_for_children(self) -> None:
        """Generators are NOT accepted (extractor needs random access)."""
        with pytest.raises(TypeError):
            bt.Sequence(bt.Action(x) for x in ["a", "b"])


class TestIdentityEquality:
    @pytest.mark.parametrize(
        "build",
        [
            lambda: bt.Action("x"),
            lambda: bt.Wait(1.0),
            lambda: bt.WaitForever(),
            lambda: bt.Invert(bt.Action("x")),
            lambda: bt.AlwaysSucceed(bt.Action("x")),
            lambda: bt.Sequence([bt.Action("x")]),
            lambda: bt.Select([bt.Action("x")]),
            lambda: bt.WhenAll([bt.Action("x")]),
            lambda: bt.WhenAny([bt.Action("x")]),
            lambda: bt.After([bt.Action("x")]),
            lambda: bt.Race([bt.Action("x")]),
            lambda: bt.If(bt.Action("c"), bt.Action("s"), bt.Action("f")),
            lambda: bt.While(bt.Action("c"), [bt.Action("b")]),
            lambda: bt.WhileAll(bt.Action("c"), [bt.Action("b")]),
        ],
    )
    def test_identity_based_eq_per_variant(
        self, build: Callable[[], bt.Behavior]
    ) -> None:
        """Two structurally-identical Behaviors compare unequal — equality is identity."""
        a, b = build(), build()
        assert a is not b
        assert (a == b) is False
        assert a != b


class TestBehaviorAttribution:
    def test_module(self) -> None:
        """Behavior.__module__ is `bonsai_bt` (required for pickle / introspection)."""
        assert bt.Behavior.__module__ == "bonsai_bt"


# ---------- Ports of Rust behavior_tests.rs (golden-truth equivalence) ----------

class TestBehaviorRustParity:
    """Tests ported from bonsai/tests/behavior_tests.rs. If a Rust test changes,
    its Python counterpart must change too. Main drift gate against Rust semantics."""

    def test_immediate_termination(self) -> None:
        """A 0.0s tick runs all leaves; reset_bt then re-runs the whole sequence."""
        acc = [0]

        def cb(args: Any, _bb: Any) -> tuple[bt.Status, float]:
            if args.action == "inc":
                acc[0] += 1
            return (bt.Status.Success, args.dt)

        tree = bt.Sequence([bt.Action("inc"), bt.Action("inc")])
        b = bt.BT(tree, None)
        b.tick(0.0, cb)
        assert acc[0] == 2
        assert b.is_finished()
        b.reset_bt()
        b.tick(1.0, cb)
        assert acc[0] == 4
        assert b.is_finished()

    def test_sequence_of_wait_then_action(self) -> None:
        """A single tick with enough dt completes both Wait and the trailing Action."""
        seen: list[Any] = []

        def cb(args: Any, _bb: Any) -> tuple[bt.Status, float]:
            seen.append(args.action)
            return (bt.Status.Success, args.dt)

        b = bt.BT(bt.Sequence([bt.Wait(1.0), bt.Action("inc")]), None)
        b.tick(1.0, cb)
        assert seen == ["inc"]

    def test_wait_half_then_half(self) -> None:
        """Two 0.5s ticks accumulate to clear a 1.0s Wait before the Action fires."""
        seen: list[Any] = []

        def cb(args: Any, _bb: Any) -> tuple[bt.Status, float]:
            seen.append(args.action)
            return (bt.Status.Success, args.dt)

        b = bt.BT(bt.Sequence([bt.Wait(1.0), bt.Action("inc")]), None)
        b.tick(0.5, cb)
        assert seen == []
        b.tick(0.5, cb)
        assert seen == ["inc"]

    def test_select_succeed_on_first(self) -> None:
        """Select short-circuits at the first Success; later siblings are never invoked."""
        calls: list[Any] = []

        def cb(args: Any, _bb: Any) -> tuple[bt.Status, float]:
            calls.append(args.action)
            return (bt.Status.Success, args.dt)

        tree = bt.Select([bt.Action("a"), bt.Action("b"), bt.Action("c")])
        b = bt.BT(tree, None)
        b.tick(0.1, cb)
        assert calls == ["a"]

    def test_select_first_failure_tries_next(self) -> None:
        """Select advances past Failures and reports Success at the first successful child."""
        calls: list[Any] = []

        def cb(args: Any, _bb: Any) -> tuple[bt.Status, float]:
            calls.append(args.action)
            if args.action == "fail":
                return (bt.Status.Failure, args.dt)
            return (bt.Status.Success, args.dt)

        tree = bt.Select([bt.Action("fail"), bt.Action("ok")])
        b = bt.BT(tree, None)
        result = b.tick(0.1, cb)
        assert result is not None
        status, _ = result
        assert status == bt.Status.Success
        assert calls == ["fail", "ok"]

    def test_if_true_branch(self) -> None:
        """If runs on_success when the condition succeeds; on_failure is not invoked."""
        seen: list[Any] = []

        def cb(args: Any, _bb: Any) -> tuple[bt.Status, float]:
            seen.append(args.action)
            return (bt.Status.Success, args.dt)

        tree = bt.If(bt.Action("cond_true"), bt.Action("yes"), bt.Action("no"))
        b = bt.BT(tree, None)
        b.tick(0.1, cb)
        assert "yes" in seen
        assert "no" not in seen

    def test_if_false_branch(self) -> None:
        """If runs on_failure when the condition fails; on_success is not invoked."""
        seen: list[Any] = []

        def cb(args: Any, _bb: Any) -> tuple[bt.Status, float]:
            seen.append(args.action)
            if args.action == "cond_false":
                return (bt.Status.Failure, args.dt)
            return (bt.Status.Success, args.dt)

        tree = bt.If(bt.Action("cond_false"), bt.Action("yes"), bt.Action("no"))
        b = bt.BT(tree, None)
        b.tick(0.1, cb)
        assert "no" in seen
        assert "yes" not in seen

    def test_invert_swaps_outcomes(self) -> None:
        """Invert flips Success <-> Failure on the child's return status."""
        def yields_success(_a: Any, _b: Any) -> tuple[bt.Status, float]:
            return (bt.Status.Success, 0.0)

        def yields_failure(_a: Any, _b: Any) -> tuple[bt.Status, float]:
            return (bt.Status.Failure, 0.0)

        b1 = bt.BT(bt.Invert(bt.Action("x")), None)
        r = b1.tick(0.0, yields_success)
        assert r is not None
        assert r[0] == bt.Status.Failure

        b2 = bt.BT(bt.Invert(bt.Action("x")), None)
        r = b2.tick(0.0, yields_failure)
        assert r is not None
        assert r[0] == bt.Status.Success

    def test_always_succeed_swallows_failure(self) -> None:
        """AlwaysSucceed coerces a child's Failure into Success."""
        def yields_failure(_a: Any, _b: Any) -> tuple[bt.Status, float]:
            return (bt.Status.Failure, 0.0)

        b = bt.BT(bt.AlwaysSucceed(bt.Action("x")), None)
        r = b.tick(0.0, yields_failure)
        assert r is not None
        assert r[0] == bt.Status.Success

    def test_when_all_waits_for_all(self) -> None:
        """WhenAll blocks the parent Sequence until both parallel children finish."""
        seen: list[Any] = []

        def cb(args: Any, _bb: Any) -> tuple[bt.Status, float]:
            seen.append(args.action)
            return (bt.Status.Success, args.dt)

        tree = bt.Sequence([
            bt.WhenAll([bt.Wait(0.5), bt.Wait(1.0)]),
            bt.Action("after"),
        ])
        b = bt.BT(tree, None)
        b.tick(0.5, cb)
        assert seen == []
        b.tick(0.5, cb)
        assert seen == ["after"]

    def test_while_loops_until_cond_fails(self) -> None:
        """While re-runs its body each iteration while the cond stays Running."""
        seen: list[str] = []

        def cb(args: Any, _bb: Any) -> tuple[bt.Status, float]:
            seen.append(args.action)
            return (bt.Status.Success, args.dt)

        tree = bt.While(bt.Wait(50.0), [bt.Wait(0.5), bt.Action("tick"), bt.Wait(0.5)])
        b = bt.BT(tree, None)
        b.tick(10.0, cb)
        assert seen.count("tick") == 10
