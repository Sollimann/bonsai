#![allow(dead_code, unused_variables, unused_imports)]
use bonsai::{Event, State, Status::Failure, Status::Success, UpdateArgs};
use kiss3d::window::Window;
use kiss3d::{light::Light, scene::SceneNode};
use na::{UnitQuaternion, Vector3};
use nalgebra as na;

/// Underlying numeric type.
pub type Scalar = f32;

/// Animations supported by Sprite
#[derive(Clone, PartialEq)]
pub enum Animation {
    /// duration, x, y
    ///
    /// Move to specified position
    MoveTo(f64, Scalar, Scalar),
    /// duration, x, y
    ///
    /// Move to specified position, relatively
    MoveBy(f64, Scalar, Scalar),
    /// duration, deg
    ///
    /// Rotate to specified degree
    RotateTo(f64, Scalar),
    /// duration, deg
    ///
    /// Rotate to specified degree, relatively
    RotateBy(f64, Scalar),
    /// duration, sx, sy
    ///
    /// Scale to specified scale
    ScaleTo(f64, Scalar, Scalar),
    /// duration, sx, sy
    ///
    /// Scale to specified scale, relatively
    ScaleBy(f64, Scalar, Scalar),
    /// duration, times
    Blink(f64, usize),
    /// duration, r, g, b
    ChangeColor(f64, Scalar, Scalar, Scalar),
}

// A test state machine that can increment and decrement.
fn tick(_c: &SceneNode, dt: f64, state: &mut State<Animation, ()>) {
    let e: Event = UpdateArgs { dt }.into();

    state.event(&e, &mut |args| match *args.action {
        Animation::MoveTo(duration, x, y) => {
            todo!()
        }
        Animation::MoveBy(duration, x, y) => {
            todo!()
        }
        Animation::RotateTo(duration, rad) => {
            todo!()
        }
        Animation::RotateBy(duration, rad) => {
            todo!()
        }
        Animation::ScaleTo(duration, sx, sy) => {
            todo!()
        }
        Animation::ScaleBy(duration, sx, sy) => {
            todo!()
        }
        Animation::Blink(duration, times) => todo!(),
        Animation::ChangeColor(duration, r, g, b) => todo!(),
    });

    // c
}

fn main() {
    let mut window = Window::new("Kiss3d: cube");
    let mut c = window.add_cube(1.0, 1.0, 1.0);

    c.set_color(1.0, 0.0, 0.0);
    window.set_light(Light::StickToCamera);

    let rot = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.014);

    while window.render() {
        // tick()
        c.prepend_to_local_rotation(&rot);
    }
}
