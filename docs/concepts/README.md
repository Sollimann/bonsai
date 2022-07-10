
# Concepts

- [Concepts](#concepts)
  - [Fundamentals](#fundamentals)
    - [What is a Behavior Tree?](#what-is-a-behavior-tree)
    - [When to use a Behavior Tree?](#when-to-use-a-behavior-tree)
      - [BT vs FSM:](#bt-vs-fsm)
    - [How to use a Behavior tree?](#how-to-use-a-behavior-tree)
  - [Types of Nodes](#types-of-nodes)
  - [Understand Asynchrous Nodes, Concurrency and Parallelism](#understand-asynchrous-nodes-concurrency-and-parallelism)

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
- Do `B` repeatedly while `A` runs: `While(A, [B])`
- Do `A`, `B` forever: `While(WaitForever, [A, B])`
- Wait for both `A` and `B` to complete: `WhenAll([A, B])`
- Wait for either `A` or `B` to complete: `WhenAny([A, B])`

See the `Behavior` enum for more information.


## Types of Nodes

bla bla


## Understand Asynchrous Nodes, Concurrency and Parallelism

bla bla
