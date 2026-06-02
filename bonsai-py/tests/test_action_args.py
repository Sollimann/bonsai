"""ActionArgs: construction, identity, frozen, repr."""
from __future__ import annotations

import math

import pytest

import bonsai_bt as bt


class TestActionArgsConstruction:
    def test_construct_with_string_action(self) -> None:
        """String action is stored verbatim and accessible via .action."""
        a = bt.ActionArgs(0.5, "inc")
        assert a.dt == 0.5
        assert a.action == "inc"

    def test_construct_with_none_action(self) -> None:
        """None is a valid action value (preserved by identity)."""
        a = bt.ActionArgs(0.0, None)
        assert a.action is None

    def test_construct_with_arbitrary_object(self) -> None:
        """Any Python object can be an action; identity is preserved."""
        sentinel = object()
        a = bt.ActionArgs(0.1, sentinel)
        assert a.action is sentinel

    def test_construct_with_dict_action(self) -> None:
        """Dict actions are passed by reference — mutations are visible."""
        d = {"type": "fire"}
        a = bt.ActionArgs(0.0, d)
        assert a.action is d

    def test_construct_with_int_dt(self) -> None:
        """int dt coerces to float at the FFI boundary."""
        a = bt.ActionArgs(1, "x")
        assert a.dt == 1.0
        assert isinstance(a.dt, float)

    @pytest.mark.parametrize("dt", [float("nan"), float("inf"), float("-inf")])
    def test_construct_with_special_dt(self, dt: float) -> None:
        """NaN / +inf / -inf dt are accepted at construction — no guard at the FFI boundary."""
        a = bt.ActionArgs(dt, "x")
        if math.isnan(dt):
            assert math.isnan(a.dt)
        else:
            assert a.dt == dt


class TestActionArgsImmutability:
    def test_dt_readonly(self) -> None:
        """ActionArgs.dt is frozen — assignment raises AttributeError."""
        a = bt.ActionArgs(0.5, "x")
        with pytest.raises(AttributeError):
            a.dt = 0.7  # type: ignore[misc]

    def test_action_readonly(self) -> None:
        """ActionArgs.action is frozen — assignment raises AttributeError."""
        a = bt.ActionArgs(0.5, "x")
        with pytest.raises(AttributeError):
            a.action = "y"  # type: ignore[misc]


class TestActionArgsRepr:
    @pytest.mark.parametrize(
        "dt, action, expected",
        [
            (0.5, "inc", "ActionArgs(dt=0.5, action='inc')"),
            (0.0, None, "ActionArgs(dt=0, action=None)"),
            (1.0, 42, "ActionArgs(dt=1, action=42)"),
        ],
    )
    def test_repr_format(self, dt: float, action: object, expected: str) -> None:
        """repr() renders as `ActionArgs(dt=..., action=...)` with Python repr on the action."""
        assert repr(bt.ActionArgs(dt, action)) == expected


class TestActionArgsModuleAttribution:
    def test_module(self) -> None:
        """ActionArgs.__module__ is `bonsai_bt` (required for pickle / introspection)."""
        assert bt.ActionArgs.__module__ == "bonsai_bt"
