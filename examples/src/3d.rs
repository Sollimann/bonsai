#![allow(dead_code, unused_variables, unused_imports)]
use bonsai::Action;
use bonsai::Behavior::{Sequence, Wait, WaitForever, While};
use bonsai::Status::Running;
use bonsai::Timer;
use bonsai::{Event, State, Status::Failure, Status::Success, UpdateArgs};
use kiss3d::window::Window;
use kiss3d::{light::Light, scene::SceneNode};
use na::{Translation3, UnitQuaternion, Vector3};
use nalgebra as na;
use rand::{random, Rng};
use std::thread::sleep;
use std::time::Duration;

/// Underlying numeric type.
pub type Scalar = f32;

/// Animations supported by Sprite
#[derive(Clone, Debug, PartialEq)]
pub enum Animation {
    /// duration, x, y
    ///
    /// Move to specified position, relatively
    MoveBy(f64, Scalar, Scalar),
    /// duration, deg
    ///
    /// Rotate to specified degree, relatively
    RotateBy(f64, Scalar),
    /// duration, sx, sy
    ///
    /// Scale to specified scale, relatively
    ScaleBy(f64, Scalar, Scalar),
    /// duration, times
    Blink(f64, usize),
    /// duration, r, g, b
    ChangeColor(f64, Option<Scalar>, Option<Scalar>, Option<Scalar>),
}

/// This method ticks the behavior tree for a given duration 'dt' to move the
/// behavior tree forward in time. Note that a tick - basically a depth-first traversal
/// - of the tree is intended to return instantly, so it is important that the action
/// callbacks return instantly. Long-running tasks/actions might take many ticks to complete
/// , where you update and monitor the task on a tick-basis.
///
/// The ticks to execute for as long as the specified time 'dt'.
fn tick(c: &mut SceneNode, timer: &mut Timer, state: &mut State<Animation, ()>) {
    // let t = timer.duration_since_start();
    let dt = timer.get_dt();
    let t = dt;
    let e: Event = UpdateArgs { dt }.into();

    #[rustfmt::skip]
    state.event(&e,&mut |args: bonsai::ActionArgs<Event, Animation, ()>| match *args.action {
            Animation::MoveBy(d, x, y) => {
                let trans = Translation3::new(x, y, 0.0);
                // c.append_translation(&trans);
                c.prepend_to_local_translation(&trans);
                if t >= d {
                    (Success, t - d)
                } else {
                    (Running, 0.0)
                }
            }
            Animation::RotateBy(d, rad) => {
                let rot = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), rad);
                c.prepend_to_local_rotation(&rot);
                if t >= d {
                    (Success, t - d)
                } else {
                    (Running, 0.0)
                }
            }
            Animation::ScaleBy(d, sx, sy) => {
                if t >= d {
                    (Success, t - d)
                } else {
                    (Running, 0.0)
                }
            }
            Animation::Blink(d, times) => {
                if t >= d {
                    (Success, t - d)
                } else {
                    (Running, 0.0)
                }
            },
            Animation::ChangeColor(d, r, g, b) => {
                let mut rng = rand::thread_rng();
                let r: f32 = r.unwrap_or_else(|| rng.gen_range(0.0..1.0));
                let g: f32 = g.unwrap_or_else(|| rng.gen_range(0.0..1.0));
                let b: f32 = b.unwrap_or_else(|| rng.gen_range(0.0..1.0));
                c.set_color(r, g, b);
                if t >= d {
                    (Success, t - d)
                } else {
                    (Running, 0.0)
                }
            },
        },
    );

    // c
}

fn main() {
    use crate::Animation::{ChangeColor, MoveBy, RotateBy};
    let mut window = Window::new("Kiss3d: cube");
    let mut c = window.add_cube(0.5, 0.5, 0.5);

    c.set_color(1.0, 0.0, 0.0);
    window.set_light(Light::StickToCamera);

    let rot = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.014);

    let mut timer = Timer::init_time();

    // create BT
    let seq = Sequence(vec![
        Wait(2.0),
        Action(ChangeColor(0.1, Some(0.3), Some(0.3), Some(0.3))),
        Wait(2.0),
        Action(MoveBy(0.0, -0.5, 0.5)),
        Action(MoveBy(0.0, 0.0, -0.5)),
        Action(ChangeColor(0.0, Some(0.2), Some(0.7), Some(0.3))),
        Wait(0.5),
        Action(ChangeColor(0.0, Some(1.0), Some(1.0), Some(1.0))),
        Action(RotateBy(0.0, 0.7)),
        Wait(0.5),
        Action(RotateBy(0.0, 0.0)), // if we add a duration, the by will stop at that stage
        While(
            Box::new(WaitForever),
            vec![
                Action(ChangeColor(0.0, None, None, None)),
                Wait(1.0),
                Action(RotateBy(0.0, 0.014)),
                Wait(2.0),
            ],
        ),
    ]);

    let mut state = State::new(seq);

    while window.render() {
        sleep(Duration::new(0, 0.1e+9 as u32));
        tick(&mut c, &mut timer, &mut state);
    }
}
