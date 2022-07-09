#![allow(dead_code, unused_variables, unused_imports)]
use bonsai::Behavior::{Sequence, Wait, WaitForever, While};
use bonsai::Status::Running;
use bonsai::{Action, RUNNING};
use bonsai::{Event, State, Status::Failure, Status::Success, UpdateArgs};
use bonsai::{Timer, BT};
use kiss3d::window::Window;
use kiss3d::{light::Light, scene::SceneNode};
use na::{Translation3, UnitQuaternion, Vector3};
use nalgebra as na;
use rand::{random, Rng};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

/// Underlying numeric type.
pub type Scalar = f32;

/// Animations supported by Sprite
#[derive(Clone, Debug, PartialEq)]
pub enum Animation {
    ComplexCondition(u32),
    /// x, y
    ///
    /// Move to specified position, relatively
    MoveBy(Scalar, Scalar),
    /// deg
    ///
    /// Rotate to specified degree, relatively
    RotateBy(Scalar),
    /// sx, sy
    ///
    /// Scale to specified scale, relatively
    ScaleBy(Scalar, Scalar),
    /// times
    Blink(usize),
    /// r, g, b
    ChangeColor(Option<Scalar>, Option<Scalar>, Option<Scalar>),
}

/// This method ticks the behavior tree for a given duration 'dt' to move the
/// behavior tree forward in time. Note that a tick - basically a depth-first traversal
/// - of the tree is intended to return instantly, so it is important that the action
/// callbacks return instantly. Long-running tasks/actions might take many ticks to complete
/// , where you update and monitor the task on a tick-basis.
///
/// The ticks to execute for as long as the specified time 'dt'.
fn tick(c: &mut SceneNode, timer: &mut Timer, bt: &mut BT<Animation, String, u32>) {
    // let t = timer.duration_since_start();
    let dt = timer.get_dt();
    let e: Event = UpdateArgs { dt }.into();

    // get data from blackboard
    let inc = *bt.get_blackboard().get_db().get("count").unwrap();

    // update state of behaviosuccessr tree
    #[rustfmt::skip]
    bt.state.event(&e,&mut |args: bonsai::ActionArgs<Event, Animation>|
        match *args.action {
            // this is just some random complex conditional statements
            Animation::ComplexCondition(v) => {
                println!("inc {}", inc);
                if inc < v {
                    println!("inc < {}", v);
                    println!("success");
                    (Success, args.dt)
                }
                else if inc > 250 && inc < 350 {
                    println!("350 > inc > 250");
                    println!("running");
                    RUNNING
                }
                else if inc > 200 {
                    println!("inc > 200");
                    println!("success");
                    (Success, args.dt)
                } else {
                    println!("failure");
                    (Failure, args.dt)
                }
            }
            Animation::MoveBy(x, y) => {
                let trans = Translation3::new(x, y, 0.0);
                // c.append_translation(&trans);
                c.prepend_to_local_translation(&trans);
                // (Success, dt)
                RUNNING
            }
            Animation::RotateBy(rad) => {
                let rot = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), rad);
                c.prepend_to_local_rotation(&rot);
                // RUNNING
                (Success, dt)
            }
            Animation::ScaleBy(sx, sy) => {
                RUNNING
            }
            Animation::Blink(times) => {
                (Success, dt)
            },
            Animation::ChangeColor(r, g, b) => {
                let mut rng = rand::thread_rng();
                let r: f32 = r.unwrap_or_else(|| rng.gen_range(0.0..1.0));
                let g: f32 = g.unwrap_or_else(|| rng.gen_range(0.0..1.0));
                let b: f32 = b.unwrap_or_else(|| rng.gen_range(0.0..1.0));
                c.set_color(r, g, b);

                (Success, dt)
            },

        },
    );

    // update blackboard
    let bb = bt.get_blackboard();
    let count = bb
        .get_db()
        .entry("count".to_string())
        .and_modify(|count| *count += 1)
        .or_insert(0)
        .to_owned();
}

fn main() {
    use crate::Animation::{ChangeColor, ComplexCondition, MoveBy, RotateBy};
    let mut window = Window::new("Kiss3d: cube");
    let mut c = window.add_cube(0.5, 0.5, 0.5);

    c.set_color(1.0, 0.0, 0.0);
    window.set_light(Light::StickToCamera);

    let rot = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.014);

    let mut timer = Timer::init_time();

    // create BT
    let seq = Sequence(vec![
        Wait(2.0),
        Action(ChangeColor(Some(0.3), Some(0.3), Some(0.3))),
        Wait(2.0),
        Action(MoveBy(-0.5, 0.5)),
        Action(MoveBy(0.0, -0.5)),
        Action(ChangeColor(Some(0.2), Some(0.7), Some(0.3))),
        Wait(0.5),
        Action(ChangeColor(Some(1.0), Some(1.0), Some(1.0))),
        Action(RotateBy(0.7)),
        Wait(0.5),
        Action(RotateBy(0.0)), // if we add a duration, the by will stop at that stage
        While(
            Box::new(WaitForever),
            vec![
                Action(ChangeColor(None, None, None)),
                Wait(1.0),
                Action(RotateBy(0.014)),
                Wait(2.0),
            ],
        ),
    ]);

    let seq2 = While(
        Box::new(WaitForever),
        vec![
            Action(ChangeColor(None, None, None)),
            // if ComplexCondition action succeeds, sequence will proceed
            // else if it fails, the sequence will restart from beginning
            Action(ComplexCondition(100)),
            Action(RotateBy(0.054)),
            Wait(0.2),
        ],
    );

    let mut h: HashMap<String, u32> = HashMap::new();
    h.insert("count".to_string(), 1);
    let mut bt = BT::new(seq2, h);

    while window.render() {
        sleep(Duration::new(0, 0.1e+9 as u32));
        tick(&mut c, &mut timer, &mut bt);
    }
}
