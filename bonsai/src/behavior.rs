#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::Float;

/// Describes a behavior.
///
/// This is used for more complex event logic.
/// Can also be used for game AI.
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Behavior<A> {
    /// Waits an amount of time before continuing
    ///
    /// Float: Time in seconds
    Wait(Float),
    /// Wait forever.
    WaitForever,
    /// A high level description of an action.
    ///
    /// An Action can either be "condition" which does not
    /// alter the system and returns either `Success` or `Failure`
    /// - e.g IsDoorOpen? IsNetworkDown?
    ///
    /// Or it can be an "act" that can alter the system
    /// and returns either `Success`, `Failure` or `Running`
    /// - e.g OpenDoor, NetworkShutdown
    Action(A),
    /// Converts `Success` into `Failure` and vice versa.
    Invert(Box<Behavior<A>>),
    /// Ignores failures and returns `Success`.
    AlwaysSucceed(Box<Behavior<A>>),
    /// Runs behaviors one by one until one succeeds.
    ///
    /// Tries the next behavior if one fails. Fails if the last fails.
    /// A short-circuited logical OR gate.
    ///
    /// Resumes the running child across ticks. Use `.memory(false)` to restart
    /// from the first child every tick instead.
    Select(Vec<Behavior<A>>),
    /// `If(condition, success, failure)`
    If(Box<Behavior<A>>, Box<Behavior<A>>, Box<Behavior<A>>),
    /// Runs behaviors one by one until all succeed.
    ///
    /// Fails if any behavior fails. Succeeds if all succeed.
    /// A short-circuited logical AND gate.
    ///
    /// Resumes the running child across ticks. Use `.memory(false)` to restart
    /// from the first child every tick instead.
    Sequence(Vec<Behavior<A>>),
    /// Reactive `Sequence`: re-walks children from the first one every tick.
    /// Built via `Sequence(...).memory(false)`, not constructed directly.
    #[doc(hidden)]
    MemorylessSequence(Vec<Behavior<A>>),
    /// Reactive `Select`: re-walks children from the first one every tick.
    /// Built via `Select(...).memory(false)`, not constructed directly.
    #[doc(hidden)]
    MemorylessSelector(Vec<Behavior<A>>),
    /// Loops while conditional behavior is running.
    ///
    /// Succeeds if the conditional behavior succeeds.
    /// Fails if the conditional behavior fails,
    /// or if any behavior in the loop body fails.
    ///
    /// # Panics
    ///
    /// Panics if the given behavior sequence is empty.
    While(Box<Behavior<A>>, Vec<Behavior<A>>),

    /// Runs a sequence on repeat as long as a conditional behavior
    /// that precedes the sequence is running.
    ///
    /// Conditional behavior is **only** checked before the sequence runs and
    /// not during the sequence.
    ///
    /// Succeeds if the conditional behavior succeeds.
    /// Fails if the conditional behavior fails,
    /// or if any behavior in the sequence fails.
    ///
    /// # Panics
    ///
    /// Panics if the given behavior sequence is empty.
    ///
    ///
    /// ```
    ///
    ///use bonsai_bt::{BT, Running, Failure, Success, Action, UpdateArgs, Behavior::WhileAll, ActionArgs};
    ///use bonsai_bt::Event;
    ///
    ///#[derive(Clone, Debug)]
    ///
    ///enum Ex { A, B, C }
    ///
    ///let rs = WhileAll(
    ///    Box::new(Action(Ex::A)),
    ///    vec![Action(Ex::B), Action(Ex::C)],
    ///);
    ///
    ///let (SUCCESS, FAILURE, RUNNING ) = ((Success, 0.0), (Failure, 0.0), (Running, 0.0));
    ///
    ///let mut bt = BT::new(rs, ());
    ///
    ///let mut i = 0;
    ///let status = bt.tick(&Event::zero_dt_args(), &mut |args: ActionArgs<Event, Ex>, _| {
    ///    match args.action {
    ///        Ex::A => {
    ///            i += 1;
    ///            if i == 4 {
    ///                SUCCESS
    ///            }
    ///            else {
    ///                RUNNING
    ///            }
    ///        }
    ///        Ex::B => {
    ///            i += 1;
    ///            SUCCESS
    ///        }
    ///        Ex::C => {
    ///            i += 1;
    ///            SUCCESS
    ///        }
    ///    }
    ///});
    ///assert!(i == 4);
    /// ```
    WhileAll(Box<Behavior<A>>, Vec<Behavior<A>>),
    /// Runs all behaviors in parallel until all succeeded.
    ///
    /// Succeeds if all behaviors succeed.
    /// Fails is any behavior fails.
    WhenAll(Vec<Behavior<A>>),
    /// Runs all behaviors in parallel until one succeeds.
    ///
    /// Succeeds if one behavior succeeds.
    /// Fails if all behaviors failed.
    WhenAny(Vec<Behavior<A>>),
    /// Runs all behaviors in parallel until all succeeds in sequence.
    ///
    /// Succeeds if all behaviors succeed, but only if succeeding in sequence.
    /// Fails if one behavior fails.
    After(Vec<Behavior<A>>),
    /// Runs all behaviors in parallel until one completes (succeeds or fails).
    ///
    /// Returns the status of the first behavior to complete,
    /// whether that is `Success` or `Failure`.
    /// If all behaviors remain `Running`, returns `Running`.
    Race(Vec<Behavior<A>>),
}

impl<A> Behavior<A> {
    /// Set whether a `Sequence` or `Select` keeps memory across ticks.
    ///
    /// `true` (the default) resumes the running child each tick; `false` restarts
    /// from the first child every tick (reactive). No effect on other node types.
    #[must_use]
    pub fn memory(self, on: bool) -> Self {
        match (self, on) {
            (Behavior::Sequence(c), false) => Behavior::MemorylessSequence(c),
            (Behavior::Select(c), false) => Behavior::MemorylessSelector(c),
            (Behavior::MemorylessSequence(c), true) => Behavior::Sequence(c),
            (Behavior::MemorylessSelector(c), true) => Behavior::Select(c),
            (other, _) => other,
        }
    }
}

#[cfg(test)]
#[cfg(feature = "serde")]
mod tests {
    use crate::{
        Behavior::{self, Action, Select, Sequence, Wait, WaitForever, WhenAny, While},
        Float,
    };

    #[derive(serde::Deserialize, serde::Serialize, Clone, Debug, PartialEq)]
    pub(crate) enum EnemyAction {
        /// Circles forever around target pos.
        Circling,
        /// Waits until player is within distance.
        PlayerWithinDistance(Float),
        /// Fly toward player.
        FlyTowardPlayer,
        /// Waits until player is far away from target.
        PlayerFarAwayFromTarget(Float),
        /// Makes player loose more blood.
        AttackPlayer(Float),
    }

    #[test]
    fn test_create_complex_behavior() {
        let circling = Action(EnemyAction::Circling);
        let circle_until_player_within_distance = Sequence(vec![
            While(Box::new(Wait(5.0)), vec![circling.clone()]),
            While(
                Box::new(Action(EnemyAction::PlayerWithinDistance(50.0))),
                vec![circling],
            ),
        ]);
        let give_up_or_attack = WhenAny(vec![
            Action(EnemyAction::PlayerFarAwayFromTarget(100.0)),
            Sequence(vec![
                Action(EnemyAction::PlayerWithinDistance(10.0)),
                Action(EnemyAction::AttackPlayer(0.1)),
            ]),
        ]);
        let attack_attempt = While(Box::new(give_up_or_attack), vec![Action(EnemyAction::FlyTowardPlayer)]);
        let enemy_behavior = While(
            Box::new(WaitForever),
            vec![circle_until_player_within_distance, attack_attempt],
        );

        let bt_serialized = serde_json::to_string_pretty(&enemy_behavior).unwrap();
        let _bt_deserialized: Behavior<EnemyAction> = serde_json::from_str(&bt_serialized).unwrap();
    }

    #[test]
    fn test_deserialize_behavior() {
        let bt_json = r#"
            {
                "While": ["WaitForever", [{
                    "Sequence": [{
                        "While": [{
                                "Wait": 5.0
                            },
                            [{
                                "Action": "Circling"
                            }]
                        ]
                    }, {
                        "While": [{
                                "Action": {
                                    "PlayerWithinDistance": 50.0
                                }
                            },
                            [{
                                "Action": "Circling"
                            }]
                        ]
                    }]
                }, {
                    "While": [{
                            "WhenAny": [{
                                "Action": {
                                    "PlayerFarAwayFromTarget": 100.0
                                }
                            }, {
                                "Sequence": [{
                                    "Action": {
                                        "PlayerWithinDistance": 10.0
                                    }
                                }, {
                                    "Action": {
                                        "AttackPlayer": 0.1
                                    }
                                }]
                            }]
                        },
                        [{
                            "Action": "FlyTowardPlayer"
                        }]
                    ]
                }]]
            }
        "#;

        let _bt_deserialized: Behavior<EnemyAction> = serde_json::from_str(bt_json).unwrap();
    }

    #[test]
    fn serde_roundtrip_memoryless_sequence() {
        let rs: Behavior<EnemyAction> = Sequence(vec![
            Action(EnemyAction::Circling),
            Action(EnemyAction::FlyTowardPlayer),
        ])
        .memory(false);
        let json = serde_json::to_string(&rs).unwrap();
        assert!(json.contains("MemorylessSequence"));
        let back: Behavior<EnemyAction> = serde_json::from_str(&json).unwrap();
        assert_eq!(rs, back);
    }

    #[test]
    fn serde_roundtrip_memoryless_select() {
        let rs: Behavior<EnemyAction> = Select(vec![
            Action(EnemyAction::Circling),
            Action(EnemyAction::FlyTowardPlayer),
        ])
        .memory(false);
        let json = serde_json::to_string(&rs).unwrap();
        assert!(json.contains("MemorylessSelector"));
        let back: Behavior<EnemyAction> = serde_json::from_str(&json).unwrap();
        assert_eq!(rs, back);
    }

    #[test]
    fn serde_deserializes_tuple_sequence() {
        // The original array form deserializes into the tuple `Sequence` variant.
        let json = r#"{ "Sequence": [{ "Action": "Circling" }] }"#;
        let back: Behavior<EnemyAction> = serde_json::from_str(json).unwrap();
        assert_eq!(back, Sequence(vec![Action(EnemyAction::Circling)]));
    }
}
