use bonsai_bt::Status;
use std::sync::mpsc::Sender;
use tokio::time::{sleep, Duration};

struct PrintOnDrop(&'static str);

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
