# Examples

`cargo build --package examples`

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
  <img src="https://github.com/Sollimann/bonsai/blob/main/docs/resources/images/boids.png" width="700" ">
</p>

## 3d

This is basically a really chaotic 3d animation intended to show how you can create a reactive and
responsive animations including behaviors such as shape-shifting, color changes, mouse callback, object
rotation and translation and timers.

`cargo run --bin 3d`

<p align="center">
  <img src="https://github.com/Sollimann/bonsai/blob/main/docs/resources/images/3d.png" width="700" ">
</p>

## Show BT in graphviz

Compile the behavior tree into a [graphviz](https://graphviz.org/) compatible [DiGraph](https://docs.rs/petgraph/latest/petgraph/graph/type.DiGraph.html).

`cargo run --bin graphviz`

<p align="center">
  <img src="https://github.com/Sollimann/bonsai/blob/main/docs/resources/images/attack_drone.png" width="700" ">
</p>

## NodeEditor

*NOTE!* If you're using WSL2 and get the following error `"ìnternal error: entered unreachable code"`, then install the following `sudo apt install libxkbcommon-dev libegl1 libwayland-dev`

To run example:

```sh
$ cargo run --bin imnodes
```
