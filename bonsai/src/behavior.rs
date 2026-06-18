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
    /// `memory = true` (default) resumes the running child across ticks.
    /// `memory = false` restarts from the first child every tick, so an earlier
    /// child can preempt a later running one. See [`Behavior::Sequence`].
    Select {
        /// Children, tried in order.
        children: Vec<Behavior<A>>,
        /// Resume the running child across ticks (`true`) or restart from the
        /// first child every tick (`false`). Defaults to `true`.
        #[cfg_attr(feature = "serde", serde(default = "default_true"))]
        memory: bool,
    },
    /// `If(condition, success, failure)`
    If(Box<Behavior<A>>, Box<Behavior<A>>, Box<Behavior<A>>),
    /// Runs behaviors one by one until all succeed.
    ///
    /// Fails if any behavior fails. Succeeds if all succeed.
    /// A short-circuited logical AND gate.
    ///
    /// `memory = true` (default) resumes the running child across ticks.
    /// `memory = false` restarts from the first child every tick, so an earlier
    /// condition child can abort a later running child when its check flips.
    ///
    /// With `memory = false`, leaf children tick with no heap allocation; a
    /// decorator or nested composite child costs O(subtree size) per tick.
    Sequence {
        /// Children, run in order.
        children: Vec<Behavior<A>>,
        /// Resume the running child across ticks (`true`) or restart from the
        /// first child every tick (`false`). Defaults to `true`.
        #[cfg_attr(feature = "serde", serde(default = "default_true"))]
        memory: bool,
    },
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

/// Serde default for the `memory` flag: trees without it deserialize as `true`.
#[cfg(feature = "serde")]
fn default_true() -> bool {
    true
}

impl<A> Behavior<A> {
    /// A [`Behavior::Sequence`] that resumes the running child across ticks.
    pub fn sequence(children: Vec<Behavior<A>>) -> Self {
        Behavior::Sequence { children, memory: true }
    }

    /// A [`Behavior::Sequence`] that restarts from the first child every tick.
    pub fn memoryless_sequence(children: Vec<Behavior<A>>) -> Self {
        Behavior::Sequence {
            children,
            memory: false,
        }
    }

    /// A [`Behavior::Select`] that resumes the running child across ticks.
    pub fn select(children: Vec<Behavior<A>>) -> Self {
        Behavior::Select { children, memory: true }
    }

    /// A [`Behavior::Select`] that restarts from the first child every tick.
    pub fn memoryless_selector(children: Vec<Behavior<A>>) -> Self {
        Behavior::Select {
            children,
            memory: false,
        }
    }
}

#[cfg(test)]
#[cfg(feature = "serde")]
mod tests {
    use crate::{
        Behavior::{self, Action, Wait, WaitForever, WhenAny, While},
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
        let circle_until_player_within_distance = Behavior::sequence(vec![
            While(Box::new(Wait(5.0)), vec![circling.clone()]),
            While(
                Box::new(Action(EnemyAction::PlayerWithinDistance(50.0))),
                vec![circling],
            ),
        ]);
        let give_up_or_attack = WhenAny(vec![
            Action(EnemyAction::PlayerFarAwayFromTarget(100.0)),
            Behavior::sequence(vec![
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
        // `memory` is intentionally omitted on both `Sequence` nodes to exercise
        // the serde default (`memory = true`).
        let bt_json = r#"
            {
                "While": ["WaitForever", [{
                    "Sequence": { "children": [{
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
                    }] }
                }, {
                    "While": [{
                            "WhenAny": [{
                                "Action": {
                                    "PlayerFarAwayFromTarget": 100.0
                                }
                            }, {
                                "Sequence": { "children": [{
                                    "Action": {
                                        "PlayerWithinDistance": 10.0
                                    }
                                }, {
                                    "Action": {
                                        "AttackPlayer": 0.1
                                    }
                                }] }
                            }]
                        },
                        [{
                            "Action": "FlyTowardPlayer"
                        }]
                    ]
                }]]
            }
        "#;

        let deserialized: Behavior<EnemyAction> = serde_json::from_str(bt_json).unwrap();
        // The omitted `memory` flags must default to `true`.
        assert!(matches!(deserialized, Behavior::While(..)));
    }

    #[test]
    fn serde_roundtrip_memoryless_sequence() {
        let rs: Behavior<EnemyAction> = Behavior::memoryless_sequence(vec![
            Action(EnemyAction::Circling),
            Action(EnemyAction::FlyTowardPlayer),
        ]);
        let json = serde_json::to_string(&rs).unwrap();
        assert!(json.contains("Sequence"));
        // The memoryless flag must survive serialization.
        assert!(json.contains("\"memory\":false"));
        let back: Behavior<EnemyAction> = serde_json::from_str(&json).unwrap();
        assert_eq!(rs, back);
    }

    #[test]
    fn serde_roundtrip_memoryless_select() {
        let rs: Behavior<EnemyAction> = Behavior::memoryless_selector(vec![
            Action(EnemyAction::Circling),
            Action(EnemyAction::FlyTowardPlayer),
        ]);
        let json = serde_json::to_string(&rs).unwrap();
        assert!(json.contains("Select"));
        assert!(json.contains("\"memory\":false"));
        let back: Behavior<EnemyAction> = serde_json::from_str(&json).unwrap();
        assert_eq!(rs, back);
    }

    #[test]
    fn serde_default_memory_is_true() {
        // A `Sequence` serialized without the `memory` field deserializes as
        // `memory = true` (backward compatibility with pre-flag trees).
        let json = r#"{ "Sequence": { "children": [{ "Action": "Circling" }] } }"#;
        let back: Behavior<EnemyAction> = serde_json::from_str(json).unwrap();
        assert_eq!(back, Behavior::sequence(vec![Action(EnemyAction::Circling)]));
    }
}
