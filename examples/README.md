# Examples

The examples below are minimal, made-up scenarios to help you get going. For real-world usage of `bonsai-bt` in the wild, see:

* [utahrobotics/utah-lunabotics-2026](https://github.com/utahrobotics/utah-lunabotics-2026) — orchestrates core autonomy on a lunar rover. The team placed 3rd out of 50 in the autonomy challenge at [NASA Lunabotics 2026](https://www.nasa.gov/learning-resources/lunabotics-challenge/).
* [catornot/bp-ort](https://github.com/catornot/bp-ort) — NPC plugins for Titanfall 2 mod servers (Northstar). [Live demo](https://www.youtube.com/watch?v=uT3aAuB4ej4&t=60s).
* [AnotherlandServer/anotherland](https://github.com/AnotherlandServer/anotherland) — NPC behaviors for the Anotherland MMORPG.

`cargo build --package examples`

## Game NPC AI console application

Demonstrates use of a behavior tree in minimal and easy to follow console application setting where a fictional
non-playing game character
updates its AI state. Run and inspect this example if you want to get a quick introduction on how behavior tree can be
used in an application.

`cargo run --bin simple_npc_ai`

## Boids flocking

Constructing boids flocking behavior by copying the same behavior tree across many agents.
Each agent follows the following rules:

1. Fly towards the center of the swarm
2. Avoid other agents and predators (predator being the mouse cursor)
3. Match the velocity of other agents


`cargo run --bin boids`

*PS!* if for some how you're unable to build and run the example, it is most likely because you're lacking some dependencies. Try installing the following dependencies:

`sudo apt-get update && sudo apt-get install libudev-dev pkg-config librust-alsa-sys-dev`

<p align="center">
  <img src="https://github.com/Sollimann/bonsai/blob/main/docs/resources/images/boids.png" width="700">
</p>

## 3d

This is basically a really chaotic 3d animation intended to show how you can create a reactive and
responsive animations including behaviors such as shape-shifting, color changes, mouse callback, object
rotation and translation and timers.

`cargo run --bin 3d`

<p align="center">
  <img src="https://github.com/Sollimann/bonsai/blob/main/docs/resources/images/3d.png" width="700">
</p>

## Async drone (example for long-running jobs)

This is an example to simulate behavior of a drone, where some of the task called in the behavior tree are long-running and must be conducted in background threads to avoid
blocking the execution of the tree.

The simulated drone will first take off, then if the batteries allow it fly to a goal point and then land. If the batteries are too low, the drone will fly back to the docking station. Finally the drone will land. All of this is done while collision avoidance is running in parallell.

`cargo run --bin async_drone`

<p align="center">
  <img src="https://github.com/Sollimann/bonsai/blob/main/docs/resources/images/async_drone.png" width="700">
</p>

## Show BT in graphviz

Compile the behavior tree into a [graphviz](https://graphviz.org/) compatible [DiGraph](https://docs.rs/petgraph/latest/petgraph/graph/type.DiGraph.html).

`cargo run --bin graphviz`

<p align="center">
  <img src="https://github.com/Sollimann/bonsai/blob/main/docs/resources/images/attack_drone.png" width="700">
</p>

## Behavior Timeout (example for Race behavior and long-running jobs)

This simple example shows an example of using the Race behavior to time out a long-running job that takes a random amount of time to complete.

`cargo run --bin race_timeout`

<p align="center">
  <img src="https://github.com/Sollimann/bonsai/blob/main/docs/resources/images/race_timeout.png" width="700">
</p>

## WebSocket visualizer (live tree inspector)

A live web-based visualizer for a running behavior tree. The example builds a 27-node tree, enabling the visualizer via a single API call `BT::with_telemetry(8910)`, and re-ticks every ~400 ms so leaf statuses (green / yellow / red) and the running-path highlight animate continuously.

To see it in example, run the following command:

`cargo run --bin visualizer_smoke`

Then open <http://127.0.0.1:8910/> in a browser. The tree renders within ~1 s and the status bar reads `connected` / `27 nodes`. `Ctrl-C` and restart — the page reconnects within ≤ 1 s. See the module-level doc comment at [src/visualizer_smoke/main.rs](src/visualizer_smoke/main.rs) for the full DFS tree layout and the per-leaf status-cycle rationale.

<p align="center">
  <img src="https://github.com/Sollimann/bonsai/blob/main/docs/resources/gifs/live_visualizer_example.gif" width="1000" ">
</p>
