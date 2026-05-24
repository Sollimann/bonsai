"""Status enum: semantics, pickle, copy, hash, identity."""
from __future__ import annotations

import copy
import pickle

import pytest

import bonsai_bt as bt


class TestStatusSemantics:
    def test_three_variants(self) -> None:
        """All three Status variants exist and form a non-empty set."""
        assert {bt.Status.Success, bt.Status.Failure, bt.Status.Running}

    @pytest.mark.parametrize(
        "variant, expected_int",
        [(bt.Status.Success, 0), (bt.Status.Failure, 1), (bt.Status.Running, 2)],
    )
    def test_eq_int_discriminant(self, variant: bt.Status, expected_int: int) -> None:
        """Discriminants 0/1/2 are locked; reordering is a breaking change."""
        assert variant == expected_int
        assert int(variant) == expected_int

    def test_equality(self) -> None:
        """Same variant compares equal; different variants compare unequal."""
        assert bt.Status.Success == bt.Status.Success
        assert bt.Status.Success != bt.Status.Failure
        assert bt.Status.Success != bt.Status.Running

    def test_identity_singleton(self) -> None:
        """PyO3 simple enums are singletons; `is` comparison works."""
        assert bt.Status.Success is bt.Status.Success

    def test_hashable_as_dict_key(self) -> None:
        """Status implements __hash__ and is usable as a dict key / set member."""
        d = {bt.Status.Success: "ok", bt.Status.Failure: "no"}
        assert d[bt.Status.Success] == "ok"
        assert d[bt.Status.Failure] == "no"

    def test_repr(self) -> None:
        """repr() returns the dotted variant name (Status.Success / Failure / Running)."""
        assert repr(bt.Status.Success) == "Status.Success"
        assert repr(bt.Status.Failure) == "Status.Failure"
        assert repr(bt.Status.Running) == "Status.Running"

    def test_module_attribution(self) -> None:
        """`module = "bonsai_bt"` is set on the pyclass — required for pickle."""
        assert bt.Status.__module__ == "bonsai_bt"


class TestStatusPickle:
    @pytest.mark.parametrize(
        "variant", [bt.Status.Success, bt.Status.Failure, bt.Status.Running]
    )
    def test_pickle_preserves_singleton_identity(self, variant: bt.Status) -> None:
        """pickle round-trip returns the same singleton (not a copy)."""
        roundtripped = pickle.loads(pickle.dumps(variant))
        assert roundtripped is variant

    @pytest.mark.parametrize(
        "variant", [bt.Status.Success, bt.Status.Failure, bt.Status.Running]
    )
    def test_copy_preserves_singleton_identity(self, variant: bt.Status) -> None:
        """copy.copy and copy.deepcopy preserve singleton identity."""
        assert copy.copy(variant) is variant
        assert copy.deepcopy(variant) is variant

    def test_dict_with_status_round_trips(self) -> None:
        """multiprocessing-style pickle: a dict containing Status survives serialization."""
        data = pickle.dumps({"result": bt.Status.Success, "count": 3})
        out = pickle.loads(data)
        assert out["result"] is bt.Status.Success
        assert out["count"] == 3
