# -*- coding: utf-8 -*-
"""Shared fixtures for bonsai-py tests."""
from __future__ import annotations

import socket
from typing import Any, Callable

import pytest

import bonsai_py as bt


@pytest.fixture
def free_port() -> int:
    """Return a free TCP port (kernel-assigned)."""
    with socket.socket() as s:
        s.bind(("127.0.0.1", 0))
        return int(s.getsockname()[1])


@pytest.fixture
def noop_callback() -> Callable[[Any, Any], tuple[Any, float]]:
    """A callback that returns Success immediately with no side effects."""

    def cb(_args: Any, _bb: Any) -> tuple[Any, float]:
        return (bt.Status.Success, 0.0)

    return cb


@pytest.fixture
def counting_callback() -> tuple[Callable[[Any, Any], tuple[Any, float]], list[Any]]:
    """A callback that records every action it sees and returns Success."""
    calls: list[Any] = []

    def cb(args: Any, _bb: Any) -> tuple[Any, float]:
        calls.append(args.action)
        return (bt.Status.Success, args.dt)

    return cb, calls


@pytest.fixture
def basic_tree() -> bt.Behavior:
    """A small reusable tree: Sequence([Wait(0.5), Action('inc'), Wait(0.5), Action('inc')])."""
    return bt.Sequence([
        bt.Wait(0.5),
        bt.Action("inc"),
        bt.Wait(0.5),
        bt.Action("inc"),
    ])
