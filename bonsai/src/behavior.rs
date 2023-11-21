/// Describes a behavior.
///
/// This is used for more complex event logic.
/// Can also be used for game AI.
#[derive(Clone, serde::Deserialize, serde::Serialize, PartialEq, Debug)]
pub enum Behavior<A> {
    /// Waits an amount of time before continuing
    ///
    /// f64: Time in seconds
    Wait(f64),
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
    /// Runs behaviors one by one until a behavior succeeds.
    ///
    /// If a behavior fails it will try the next one.
    /// Fails if the last behavior fails.
    /// Can be thought of as a short-circuited logical OR gate.
    Select(Vec<Behavior<A>>),
    /// `If(condition, success, failure)`
    If(Box<Behavior<A>>, Box<Behavior<A>>, Box<Behavior<A>>),
    /// Runs behaviors one by one until all succeeded.
    ///
    /// The sequence fails if a behavior fails.
    /// The sequence succeeds if all the behavior succeeds.
    /// Can be thought of as a short-circuited logical AND gate.
    Sequence(Vec<Behavior<A>>),
    /// Loops while conditional behavior is running.
    ///
    /// Succeeds if the conditional behavior succeeds.
    /// Fails if the conditional behavior fails,
    /// or if any behavior in the loop body fails.
    While(Box<Behavior<A>>, Vec<Behavior<A>>),

    /// Runs a sequence on repeat as long as a conditional behavior
    /// that precedes the sequence is running.
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
    RepeatSequence(Box<Behavior<A>>, Vec<Behavior<A>>),
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
}

#[cfg(test)]
mod tests {
    use crate::Behavior::{self, Action, Sequence, Wait, WaitForever, WhenAny, While};

    #[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
    pub(crate) enum EnemyAction {
        /// Circles forever around target pos.
        Circling,
        /// Waits until player is within distance.
        PlayerWithinDistance(f64),
        /// Fly toward player.
        FlyTowardPlayer,
        /// Waits until player is far away from target.
        PlayerFarAwayFromTarget(f64),
        /// Makes player loose more blood.
        AttackPlayer(f64),
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
}
