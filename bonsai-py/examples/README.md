# bonsai-py examples

Pure-Python examples mirroring `examples/` in the Rust workspace. Each example is a single self-contained `.py` file.

## Prerequisites

Create and activate a Python venv (one-time), then build & install the extension:

```bash
# 1. Create a venv (only needed the first time)
python3 -m venv .venv

# 2. Activate it (every new shell)
source .venv/bin/activate            # macOS / Linux / WSL
# .\.venv\Scripts\Activate.ps1       # Windows PowerShell

# 3. Install build deps + build the extension into the venv
pip install maturin
cd bonsai-py && maturin develop --release && cd ..
```

After that, just `source .venv/bin/activate` + `python bonsai-py/examples/<name>.py` in any new shell.

## Examples (7)

### [simple_npc_ai.py](simple_npc_ai.py) — console NPC
NPC runs and shoots until action points are exhausted, then rests and dies. Demonstrates `WhileAll`, blackboard mutation via `@dataclass`, structural-`match` callback.

```bash
python bonsai-py/examples/simple_npc_ai.py
```

### [race_timeout.py](race_timeout.py) — `Race` between work and timeout
A simulated long-running job (random 200–1200 ms on a `threading.Thread`) races a 600 ms timeout. The callback polls the work's `queue.Queue` non-blockingly. Demonstrates `Race`, asyncio main loop + threading worker, the unsendable-BT constraint.

```bash
python bonsai-py/examples/race_timeout.py
```

### [graphviz_demo.py](graphviz_demo.py) — tree visualization
Builds an attack-drone tree (mix of plain-string and `@dataclass(frozen=True)` payload actions) and prints the graphviz DOT representation. Paste the output into <https://dreampuf.github.io/GraphvizOnline/> to render it.

```bash
python bonsai-py/examples/graphviz_demo.py
python bonsai-py/examples/graphviz_demo.py > tree.dot
```

### [visualizer_smoke.py](visualizer_smoke.py) — live web visualizer
Drives a deliberately rich 27-node tree at ~400 ms/tick with a 5-step status rotation and per-leaf phase offset; the browser shows continuous color animation. Demonstrates `BT.with_telemetry(port)`, `reset_bt()`, and every major factory.

```bash
python bonsai-py/examples/visualizer_smoke.py
```

Then open <http://127.0.0.1:8910/> in a browser. `Ctrl-C` to stop.

### [boids_console.py](boids_console.py) — shared BT across N agents
Builds **one** `Behavior` tree and binds it to 10 independent `BT` instances (each with its own `Boid` dataclass blackboard). Updates positions every tick for 30 frames. Demonstrates the shared-subtree pattern, real-time-loop dt, `WhenAll` for parallel updates.

```bash
python bonsai-py/examples/boids_console.py
```

### [threaded_drone.py](threaded_drone.py) — multi-job mission (threading)
Drone mission: takeoff → check battery → fly (or fall back to land) → land → repeat. Each long-running step runs on a background `threading.Thread`; the BT polls per-job `queue.Queue`s. Prints the tree's `graphviz()` at the start, then runs the mission for ~8 seconds. **Pick this variant when actions block on hardware or sync IO** — blocking syscalls, vendor SDKs without an async API, `subprocess.run`, etc.

```bash
python bonsai-py/examples/threaded_drone.py
```

### [async_drone.py](async_drone.py) — multi-job mission (asyncio)
Same mission and tree as `threaded_drone.py`, but background jobs are `async def` coroutines on a single asyncio event loop, communicating via `asyncio.Queue`. **Pick this variant when actions are awaitable** (`aiohttp`, async DB drivers, websockets) — N-way concurrency without OS thread overhead. 

```bash
python bonsai-py/examples/async_drone.py
```
