# -*- coding: utf-8 -*-
"""Cross-process and cross-thread semantics: BT is unsendable + unpicklable."""
from __future__ import annotations

import pickle
import threading

import pytest

import bonsai_py as bt


class TestBTUnsendable:
    def test_thread_send_raises_panic(self) -> None:
        """PyBT is unsendable; touching it from a different thread raises PyO3's PanicException."""
        b = bt.BT(bt.Action("x"), None)
        captured: list[BaseException] = []

        def worker() -> None:
            try:
                b.tick(0.0, lambda _a, _b: (bt.Status.Success, 0.0))
            except BaseException as e:  # PanicException is BaseException-subclass
                captured.append(e)

        t = threading.Thread(target=worker)
        t.start()
        t.join()

        assert len(captured) == 1
        msg = str(captured[0])
        assert "unsendable" in msg or "thread" in msg.lower()


class TestBTUnpicklable:
    def test_bt_not_picklable(self) -> None:
        """BT instances have no __reduce__ — pickling raises TypeError/PicklingError."""
        b = bt.BT(bt.Action("x"), None)
        with pytest.raises((TypeError, pickle.PicklingError)):
            pickle.dumps(b)


class TestBehaviorUnpicklable:
    def test_behavior_not_picklable(self) -> None:
        """Behavior nodes are not picklable (no __reduce__ implemented)."""
        with pytest.raises((TypeError, pickle.PicklingError)):
            pickle.dumps(bt.Action("x"))


class TestActionArgsUnpicklable:
    def test_action_args_not_picklable(self) -> None:
        """ActionArgs instances are not picklable (no __reduce__ implemented)."""
        with pytest.raises((TypeError, pickle.PicklingError)):
            pickle.dumps(bt.ActionArgs(0.5, "x"))


class TestStatusPicklableAcrossProcesses:
    """Status IS picklable — the one multiprocessing-friendly type in the binding."""

    def test_status_round_trip_through_pickle(self) -> None:
        """Each Status variant round-trips through pickle and returns the same singleton."""
        for s in (bt.Status.Success, bt.Status.Failure, bt.Status.Running):
            assert pickle.loads(pickle.dumps(s)) is s
