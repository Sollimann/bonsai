"""Performance budget -- generous bounds to avoid CI flakiness. Heavy
benchmarks gated behind `@pytest.mark.bench` (run with `pytest -m bench`)."""
from __future__ import annotations

import time
from typing import Any

import pytest

import bonsai_bt as bt


@pytest.mark.perf
class TestTickBudget:
    def test_simple_tick_100_iterations_under_500ms(self) -> None:
        """100 BT(tree).tick(cb) round-trips complete under 500 ms — catches order-of-magnitude regressions."""

        def cb(_a: Any, _b: Any) -> tuple[bt.Status, float]:
            return (bt.Status.Success, 0.0)

        # Hoist tree construction; each iter still needs a fresh BT because
        # the tree finishes after one tick and tick() returns None thereafter.
        tree = bt.Sequence([bt.Action("x")])
        start = time.perf_counter()
        for _ in range(100):
            bt.BT(tree, None).tick(0.0, cb)
        elapsed = time.perf_counter() - start
        assert elapsed < 0.5, f"100 ticks took {elapsed*1000:.0f}ms (budget: 500 ms)"

    def test_construction_under_5_seconds_per_1000(self) -> None:
        """1000 Sequence([Action, Wait]) constructions fit in 5 seconds — guards against construction-time blowups."""
        start = time.perf_counter()
        for _ in range(1000):
            bt.Sequence([bt.Action("a"), bt.Wait(0.1)])
        elapsed = time.perf_counter() - start
        assert elapsed < 5.0, f"1000 constructions took {elapsed*1000:.0f}ms"


@pytest.mark.bench
class TestBenchmarks:
    """Microbenchmarks for local profiling. Run with `pytest -m bench`."""

    def test_bench_tick_throughput(self) -> None:
        """Prints achieved ticks/sec for 10_000 BT.tick round-trips — informational, no assertion."""
        import timeit

        def one_tick() -> None:
            b = bt.BT(bt.Action("x"), None)
            b.tick(0.0, lambda _a, _bb: (bt.Status.Success, 0.0))

        n = 10_000
        elapsed = timeit.timeit(one_tick, number=n)
        print(f"\nbench: {n} ticks in {elapsed:.3f}s = {n/elapsed:.0f} ticks/sec")
