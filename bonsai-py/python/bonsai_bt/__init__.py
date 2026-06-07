"""bonsai-bt - behavior trees in Python, powered by the Rust bonsai-bt crate."""

from importlib.metadata import PackageNotFoundError, version as _version

try:
    __version__ = _version("bonsai-bt")
except PackageNotFoundError:  # editable install before metadata is in place
    __version__ = "0.0.0+unknown"

from .bonsai_bt import *  # noqa: F401,F403  (re-export the compiled module)

__all__ = [
    # types
    "Status", "ActionArgs", "Behavior", "BT",
    # factories (leaves, decorators, composites, control flow)
    "Action", "Wait", "WaitForever",
    "Invert", "AlwaysSucceed",
    "Sequence", "Select", "ReactiveSequence", "ReactiveSelect",
    "WhenAll", "WhenAny", "After", "Race",
    "If", "While", "WhileAll",
    # constants
    "RUNNING",
]
