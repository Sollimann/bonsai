
<p align="center">
  <img src="https://raw.githubusercontent.com/Sollimann/bonsai/main/docs/resources/gifs/bonsai.gif" width="350" alt="Bonsai logo">
</p>

[![PyPI](https://img.shields.io/pypi/v/bonsai-bt.svg)](https://pypi.org/project/bonsai-bt/)
[![Python](https://img.shields.io/pypi/pyversions/bonsai-bt.svg)](https://pypi.org/project/bonsai-bt/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Python bindings for the [bonsai-bt](https://github.com/sollimann/bonsai)
behavior-tree library.

## Install

```bash
pip install bonsai-bt
```

The package is published on PyPI as [**`bonsai-bt`**](https://pypi.org/project/bonsai-bt/) and imported as `bonsai_bt`:

```python
import bonsai_bt as bt
```

## Installation (dev)

For building from source — needed if you're contributing to the Rust core or developing against an unpublished version:

```bash
python -m venv .venv
source .venv/bin/activate   # Windows: .venv\Scripts\Activate.ps1
pip install maturin
cd bonsai-py
maturin develop
python -c "import bonsai_bt; print(bonsai_bt.__version__)"
```

## Same BT in Rust and Python

A minimal three-node tree (`Hello → Wait(1.0) → Goodbye`) implemented in both languages. Semantics are identical because the Python package is a thin wrapper around the Rust crate; only the API surface differs (Rust requires an `enum` + explicit types; Python uses any hashable object as the action payload).

### Rust

```rust
use bonsai_bt::{Behavior, Event, Status, UpdateArgs, BT};

#[derive(Clone, Debug)]
enum Greet { Hello, Goodbye }

fn main() {
    let tree = Behavior::sequence(vec![
        Behavior::Action(Greet::Hello),
        Behavior::Wait(1.0),
        Behavior::Action(Greet::Goodbye),
    ]);

    let mut bt: BT<Greet, ()> = BT::new(tree, ());

    for _ in 0..5 {
        let e: Event = UpdateArgs { dt: 0.5 }.into();
        bt.tick(&e, &mut |args, _bb| {
            match *args.action {
                Greet::Hello   => println!("hello"),
                Greet::Goodbye => println!("goodbye"),
            }
            (Status::Success, args.dt)
        });
    }
}
```

### Python

```python
import bonsai_bt as bt

tree = bt.Sequence([
    bt.Action("hello"),
    bt.Wait(1.0),
    bt.Action("goodbye"),
])

tree_bt = bt.BT(tree, None)

def cb(args, _bb):
    print(args.action)
    return (bt.Status.Success, args.dt)

for _ in range(5):
    tree_bt.tick(0.5, cb)
```

Output (both):

    hello
    goodbye

For richer examples — multi-job orchestration, visualizer integration, parallel agents — see [examples/](examples/).

## License

MIT - see [LICENSE](../LICENSE).
