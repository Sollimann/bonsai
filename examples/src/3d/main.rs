use bonsai_bt::Behavior::{If, Invert, Wait, WhenAny, While};
use bonsai_bt::Status::{self};
use bonsai_bt::{Action, RUNNING};
use bonsai_bt::{Event, Status::Failure, Status::Success, UpdateArgs};
use bonsai_bt::{Timer, BT};
use kiss3d::event::EventManager;
use kiss3d::text::Font;
use kiss3d::window::Window;
use kiss3d::{light::Light, scene::SceneNode};
use na::{Point2, Point3, Translation3, UnitQuaternion, Vector3};
use nalgebra as na;
use rand::Rng;
use serde_json::{Number, Value};
use std::collections::HashMap;
use std::thread::sleep;
use std::time::Duration;

/// Underlying numeric type.
pub type Scalar = f32;

/// Animations supported by Sprite
#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, PartialEq)]
pub enum Animation {
    /// check for events with the mouse
    MouseCallback,
    /// time has passed longer than
    LongerThan(f64),
    /// counter
    ///
    /// Random complex condition
    ComplexCondition(u64),
    /// x, y
    ///
    /// Move to specified position, relatively
    MoveBy(Scalar, Scalar),
    /// deg
    ///
    /// Rotate to specified degree, relatively
    RotateBy(Scalar),
    /// sx, sy, sz
    ///
    /// Scale to specified scale, relatively
    ScaleBy(Option<Scalar>, Option<Scalar>, Option<Scalar>),
    /// times
    WriteText(f64),
    /// r, g, b
    ChangeColor(Option<Scalar>, Option<Scalar>, Option<Scalar>),
}

fn mouse_pos(x: f64, y: f64) -> serde_json::Map<String, Value> {
    let mut pos = serde_json::Map::new();
    pos.insert("x".to_string(), Value::Number(Number::from_f64(x).unwrap()));
    pos.insert("y".to_string(), Value::Number(Number::from_f64(y).unwrap()));
    pos
}

fn write_to_screen(txt: String, w: &mut Window) {
    let font = Font::default();
    w.draw_text(
        &txt,
        &Point2::new(-10.0, -10.0),
        120.0,
        &font,
        &Point3::new(1.0, 0.0, 1.0),
    );
}
/// This method ticks the behavior tree for a given duration 'dt' to move the
/// behavior tree forward in time. Note that a tick - basically a depth-first traversal
/// - of the tree is intended to return instantly, so it is important that the action
/// callbacks return instantly. Long-running tasks/actions might take many ticks to complete
/// , where you update and monitor the task on a tick-basis.
///
/// The ticks to execute for as long as the specified time 'dt'.
fn game_tick(
    c: &mut SceneNode,
    w: &mut Window,
    mut events: EventManager,
    timer: &mut Timer,
    bt: &mut BT<Animation, String, serde_json::Value>,
) {
    // timer since bt was first invoked
    let t = timer.duration_since_start();

    // have bt advance dt seconds into the future
    let dt = timer.get_dt();

    // proceed to next iteration in event loop
    let e: Event = UpdateArgs { dt }.into();

    // get data from blackboard
    let db = &*bt.get_blackboard().get_db();
    let inc: u64 = db.get("count").map_or(Some(0), |x| x.as_u64()).unwrap();

    let mut last_pos = mouse_pos(0.0, 0.0);
    // update state of behaviosuccessr tree
    #[rustfmt::skip]
    bt.state.tick(&e,&mut |args: bonsai_bt::ActionArgs<Event, Animation>|
        match *args.action {
            Animation::LongerThan(dur) => {
                if t > dur {
                    (Status::Success, args.dt)
                } else {
                    (Status::Failure, args.dt)
                }
            }
            Animation::MouseCallback => {
                let mut status = RUNNING;
                for event in events.iter() {
                    match event.value {
                        kiss3d::event::WindowEvent::MouseButton(button, kiss3d::event::Action::Press, modif) => {
                            let txt = format!("mouse press event on {:?} with {:?}", button, modif);
                            write_to_screen(txt, w);
                            status = (Status::Success, args.dt)
                        },
                        kiss3d::event::WindowEvent::Key(key, action, modif) => {
                            let txt = format!("key event {:?} on {:?} with {:?}", key, action, modif);
                            write_to_screen(txt, w);
                        },
                        kiss3d::event::WindowEvent::CursorPos(x, y, _modif) => {
                            println!("new cursor pos: ({},{})", x, y);
                            last_pos = mouse_pos(x, y)
                        }
                        _ => ()
                    };
                }

                status
            },
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
                c.append_translation(&trans);
                c.prepend_to_local_translation(&trans);
                // RUNNING
                (Success, dt)
            }
            Animation::RotateBy(rad) => {
                let rot = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), rad);
                c.prepend_to_local_rotation(&rot);
                // RUNNING
                (Success, dt)
            }
            Animation::ScaleBy(sx, sy, sz) => {
                let mut rng = rand::thread_rng();
                let sx: f32 = sx.unwrap_or_else(|| rng.gen_range(0.0..0.5));
                let sy: f32 = sy.unwrap_or_else(|| rng.gen_range(0.0..0.5));
                let sz: f32 = sz.unwrap_or_else(|| rng.gen_range(0.0..0.5));
                c.set_local_scale(sx, sy, sz);
                (Success, dt)
            }
            Animation::WriteText(time) => {
                let font = Font::default();
                let txt = format!("{} secs has passed", time);
                w.draw_text(
                    &txt,
                    &Point2::origin(),
                    120.0,
                    &font,
                    &Point3::new(0.0, 1.0, 1.0),
                );
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
    let db = bt.get_blackboard().get_db();

    // update count
    let _count = db
        .entry("count".to_string())
        .and_modify(|value| {
            let mut count: u64 = value.as_u64().unwrap();
            count += 1;
            *value = serde_json::Value::Number(Number::from(count));
        })
        .or_insert_with(|| serde_json::Value::Number(Number::from(0)))
        .to_owned();

    // update last pos
    let _last_pos = db
        .entry("last_pos".to_string())
        .and_modify(|value| {
            *value = serde_json::Value::Object(last_pos);
        })
        .or_insert_with(|| serde_json::Value::Number(Number::from(0)))
        .to_owned();
}

fn main() {
    use crate::Animation::{
        ChangeColor, ComplexCondition, LongerThan, MouseCallback, MoveBy, RotateBy, ScaleBy, WriteText,
    };
    let mut window = Window::new("Kiss3d: cube");
    let mut c = window.add_cube(0.5, 0.5, 0.5);

    c.set_color(1.0, 0.0, 0.0);
    window.set_light(Light::StickToCamera);

    let mut timer = Timer::init_time();

    // create BT
    let behavior = While(
        Box::new(Action(MouseCallback)),
        vec![
            Action(ChangeColor(None, None, None)),
            If(
                // if ComplexCondition action `succeeds`, sequence will proceed
                // if it returns `running`, sequence will restart from beginning
                // if `fails`, the sequence will restart from beginning
                Box::new(Action(ComplexCondition(100))),
                // if success
                Box::new(Action(MoveBy(0.0, 0.0))),
                // if failure
                Box::new(Action(MoveBy(-0.005, -0.005))),
            ),
            Action(RotateBy(0.054)),
            Wait(0.2),
            If(
                // if ComplexCondition action `succeeds`, sequence will proceed
                // if it returns `running`, sequence will restart from beginning
                // if `fails`, the sequence will restart from beginning
                Box::new(Invert(Box::new(Action(LongerThan(5.0))))),
                // if success
                Box::new(Action(ScaleBy(None, None, None))),
                // if failure
                Box::new(Action(WriteText(5.0))),
            ),
        ],
    );

    // only run animation bt for 20.0 secs
    let behavior = WhenAny(vec![Wait(20.0), behavior]);

    let blackboard: HashMap<String, serde_json::Value> = HashMap::new();
    let bt_serialized = serde_json::to_string_pretty(&behavior).unwrap();
    println!("creating bt: \n {} \n", bt_serialized);
    let mut bt = BT::new(behavior, blackboard);

    while window.render() {
        let events = window.events();

        sleep(Duration::new(0, 0.1e+9 as u32));
        game_tick(&mut c, &mut window, events, &mut timer, &mut bt);
    }
}
