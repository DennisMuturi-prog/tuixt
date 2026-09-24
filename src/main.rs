use std::io;

use crossterm::{cursor::SetCursorStyle, execute, terminal::size};
use tuixt::{app::{App, compute_line_starts}, piece_tree::PieceTree};

fn main() -> io::Result<()> {
    // testing_if_lines_are_being_created_correctly();
    let (initial_window_width, initial_window_height) = match size() {
        Ok(dimensions) => dimensions,
        Err(err) => {
            println!("error occurred could not find window dimensions");
            return Err(err);
        }
    };
    let mut app = App::new(
        "",
        5,
        initial_window_width as usize,
        false,
    );
    ratatui::run(|terminal| app.run(terminal))?;
    execute!(std::io::stdout(), SetCursorStyle::DefaultUserShape)
}
fn testing_if_lines_are_being_created_correctly() {
    let mut piece_tree = PieceTree::default();
    piece_tree.insert("hello world", 0);
    piece_tree.insert("\n", 11);
    let mut buffer = String::new();
    piece_tree.get_lines_text(0, 5, &mut buffer);

    let mut lines = Vec::new();
    compute_line_starts(&buffer, &mut lines);
    println!("lines are {:?}", lines);
    println!("buffer is {}", buffer);
    panic!("hello")
}
