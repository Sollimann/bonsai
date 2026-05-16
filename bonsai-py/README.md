# bonsai-py - Python bindings

Python bindings for the [bonsai-bt](https://github.com/sollimann/bonsai)
behavior-tree library.

## Installation (dev)

```bash
python -m venv .venv
source .venv/bin/activate   # Windows: .venv\Scripts\Activate.ps1
pip install maturin
cd bonsai-py
maturin develop
python -c "import bonsai_py; print(bonsai_py.__version__)"
```

## License

MIT - see [LICENSE](../LICENSE).
