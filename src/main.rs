#![windows_subsystem = "windows"]

mod app;
mod display;
mod tasks;
mod windows;

use tokio::{sync::mpsc, time::Duration};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (tx, rx) = mpsc::unbounded_channel::<(tasks::EventFn, Duration)>();

    tasks::spawn_delayed_task_thread(rx);

    windows::main_loop(app::AppData::new(tx, app::Settings::load()?))?;

    Ok(())
}
