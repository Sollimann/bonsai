<h1 align="center" style="font-family:Papyrus; font-size:4em;">Bonsai</h1>
<p align="center">
  <img src="https://github.com/Sollimann/bonsai/blob/main/docs/resources/gifs/bonsai.gif" width="350">
</p>

<p align="center">
    <em>Rust implementation of Behavior Trees</em>
</p>

<!-- [![Build Status](https://github.com/Sollimann/CleanIt/workflows/rust-ci/badge.svg)](https://github.com/Sollimann/CleanIt/actions) -->
<!-- [![codecov](https://codecov.io/gh/Sollimann/CleanIt/branch/main/graph/badge.svg?token=EY3JRZN71M)](https://codecov.io/gh/Sollimann/CleanIt) -->
<!-- [![version](https://img.shields.io/badge/version-1.0.0-blue)](https://GitHub.com/Sollimann/CleanIt/releases/) -->
[![minimum rustc 1.60](https://img.shields.io/badge/rustc-1.60+-blue.svg)](https://rust-lang.github.io/rfcs/2495-min-rust-version.html)
[![Maintenance](https://img.shields.io/badge/Maintained%3F-yes-green.svg)](https://GitHub.com/Sollimann/bonsai/graphs/commit-activity)
[![GitHub pull-requests](https://img.shields.io/github/issues-pr/Sollimann/bonsai.svg)](https://GitHub.com/Sollimann/bonsai/pulls)
[![GitHub pull-requests closed](https://img.shields.io/github/issues-pr-closed/Sollimann/bonsai.svg)](https://GitHub.com/Sollimann/bonsai/pulls)
![ViewCount](https://views.whatilearened.today/views/github/Sollimann/bonsai.svg)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)


## What is a Behavior Tree?

A _Behavior Tree_ (BT) is a data structure in which we can set the rules of how certain _behavior's_ can occur, and the order in which they would execute. BTs are a very efficient way of creating complex systems that are both modular and reactive. These properties are crucial in many applications, which has led to the spread of BT from computer game programming to many branches of AI and Robotics.

Behavior tree fundamentals:

1. **Behavior Trees are trees (duh):** They start at a root node and are designed to be traversed in a specific order until a terminal state is reached (success or failure). The system would run an update from the root called a _tick_. To
guarantee that every node is visited exactly once, the ticks traverse the tree in a depth first traverse.
2. **Leaf nodes are executable behaviors:** Each leaf will do something, whether it’s a simple check or a complex action, and will output a status (success, failure, or running). In other words, leaf nodes are where you connect a BT to the lower-level code for your specific application.
3. **Internal nodes control tree traversal:** The internal (non-leaf) nodes of the tree will accept the resulting status of their children and apply their own rules to dictate which node should be expanded next.

## When to use a Behavior Tree?

* Use BT's to manage complexity when system control logic grows.
* Use BT's if priority ordering of conditions and actions matter. 
* Use BT's when failures can occur and your system would need repeated attempts to complete a task.
* Use BT's when you need parallell semantics. It means that multiple processes can happen at the same time and the logic can be constructed around how these processes runs or terminate.

#### BT vs FSM:

* _BT's has a predictable and intuitive structure._ In comparison _Finite State Machines_ (FSM) can easily become unmanageable as the logic grows.
* _Streamlined logic._ BT's have _one-to-many_ relations between nodes, while FSM's have many-to-many relations.
* _Modular and reasuable components._ In BTs you can create macros of behaviors that can easily be put together to create more complex logic. Conversely, with the FSMs, many of the states are typically tied to that specific context.

### Kanban

Link to project Kanban board
[link](https://github.com/Sollimann/b3/projects/1)