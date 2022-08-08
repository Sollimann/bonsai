<h1 align="left" style="font-family:Papyrus; font-size:4em;"> Contents </h1>

- [Fundamentals](#fundamentals)
  - [What is a Behavior Tree?](#what-is-a-behavior-tree)
  - [When to use a Behavior Tree?](#when-to-use-a-behavior-tree)
    - [BT vs FSM:](#bt-vs-fsm)
  - [How to use a Behavior tree?](#how-to-use-a-behavior-tree)
- [Parallel semantics](#parallel-semantics)
- [Behavior vs State](#behavior-vs-state)
- [Events](#events)
- [Instant Actions](#instant-actions)

## Fundamentals

### What is a Behavior Tree?

A _Behavior Tree_ (BT) is a data structure in which we can set the rules of how certain _behavior's_ can occur, and the order in which they would execute. BTs are a very efficient way of creating complex systems that are both modular and reactive. These properties are crucial in many applications, which has led to the spread of BT from computer game programming to many branches of AI and Robotics.

Behavior tree fundamentals:

1. **Behavior Trees are trees (duh):** They start at a root node and are designed to be traversed in a specific order until a terminal state is reached (success or failure). The system would run an update from the root called a _tick_. For each execution tick the tree is traversed from the root in depth-first Search/Traverse (DFS) from left. In DFS, you go as deep as possible down one path before backing up and trying a different one. DFS is like walking through a maze. You explore one path, hit a dead end, and go back and try a different one.
2. **Prioritized ordering of behaviors:**  The main power of BT's comes from their ability to represent multiple different courses of action, in order of priority from most favorable to least favorable (from left to right in the tree), and to return success if it managed to succeed at any course of action. A lower priority task should be preempted (meaning, one task taking over in place of
another) if a higher-priority task requires the same resources.
3. **Leaf nodes are executable behaviors:** Each leaf will do something, whether it’s a simple check or a complex action, and will output a status (success, failure, or running). In other words, leaf nodes are where you connect a BT to the lower-level code for your specific application.
4. **Internal nodes control tree traversal:** The internal (non-leaf) nodes of the tree will accept the resulting status of their children and apply their own rules to dictate which node should be expanded next.
5. **Task Tracking through working memory:** Behavior trees keep track of progress on tasks by maintaining a working memory (often referred to as a *Blackboard* or *Cache*) that holds variables and values relevant to the BT. Working memory is important for reasoning and the guidance of decision-making and behavior, but is then discarded once it has served its purpose. The simplest forms of working memory that we utilize in programming are boolean parameters to tell if an event has happened or not, counters to tell how many times an event has happened and internal clocks (timers) to keep track of occurrence of events through time. Drawing a parallel to human biology, working memory is what allows you to remember what phone number the operator told you, just long enough to dial it.
6. **Behavior Trees can have parallell semantics:** This library has parallel semantics for AI behavior trees. It means that multiple processes can happen at the same time and the logic can be constructed around how these processes runs or terminate. A property of BT's with parallel semantics is that you can control termination conditions externally, as opposed to most programming languages where termination condition is controlled internally.


### When to use a Behavior Tree?

* Use BT's to manage complexity when system control logic grows.
* Use BT's if priority ordering of conditions and actions matter.
* Use BT's when failures can occur and your system would need repeated attempts to complete a task.
* Use BT's when you need parallell semantics. It means that multiple processes can happen at the same time and the logic can be constructed around how these processes runs or terminate.

#### BT vs FSM:

* _BT's has a predictable and intuitive structure._ In comparison _Finite State Machines_ (FSM) can easily become unmanageable as the logic grows.
* _Streamlined logic._ BT's have _one-to-many_ relations between nodes, while FSM's have many-to-many relations.
* _Modular and reasuable components._ In BTs you can create macros of behaviors that can easily be put together to create more complex logic. Conversely, with the FSMs, many of the states are typically tied to that specific context.

### How to use a Behavior tree?

An AI behavior tree is a very generic way of organizing interactive logic.
It has built-in semantics for processes that signals `Running`, `Success` or
`Failure`.

For example, if you have a state `A` and a state `B`:

- Move from state `A` to state `B` if `A` succeeds: `Sequence([A, B])`
- Try `A` first and then try `B` if `A` fails: `Select([A, B])`
- If `condition` succeedes do `A`, else do `B` : `If(condition, A, B)`
- If `A` succeeds, return failure (and vice-versa): `Invert(A)`
- Do `B` repeatedly while `A` runs: `While(A, [B])`
- Do `A`, `B` forever: `While(WaitForever, [A, B])`
- Run `A` and `B` in parallell and wait for both to succeed: `WhenAll([A, B])`
- Run `A` and `B` in parallell and wait for any to succeed: `WhenAny([A, B])`
- Run `A` and `B` in parallell, but `A` has to succeed before `B`: `After([A, B])`

See the `Behavior` enum for more information.

## Parallel semantics
Parallel semantics has two important properties:

* Makes it easier to create more complex behavior from simpler ones
* Infinite loops of behavior can be terminated based on external conditions

This semantics solves the problem of scaling up building blocks of behavior to any arbitrary size. Such trees can be put together in many complex ways, resulting in many complex states.

The building blocks used in this library has been crafted to cover as many parallel scenarios as possible using common sense as guide.

One effect of parallel semantics is that it can't be simulated perfectly on a single thread. The behavior is deterministic, but the logic breaks down for shorter time intervals than given by updates per second. The consequences are determined by the interaction of side effects.

For example, in a racing game, `WhenAny` can be used to detect when there is a winner, keeping track of a process for each car. The first car in the list might trigger `Success` even if the second car logically should come first, but only if the first car completes within a delta time interval. However, this will have no logical consequences if the physical simulation runs separately and the winner is picked based on who actually crossed the finish line first.

## Behavior vs State

For each behavior there is a state that keeps track of current running process. When you declare a behavior, this state is not included, resulting in a compact representation that can be copied or shared between objects having same behavior. Behavior means the declarative representation of the behavior, and State represents the executing instance of that behavior.

## Events

The Bonsai behavior tree models the world in terms of an discretized event loop where updates comes with a *delta time interval* `dt`. When an process terminates, it tells how much time is left of the delta time interval, such that the next process can be executed for the remaining time.

Events are partially consumable. When one action terminates, it can pass on the remaining delta time to the next action.

## Instant Actions

Update actions that does not consume delta time, returning the same delta time as they receive, can lead to infinite loops. A `Wait` behavior can be used to prevent this. The meaning of update is defined as "consume time to stop" so it will continue running actions until it hits one that does not have enough time to terminate.
