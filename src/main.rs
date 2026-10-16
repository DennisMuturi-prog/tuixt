use std::io;

use crossterm::{cursor::SetCursorStyle, event::{DisableBracketedPaste, EnableBracketedPaste}, execute, terminal::size};
use tuixt::app::App;

fn main() -> io::Result<()> {
    let (initial_window_width, initial_window_height) = match size() {
        Ok(dimensions) => dimensions,
        Err(err) => {
            println!("error occurred could not find window dimensions");
            return Err(err);
        }
    };
    execute!(std::io::stdout(), EnableBracketedPaste)?;
    let mut app = App::new(
        "",
        (initial_window_height-6) as usize,
        (initial_window_width-6) as usize,
        false,
    );
    ratatui::run(|terminal| app.run(terminal))?;
    execute!(std::io::stdout(), DisableBracketedPaste)?;
    execute!(std::io::stdout(), SetCursorStyle::DefaultUserShape)
}

