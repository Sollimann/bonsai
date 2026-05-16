"""BT class: tick mechanics, callback errors, blackboard, reset, finished state."""
from __future__ import annotations

from typing import Any, Callable

import pytest

import bonsai_py as bt


class TestBTConstruction:
    def test_dict_blackboard(self) -> None:
        """A dict can serve as the blackboard; round-trips by equality."""
        b = bt.BT(bt.Action("x"), {"k": 1})
        assert b.blackboard() == {"k": 1}

    def test_none_blackboard(self) -> None:
        """None is a legal blackboard for trees that don't need shared state."""
        b = bt.BT(bt.Action("x"), None)
        assert b.blackboard() is None

    def test_custom_blackboard(self) -> None:
        """Any Python object can be the blackboard; identity is preserved."""
        class State:
            def __init__(self) -> None:
                self.counter = 0

        s = State()
        b = bt.BT(bt.Action("x"), s)
        assert b.blackboard() is s

    def test_module(self) -> None:
        """BT.__module__ is `bonsai_py` (required for pickle / introspection)."""
        assert bt.BT.__module__ == "bonsai_py"


class TestTick:
    def test_doctest_equivalent(self) -> None:
        """Line-for-line port of the bonsai/src/lib.rs Rust doctest: 5 ticks at 0.5s land count==1."""
        tree = bt.Sequence([
            bt.Wait(1.0), bt.Action("inc"),
            bt.Wait(1.0), bt.Action("inc"),
            bt.Wait(0.5), bt.Action("dec"),
        ])
        bb = {"count": 0}
        b = bt.BT(tree, bb)
        acc = 0

        def cb(args: Any, _blackboard: Any) -> tuple[bt.Status, float]:
            nonlocal acc
            if args.action == "inc":
                acc += 1
                return (bt.Status.Success, args.dt)
            if args.action == "dec":
                acc -= 1
                return (bt.Status.Success, args.dt)
            return bt.RUNNING

        for _ in range(5):
            b.tick(0.5, cb)
        bb["count"] = acc
        assert bb["count"] == 1
        assert b.tick_count() == 5

    def test_tick_returns_status_and_dt(
        self,
        basic_tree: bt.Behavior,
        noop_callback: Callable[[Any, Any], tuple[Any, float]],
    ) -> None:
        """tick() returns a (Status, float) tuple when the BT has not yet finished."""
        b = bt.BT(basic_tree, None)
        result = b.tick(2.0, noop_callback)
        assert result is not None
        status, remaining = result
        assert isinstance(status, bt.Status)
        assert isinstance(remaining, float)

    def test_tick_on_finished_returns_none(self) -> None:
        """Once is_finished() is True, every subsequent tick() returns None."""
        def done(_a: Any, _b: Any) -> tuple[bt.Status, float]:
            return (bt.Status.Success, 0.0)

        b = bt.BT(bt.Action("x"), None)
        b.tick(0.0, done)
        assert b.is_finished()
        assert b.tick(0.0, done) is None

    def test_reset_bt_on_unstarted_tree(self) -> None:
        """reset_bt() on a never-ticked BT is a no-op; subsequent tick still works normally."""
        b = bt.BT(bt.Action("x"), None)
        b.reset_bt()
        assert not b.is_finished()
        assert b.tick_count() == 0
        result = b.tick(0.0, lambda _a, _bb: (bt.Status.Success, 0.0))
        assert result is not None

    def test_tick_count_survives_reset(
        self,
        noop_callback: Callable[[Any, Any], tuple[Any, float]],
    ) -> None:
        """tick_count accumulates across ticks and persists across reset_bt (never zeroed)."""
        b = bt.BT(bt.Wait(10.0), None)
        for _ in range(3):
            b.tick(0.1, noop_callback)
        assert b.tick_count() == 3
        b.reset_bt()
        assert b.tick_count() == 3, "tick_count must survive reset_bt"
        assert not b.is_finished()


class TestCallbackContract:
    def test_callback_exception_propagates_with_message(self) -> None:
        """A Python exception raised inside the callback bubbles up through tick() intact."""
        def boom(_a: Any, _b: Any) -> tuple[bt.Status, float]:
            raise ValueError("boom")

        b = bt.BT(bt.Action("x"), None)
        with pytest.raises(ValueError, match="boom"):
            b.tick(0.0, boom)

    def test_callback_wrong_return_shape_rejected(self) -> None:
        """A callback that returns a non-(Status, float) value is rejected by the extractor."""
        def bad(_a: Any, _b: Any) -> str:
            return "not a tuple"

        b = bt.BT(bt.Action("x"), None)
        with pytest.raises(Exception):
            b.tick(0.0, bad)

    def test_when_all_short_circuits_on_callback_raise(self) -> None:
        """After a callback raises on a WhenAll child, later siblings in the same tick are not invoked."""
        order: list[str] = []

        def cb(args: Any, _bb: Any) -> tuple[bt.Status, float]:
            order.append(args.action)
            if args.action == "b":
                raise ValueError("stop")
            return (bt.Status.Success, 0.0)

        tree = bt.WhenAll([bt.Action("a"), bt.Action("b"), bt.Action("c")])
        b = bt.BT(tree, None)
        with pytest.raises(ValueError):
            b.tick(0.0, cb)
        assert order == ["a", "b"]

    def test_callback_returns_nan_dt_sanitized(self) -> None:
        """A NaN dt returned from the callback is sanitized by upstream BT and never surfaces to Python."""
        import math

        def nan_cb(args: Any, _bb: Any) -> tuple[bt.Status, float]:
            if args.action == "x":
                return (bt.Status.Running, float("nan"))
            return (bt.Status.Success, args.dt)

        b = bt.BT(bt.Sequence([bt.Action("x")]), None)
        result = b.tick(1.0, nan_cb)
        assert result is not None
        _, dt = result
        assert not math.isnan(dt)


class TestBlackboard:
    def test_blackboard_identity_preserved(self) -> None:
        """blackboard() returns the same Python object passed to BT() — not a copy."""
        bb = {"count": 0}
        b = bt.BT(bt.Action("x"), bb)
        assert b.blackboard() is bb

    def test_blackboard_mutation_persists_via_callback(self) -> None:
        """Mutations done through the callback's blackboard handle persist in the original object."""
        bb = {"count": 0}

        def inc(_args: Any, blackboard: Any) -> tuple[bt.Status, float]:
            blackboard["count"] += 1
            return (bt.Status.Success, 0.0)

        b = bt.BT(bt.Sequence([bt.Action("x"), bt.Action("x")]), bb)
        b.tick(0.0, inc)
        assert bb["count"] == 2

    def test_action_identity_through_callback(self) -> None:
        """The action object passed to bt.Action(...) arrives at the callback by identity, not equality."""
        sentinel = object()
        seen: list[bool] = []

        def cb(args: Any, _bb: Any) -> tuple[bt.Status, float]:
            seen.append(args.action is sentinel)
            return (bt.Status.Success, 0.0)

        b = bt.BT(bt.Action(sentinel), None)
        b.tick(0.0, cb)
        assert seen == [True]

    def test_reset_preserves_blackboard(self) -> None:
        """reset_bt() rewinds tree state but does NOT touch the blackboard contents."""
        bb = {"count": 5}
        b = bt.BT(bt.Action("x"), bb)

        def done(_a: Any, _b: Any) -> tuple[bt.Status, float]:
            return (bt.Status.Success, 0.0)

        b.tick(0.0, done)
        b.reset_bt()
        assert b.blackboard() is bb
        assert bb["count"] == 5


class TestRunningConstant:
    def test_value(self) -> None:
        """bt.RUNNING is the tuple (Status.Running, 0.0)."""
        assert bt.RUNNING == (bt.Status.Running, 0.0)

    def test_first_element_is_running(self) -> None:
        """Unpacking bt.RUNNING yields Status.Running as the first element."""
        status, _ = bt.RUNNING
        assert status is bt.Status.Running

    def test_used_as_callback_return(self) -> None:
        """`return bt.RUNNING` from a callback is a valid Running tick — BT stays unfinished."""

        def keep_running(_a: Any, _b: Any) -> tuple[bt.Status, float]:
            return bt.RUNNING

        b = bt.BT(bt.Action("x"), None)
        result = b.tick(0.0, keep_running)
        assert result is not None
        status, _ = result
        assert status == bt.Status.Running
        assert not b.is_finished()
