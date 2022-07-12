use bonsai_bt::{Event, Status::Success, UpdateArgs, BT, RUNNING};
use ggez::mint;

//algorithm stuff
const SPEED_LIMIT: f32 = 400.0; // Pixels per second
const VISUAL_RANGE: f32 = 32.0; // Pixels
const MIN_DISTANCE: f32 = 16.0; // Pixels

#[derive(Clone, Debug)]
pub enum Action {
    /// avoid others
    AvoidOthers,
    /// Fly towards center
    FlyTowardsCenter,
    /// Match velocity
    MatchVelocity,
    /// Limit speed
    LimitSpeed,
    /// Keep within bounds
    KeepWithinBounds,
}

#[derive(Debug, Clone)]
pub struct Boid {
    pub x: f32,
    pub y: f32,
    pub dx: f32,
    pub dy: f32,
    pub color: [f32; 4],
    pub bt: BT<Action, String, f32>,
}

impl Boid {
    pub fn new(win_width: f32, win_height: f32, bt: BT<Action, String, f32>) -> Boid {
        Boid {
            x: (rand::random::<f32>() * win_width / 2.0 + win_width / 4.0),
            y: (rand::random::<f32>() * win_height / 2.0 + win_height / 4.0),
            dx: (rand::random::<f32>() - 0.5) * SPEED_LIMIT,
            dy: (rand::random::<f32>() - 0.5) * SPEED_LIMIT,
            color: [
                //rgb
                (rand::random::<f32>() * 128.0 + 128.0) / 255.0,
                (rand::random::<f32>() * 128.0 + 128.0) / 255.0,
                (rand::random::<f32>() * 128.0 + 128.0) / 255.0,
                0.5,
            ],
            bt,
        }
    }

    fn distance(&self, boid: &Boid) -> f32 {
        ((self.x - boid.x).powi(2) + (self.y - boid.y).powi(2)).sqrt()
    }
}

pub fn game_tick(dt: f32, cursor: mint::Point2<f32>, boid: &mut Boid, other_boids: Vec<Boid>) {
    // proceed to next iteration in event loop
    let e: Event = UpdateArgs { dt: dt.into() }.into();

    // unwrap bt for boid
    let mut bt = boid.bt.clone();
    let db = &*bt.get_blackboard().get_db();
    let win_width: f32 = *db.get("win_width").unwrap();
    let win_height: f32 = *db.get("win_height").unwrap();

    #[rustfmt::skip]
    bt.state.tick(&e,&mut |args: bonsai_bt::ActionArgs<Event, Action>| {
        match &*args.action {
            Action::AvoidOthers => {
                let avoid_factor = 0.5;
                let mut move_x = 0.0;
                let mut move_y = 0.0;
                for other in &other_boids {
                    let dist = boid.distance(other);
                    if dist < MIN_DISTANCE && dist > 0.0 {
                        move_x += boid.x - other.x;
                        move_y += boid.y - other.y;
                    }
                }
                boid.dx += move_x * avoid_factor;
                boid.dy += move_y * avoid_factor;

                RUNNING
            },
            Action::FlyTowardsCenter => {
                let centering_factor = 0.05; // adjust velocity by this %
                let mut center_x = 0.0;
                let mut center_y = 0.0;
                let mut num_neighbors = 0.0;
                for other in &other_boids {
                    if boid.distance(other) < VISUAL_RANGE {
                        center_x += other.x;
                        center_y += other.y;
                        num_neighbors += 1.0;
                    }
                }
                if num_neighbors > 0.0 {
                    center_x /= num_neighbors;
                    center_y /= num_neighbors;

                    boid.dx += (center_x - boid.x) * centering_factor;
                    boid.dy += (center_y - boid.y) * centering_factor;
                }

                RUNNING
            },
            Action::MatchVelocity => {
                let matching_factor = 0.1;
                let mut avg_dx = 0.0;
                let mut avg_dy = 0.0;
                let mut num_neighbors = 0.0;
                for other in &other_boids {
                    if boid.distance(other) < VISUAL_RANGE {
                        avg_dx += other.dx;
                        avg_dy += other.dy;
                        num_neighbors += 1.0;
                    }
                }
                if num_neighbors > 0.0 {
                    avg_dx /= num_neighbors;
                    avg_dy /= num_neighbors;

                    boid.dx += (avg_dx - boid.dx) * matching_factor;
                    boid.dy += (avg_dy - boid.dy) * matching_factor;
                }
                (Success, args.dt)
            },
            Action::LimitSpeed => {
                let speed = (boid.dx * boid.dx + boid.dy * boid.dy).sqrt();
                if speed > SPEED_LIMIT {
                    boid.dx = (boid.dx / speed) * SPEED_LIMIT;
                    boid.dy = (boid.dy / speed) * SPEED_LIMIT;
                }

                (Success, args.dt)
            },
            Action::KeepWithinBounds => {
                let edge_buffer: f32 = 40.0;
                let turn_factor: f32 = 16.0;
                let mut x_bounded = true;
                let mut y_bounded = true;

                if boid.x < win_width - edge_buffer {
                    boid.dx += turn_factor;
                    x_bounded = !x_bounded;
                }
                if boid.x > edge_buffer {
                    boid.dx -= turn_factor;
                    x_bounded = !x_bounded;
                }
                if boid.y < win_height - edge_buffer {
                    boid.dy += turn_factor;
                    y_bounded = !y_bounded
                }
                if boid.y > edge_buffer {
                    boid.dy -= turn_factor;
                    y_bounded = !y_bounded
                }
                if !x_bounded {
                    boid.dx *= 0.8;
                }
                if !y_bounded {
                    boid.dy *= 0.8;
                }
                if ((boid.x - cursor.x).powi(2) + (boid.y - cursor.y).powi(2)).sqrt() < 20.0 {
                    boid.dx += (boid.x - cursor.x) * 1.0;
                    boid.dy += (boid.y - cursor.y) * 1.0;
                }

                RUNNING
            },
        }

    });
}
