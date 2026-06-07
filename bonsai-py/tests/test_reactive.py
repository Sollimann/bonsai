"""ReactiveSequence / ReactiveSelect tick semantics through the PyO3 bindings.

Mirrors the headline cases from the Rust integration tests in
`bonsai/tests/behavior_tests.rs` to confirm the Python factories produce
behaviors with identical short-circuit semantics.
"""
from __future__ import annotations

from typing import Any

import bonsai_bt as bt


def _short_circuit_callback(
    *, cond_passes: bool, count_box: list[int]
) -> Any:
    """Build a callback whose `cond` returns Success/Failure based on the toggle
    and whose `inc` bumps a counter and returns Success. Used to assert that
    later children are NOT ticked when the composite short-circuits.
    """

    def cb(args: Any, _bb: Any) -> tuple[bt.Status, float]:
        action = args.action
        if action == "cond":
            return (bt.Status.Success if cond_passes else bt.Status.Failure, args.dt)
        if action == "inc":
            count_box[0] += 1
            return (bt.Status.Success, args.dt)
        raise AssertionError(f"unknown action: {action!r}")

    return cb


class TestReactiveSequenceShortCircuit:
    def test_failure_short_circuits(self) -> None:
        """A failing condition aborts the composite; later children are not ticked."""
        count: list[int] = [0]
        tree = bt.ReactiveSequence([bt.Action("cond"), bt.Action("inc")])
        machine = bt.BT(tree, None)
        status, _dt = machine.tick(
            0.0, _short_circuit_callback(cond_passes=False, count_box=count)
        )
        assert status == bt.Status.Failure
        assert count[0] == 0, "later children must not be ticked on failure"

    def test_all_success(self) -> None:
        """All children succeed → composite returns Success and BT finishes."""
        count: list[int] = [0]
        tree = bt.ReactiveSequence([bt.Action("cond"), bt.Action("inc")])
        machine = bt.BT(tree, None)
        status, _dt = machine.tick(
            0.0, _short_circuit_callback(cond_passes=True, count_box=count)
        )
        assert status == bt.Status.Success
        assert count[0] == 1
        assert machine.is_finished()

    def test_empty_is_success(self) -> None:
        """Empty ReactiveSequence is vacuous Success."""
        tree = bt.ReactiveSequence([])
        machine = bt.BT(tree, None)
        status, _dt = machine.tick(0.0, lambda _a, _bb: (bt.Status.Success, 0.0))
        assert status == bt.Status.Success


class TestReactiveSelectShortCircuit:
    def test_success_short_circuits(self) -> None:
        """First succeeding child wins; later children (including a counter) are not ticked."""
        count: list[int] = [0]
        tree = bt.ReactiveSelect([bt.Action("cond"), bt.Action("inc")])
        machine = bt.BT(tree, None)
        status, _dt = machine.tick(
            0.0, _short_circuit_callback(cond_passes=True, count_box=count)
        )
        assert status == bt.Status.Success
        assert count[0] == 0, "short-circuited siblings must not be ticked"

    def test_all_fail_returns_failure(self) -> None:
        """Every child fails → composite returns Failure."""
        # Both children return Failure, so the trailing inc would only run if
        # the composite walked past them — which it does only on Failure path.
        count: list[int] = [0]

        def cb(args: Any, _bb: Any) -> tuple[bt.Status, float]:
            if args.action in ("a", "b"):
                return (bt.Status.Failure, args.dt)
            raise AssertionError(args.action)

        tree = bt.ReactiveSelect([bt.Action("a"), bt.Action("b")])
        machine = bt.BT(tree, None)
        status, _dt = machine.tick(0.0, cb)
        assert status == bt.Status.Failure
        # Sanity: count box stays at 0 since no "inc" exists in this tree.
        assert count[0] == 0

    def test_empty_is_failure(self) -> None:
        """Empty ReactiveSelect is vacuous Failure (dual of empty ReactiveSequence)."""
        tree = bt.ReactiveSelect([])
        machine = bt.BT(tree, None)
        status, _dt = machine.tick(0.0, lambda _a, _bb: (bt.Status.Success, 0.0))
        assert status == bt.Status.Failure


class TestReactiveSequenceReEvaluation:
    def test_running_child_is_aborted_when_earlier_condition_fails_next_tick(self) -> None:
        """Headline reactive behavior: a previously-running child must NOT be
        resumed when an earlier condition flips to Failure on the next tick."""
        state = {"cond_passes": True, "long_action_ticks": 0}

        def cb(args: Any, _bb: Any) -> tuple[bt.Status, float]:
            action = args.action
            if action == "cond":
                return (bt.Status.Success if state["cond_passes"] else bt.Status.Failure, args.dt)
            if action == "long":
                state["long_action_ticks"] += 1
                return (bt.Status.Running, 0.0)
            raise AssertionError(action)

        tree = bt.ReactiveSequence([bt.Action("cond"), bt.Action("long")])
        machine = bt.BT(tree, None)

        # Tick 1: cond passes, long action returns Running → composite Running.
        status1, _ = machine.tick(0.0, cb)
        assert status1 == bt.Status.Running
        assert state["long_action_ticks"] == 1

        # Flip the condition externally.
        state["cond_passes"] = False

        # Tick 2: cond now fails → composite Failure short-circuits BEFORE long.
        status2, _ = machine.tick(0.0, cb)
        assert status2 == bt.Status.Failure
        assert state["long_action_ticks"] == 1, (
            "the previously-running long action must NOT be re-ticked once the "
            "earlier condition fails"
        )


class TestReactiveSequenceRepr:
    """The Phase 3 binding wiring is sanity-checked elsewhere, but the
    reactive-specific repr format is asserted here in the reactive test file."""

    def test_repr_includes_child_count(self) -> None:
        node = bt.ReactiveSequence([bt.Action("a"), bt.Action("b"), bt.Action("c")])
        assert repr(node) == "ReactiveSequence(3)"

    def test_select_repr_includes_child_count(self) -> None:
        node = bt.ReactiveSelect([bt.Action("a")])
        assert repr(node) == "ReactiveSelect(1)"

    def test_empty_repr(self) -> None:
        assert repr(bt.ReactiveSequence([])) == "ReactiveSequence(0)"
        assert repr(bt.ReactiveSelect([])) == "ReactiveSelect(0)"
