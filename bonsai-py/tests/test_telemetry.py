"""with_telemetry: chainable, bind failures, poisoned state, host parameter."""
from __future__ import annotations

from typing import Any

import pytest

import bonsai_py as bt


class TestWithTelemetry:
    def test_chainable(self, free_port: int) -> None:
        """with_telemetry returns the same BT instance (PyRefMut self), enabling fluent chaining."""
        b = bt.BT(bt.Action("x"), None)
        b_after = b.with_telemetry(free_port)
        assert b_after is b
        result = b_after.tick(0.0, lambda _a, _bb: (bt.Status.Success, 0.0))
        assert result is not None

    def test_host_parameter(self, free_port: int) -> None:
        """The optional `host` kwarg lets the listener bind to a non-loopback interface."""
        b = bt.BT(bt.Action("x"), None).with_telemetry(free_port, host="127.0.0.1")
        assert b is not None

    def test_explicit_loopback(self, free_port: int) -> None:
        """All-keyword form (port=..., host=...) works for callers that prefer kwargs."""
        b = bt.BT(bt.Action("x"), None).with_telemetry(port=free_port, host="127.0.0.1")
        assert b is not None

    def test_bound_port_raises_os_error(self, free_port: int) -> None:
        """A second bind on a port held by another BT raises OSError with the bind message."""
        holder = bt.BT(bt.Action("x"), None).with_telemetry(free_port)
        assert holder is not None
        with pytest.raises(OSError, match="failed"):
            bt.BT(bt.Action("y"), None).with_telemetry(free_port)

    def test_unbindable_host_raises_os_error(self) -> None:
        """Binding to an RFC-reserved address (240.0.0.0/4) raises OSError without hitting DNS."""
        with pytest.raises(OSError):
            bt.BT(bt.Action("x"), None).with_telemetry(0, host="240.0.0.1")


class TestPoisonedBT:
    def test_failed_with_telemetry_poisons_bt(self, free_port: int) -> None:
        """A failed with_telemetry poisons the BT; every subsequent method raises RuntimeError."""
        holder = bt.BT(bt.Action("x"), None).with_telemetry(free_port)
        assert holder is not None

        victim = bt.BT(bt.Action("y"), None)
        with pytest.raises(OSError):
            victim.with_telemetry(free_port)

        with pytest.raises(RuntimeError, match="invalidated"):
            victim.tick(0.0, lambda _a, _b: (bt.Status.Success, 0.0))

        with pytest.raises(RuntimeError, match="invalidated"):
            victim.blackboard()

        with pytest.raises(RuntimeError, match="invalidated"):
            victim.tick_count()
