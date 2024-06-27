//! A simple API for executing delayed tasks

use tokio::{
    sync::mpsc::UnboundedReceiver,
    task,
    time::{self, Duration},
};

pub type EventFn = Box<dyn Fn() -> anyhow::Result<()> + Send + Sync>;

pub fn spawn_delayed_task_thread(mut rx: UnboundedReceiver<(EventFn, Duration)>) {
    task::spawn(async move {
        let delay = time::sleep(Duration::MAX);
        tokio::pin!(delay);

        let event: EventFn = Box::new(|| Ok(()));
        tokio::pin!(event);

        loop {
            tokio::select! {
                Some((e, d)) = rx.recv() => {
                    // Populate the event state from the message and schedule the task to be executed
                    *event.as_mut() = e;
                    delay.as_mut().reset(time::Instant::now() + d);
                }
                _ = &mut delay => {
                    event().expect("event result");

                    // Clear event state and reset the timer to the far future so this thread goes back to sleep
                    *event.as_mut() = Box::new(||{ Ok(())});
                    delay.as_mut().reset(time::Instant::now() + Duration::from_secs(86400 * 365 * 30)); // Far Future
                },
            }
        }
    });
}
