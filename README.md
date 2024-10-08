<h1 align="center" style="font-family:Papyrus; font-size:4em;"> Bonsai 盆栽 </h1>
<p align="center">
  <img src="https://github.com/Sollimann/bonsai/blob/main/docs/resources/gifs/bonsai.gif" width="350" ">
</p>

<p align="center">
    <em>Rust implementation of Behavior Trees</em>
</p>

<!-- [![version](https://img.shields.io/badge/version-1.0.0-blue)](https://GitHub.com/Sollimann/CleanIt/releases/) -->
[![Build Status](https://github.com/Sollimann/bonsai/actions/workflows/rust-ci.yml/badge.svg)](https://github.com/Sollimann/bonsai/actions)
[![Bonsai crate](https://img.shields.io/crates/v/bonsai-bt.svg)](https://crates.io/crates/bonsai-bt)
[![minimum rustc 1.72](https://img.shields.io/badge/rustc-1.56+-blue.svg)](https://rust-lang.github.io/rfcs/2495-min-rust-version.html)
[![Docs](https://docs.rs/bonsai-bt/badge.svg)](https://docs.rs/bonsai-bt)
[![codecov](https://codecov.io/gh/Sollimann/bonsai/branch/main/graph/badge.svg?token=JX8JBPWORV)](https://codecov.io/gh/Sollimann/bonsai)
[![Maintenance](https://img.shields.io/badge/Maintained%3F-yes-green.svg)](https://GitHub.com/Sollimann/bonsai/graphs/commit-activity)
[![GitHub pull-requests](https://img.shields.io/github/issues-pr/Sollimann/bonsai.svg)](https://GitHub.com/Sollimann/bonsai/pulls)
[![GitHub pull-requests closed](https://img.shields.io/github/issues-pr-closed/Sollimann/bonsai.svg)](https://GitHub.com/Sollimann/bonsai/pulls)
![ViewCount](https://views.whatilearened.today/views/github/Sollimann/bonsai.svg)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Contents

* [Quick intro to Behavior Trees](https://www.youtube.com/watch?v=KeShMInMjro)
* [Concepts](docs/concepts/README.md)
* [Examples](examples/README.md)
* [Development Guide](DEVELOPMENT.md)
* [Kanban Board](https://github.com/Sollimann/b3/projects/1)
* [Honorable Mentions](#similar-crates)

## Using Bonsai
Bonsai is available on crates.io. The recommended way to use it is to add a line into your Cargo.toml such as:

```toml
[dependencies]
bonsai-bt = "*"
```

## What is a Behavior Tree?

A _Behavior Tree_ (BT) is a data structure in which we can set the rules of how certain _behavior's_ can occur, and the order in which they would execute. BTs are a very efficient way of creating complex systems that are both modular and reactive. These properties are crucial in many applications, which has led to the spread of BT from computer game programming to many branches of AI and Robotics.

### How to use a Behavior tree?

A Behavior Tree forms a tree structure where each node represents a process. When the process terminates, it signals `Success` or `Failure`. This can then be used by the parent node to select the next process. A signal `Running` is used to tell the process is not done yet.

For example, if you have a state `A` and a state `B`:

- Move from state `A` to state `B` if `A` succeeds: `Sequence([A, B])`
- Try `A` first and then try `B` if `A` fails: `Select([A, B])`
- If `condition` succeedes do `A`, else do `B` : `If(condition, A, B)`
- If `A` succeeds, return failure (and vice-versa): `Invert(A)`
- Do `A`, `B` repeatedly while `LoopCondition` runs: `While(LoopCondition, [A, B])`. Checks condition node between nodes `A`, `B`.
- Do `A`, `B` forever: `While(WaitForever, [A, B])`
- Do `A`, `B` repeatedly while `LoopCondition` runs: `WhileAll(LoopCondition, [A, B])`. After *All* nodes `A`, `B` are completed successfully, check the condition node.
- Run `A` and `B` in parallell and wait for both to succeed: `WhenAll([A, B])`
- Run `A` and `B` in parallell and wait for any to succeed: `WhenAny([A, B])`
- Run `A` and `B` in parallell, but `A` has to succeed before `B`: `After([A, B])`

See the `Behavior` enum for more information.

### Calling long-running tasks in behavior tree

To make sure that the behavior tree is always responsive, it is important that the actions that are created executes instantly so that they do not block the tree traversal. If you have long-running tasks/functions that can take seconds or minutes to execute - either `async` or `sync` - then we can dispatch those jobs into background threads, and get status of the task through a channel.

see *async drone* example in the `/examples` folder for more details.

## Example of use

See [Examples](examples/README.md) folder.

## Similar Crates

Bonsai is inspired by many other crates out there, here's a few worth mentioning:

* [ai_behavior](https://github.com/PistonDevelopers/ai_behavior) (bonsai is a continuation of this crate)
* [aspen](https://gitlab.com/neachdainn/aspen)
* [behavior-tree](https://github.com/darthdeus/behavior-tree)
* [stackbt](https://github.com/eaglgenes101/stackbt)
