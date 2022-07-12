# Examples

`cargo build --package examples`

## Boids flocking

Constructing boids flocking behavior by copying the same behavior tree across many agents.
Each agent follows the following rules:

1. Fly towards the center of the swarm
2. Avoid other agents and predators (predator being the mouse cursor)
3. Match the velocity of other agents


`cargo run --bin boids`

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
