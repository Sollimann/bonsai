# Examples

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
