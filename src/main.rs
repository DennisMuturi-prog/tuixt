use std::io;

use tuixt::app::App;

fn main()-> io::Result<()> {
    ratatui::run(|terminal| App::default().run(terminal))
}
