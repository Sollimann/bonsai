use bonsai_bt::Status;
use std::sync::mpsc::Sender;
use tokio::time::{sleep, Duration};

struct PrintOnDrop(&'static str);

#[derive(Clone, Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Point {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

impl Drop for PrintOnDrop {
    fn drop(&mut self) {
        println!("{}", self.0)
    }
}

pub async fn collision_avoidance_task(tx: Sender<Status>) {
    let _on_drop = PrintOnDrop("collision avoidance task dropped");

    println!("collision avoidance task started");

    let timeout = Duration::from_millis(100);
    let mut duration = 0;
    loop {
        tx.send(Status::Running).unwrap();
        sleep(timeout).await;
        duration += timeout.as_millis();
        println!("collision avoidance task running for {duration:?} millis");
    }
}

pub async fn takeoff_task(tx: Sender<Status>) {
    let _on_drop = PrintOnDrop("Takeoff task dropped");

    println!("takeoff task started");

    let timeout = Duration::from_millis(300);
    let mut duration = 0;
    let mut count = 0;
    loop {
        tx.send(Status::Running).unwrap();
        sleep(timeout).await;
        duration += timeout.as_millis();
        println!("takeoff task running for {duration:?} millis");
        if count > 5 {
            break;
        }
        count += 1;
    }
    println!("takeoff task finished");
    tx.send(Status::Success).unwrap();
}

pub async fn landing_task(tx: Sender<Status>) {
    let _on_drop = PrintOnDrop("Takeoff task dropped");

    println!("landing task started");

    let timeout = Duration::from_millis(300);
    let mut duration = 0;
    let mut count = 0;
    loop {
        tx.send(Status::Running).unwrap();
        sleep(timeout).await;
        duration += timeout.as_millis();
        println!("landing task running for {duration:?} millis");
        if count > 5 {
            break;
        }
        count += 1;
    }
    println!("landing task finished");
    tx.send(Status::Success).unwrap();
}

pub async fn fly_to_point_task(point: Point, tx: Sender<Status>) {
    let _on_drop = PrintOnDrop("flying task dropped");

    println!("flying task started");
    println!("flying to point {point:?}");

    let timeout = Duration::from_millis(500);
    let mut duration = 0;
    let mut count = 0;
    loop {
        tx.send(Status::Running).unwrap();
        sleep(timeout).await;
        duration += timeout.as_millis();
        println!("flying task running for {duration:?} millis");
        if count > 4 {
            break;
        }
        count += 1;
    }
    println!("flying task finished");
    tx.send(Status::Success).unwrap();
}
