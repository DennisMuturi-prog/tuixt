use std::{env, fs, io};

use crossterm::{
    cursor::SetCursorStyle,
    event::{DisableBracketedPaste, EnableBracketedPaste},
    execute,
    terminal::size,
};
use tree_sitter::{Parser, Query, QueryCursor};
use tuixt::app::{App, build_capture_styles};

fn main() -> io::Result<()> {
    let (file_path,file_contents) = {
        let mut args = env::args();
        match args.nth(1) {
            Some(arg1) => {
                println!("arg is {}", arg1);
                match fs::read_to_string(&arg1){
                    Ok(source) =>{
                        (arg1,source)
                    },
                    Err(err) => {
                        panic!("the err is {}",err);
                    },
                }
            }
            None => {
                println!("provide the file path or file name");
                ("".to_string(),"".to_string())
            }
        }
    };
    let (initial_window_width, initial_window_height) = match size() {
        Ok(dimensions) => dimensions,
        Err(err) => {
            println!("error occurred could not find window dimensions");
            return Err(err);
        }
    };
    if initial_window_height < 11  || initial_window_width <11 {
        println!("not enough port view ,resize then start again");
        return Ok(());
    }
    execute!(std::io::stdout(), EnableBracketedPaste)?;
    let language=tree_sitter_rust::LANGUAGE.into();
    let mut parser = Parser::new();
    parser
        .set_language(&language)
        .expect("Error loading Rust grammar");
    let highlights_query = Query::new(&language, tree_sitter_rust::HIGHLIGHTS_QUERY).unwrap(); 
    let query_cursor = QueryCursor::new();
    let highlights_styles= build_capture_styles(&highlights_query);
    let mut app = App::new(&file_contents, (initial_window_width-6) as usize, (initial_window_height-6) as usize, true, parser,file_path,highlights_query,query_cursor,highlights_styles);
    ratatui::run(|terminal| app.run(terminal))?;
    execute!(std::io::stdout(), DisableBracketedPaste)?;
    execute!(std::io::stdout(), SetCursorStyle::DefaultUserShape)
}
