mod window_managers;
mod app;

use std::io;
use std::process::Command;
use crate::app::App;

fn main() -> io::Result<()> {
    //Command::new("clear").spawn()?.wait()?;
    let mut app = App::new();
    let mut terminal = ratatui::init();
    let app_result = app.run(&mut terminal);
    ratatui::restore();

    let wm_command = match app.select() {
        Some(wm_command) => wm_command,
        None => return app_result,
    };

    Command::new("clear").spawn()?.wait()?;

    println!("Launching {}", wm_command);
    Command::new("sh")
        .arg("-c")
        .arg(wm_command)
        .spawn()?
        .wait()?;

    Ok(())
}
