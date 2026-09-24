use std::{io, usize};

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    layout::{Alignment, Constraint, Direction, Layout, Position},
    prelude::Rect,
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::piece_tree::PieceTree;

#[derive(Debug, Default)]
pub struct App {
    exit: bool,
    mode: Mode,
    column_number: usize,
    row_number: usize,
    index: usize,
    start_line_number: usize,
    start_index: usize,
    buffer: String,
    piece_tree: PieceTree,
    lines: Vec<TextEditorLine>,
    wrap: bool,
    window_height: usize,
    window_width: usize,
}

impl App {
    /// runs the application's main loop until the user quits
    pub fn new(
        initial_content: &str,
        window_height: usize,
        window_width: usize,
        wrap: bool,
    ) -> Self {
        let piece_tree = PieceTree::new(initial_content);
        let mut buffer = String::with_capacity(window_height * window_width);
        piece_tree.get_lines_text(0, window_height + 1, &mut buffer);
        let mut lines = Vec::with_capacity(window_height + 1);
        compute_line_starts(&buffer, &mut lines);
        Self {
            piece_tree,
            buffer,
            lines,
            window_height,
            window_width,
            wrap,
            ..Self::default()
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(3),
            ])
            .split(area);

        let title_block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default());
        let line_numbers_block = Block::default()
            .borders(Borders::RIGHT)
            .style(Style::default());

        let title = Paragraph::new(Text::styled(
            "Text editor",
            Style::default().fg(Color::Green),
        ))
        .block(title_block.clone());
        frame.render_widget(title, chunks[0]);
        let text_section = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(5), Constraint::Min(1)])
            .split(chunks[1]);

        let line_numbers = Paragraph::new("hello")
            .block(line_numbers_block)
            .alignment(Alignment::Right);
        let text_content = Paragraph::new("world");
        frame.render_widget(text_content, text_section[1]);
        frame.render_widget(line_numbers, text_section[0]);
        if let Mode::Editing = self.mode {
            frame.set_cursor_position(Position::new(
                // Draw the cursor at the current position in the input field.
                // This position is can be controlled via the left and right arrow key
                text_section[1].x + self.column_number as u16,
                // Move one line down, from the border to the input line
                text_section[1].y + self.row_number as u16,
            ))
        }
        let footer_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[2]);

        let current_navigation_text = vec![
            // The first half of the text
            match self.mode {
                Mode::Normal => Span::styled("Normal Mode", Style::default().fg(Color::Green)),
                Mode::Editing => Span::styled("Editing Mode", Style::default().fg(Color::Yellow)),
                Mode::Exiting => Span::styled("Exiting", Style::default().fg(Color::LightRed)),
            }
            .to_owned(),
            // A white divider bar to separate the two sections
            Span::styled(" | ", Style::default().fg(Color::White)),
            // The final section of the text, with hints on what the user is editing
            Span::styled(
                format!(
                    "column {} row {} index:{}",
                    self.column_number, self.row_number, self.index
                ),
                Style::default().fg(Color::Green),
            ),
        ];

        let mode_footer = Paragraph::new(Line::from(current_navigation_text))
            .block(Block::default().borders(Borders::ALL));
        let current_keys_hint = {
            match self.mode {
                Mode::Normal => {
                    Span::styled("(q) to quit / (e) to edit", Style::default().fg(Color::Red))
                }
                Mode::Editing => Span::styled(
                    "(ESC) to go to normal mode",
                    Style::default().fg(Color::Red),
                ),
                Mode::Exiting => Span::styled("(q) to quit", Style::default().fg(Color::Red)),
            }
        };

        let key_notes_footer = Paragraph::new(Line::from(current_keys_hint))
            .block(Block::default().borders(Borders::ALL));

        frame.render_widget(mode_footer, footer_chunks[0]);
        frame.render_widget(key_notes_footer, footer_chunks[1]);

        if let Mode::Exiting = self.mode {
            // frame.render_widget(Clear, frame.area()); //this clears the entire screen and anything already drawn
            let popup_block = Block::default()
                .title("Y/N")
                .borders(Borders::NONE)
                .style(Style::default().bg(Color::DarkGray));

            let exit_text = Text::styled(
                "Would you save the file? (y/n)",
                Style::default().fg(Color::Red),
            );
            // the `trim: false` will stop the text from being cut off when over the edge of the block
            let exit_pop_up_area = centered_rect(60, 25, area);
            let exit_paragraph = Paragraph::new(exit_text)
                .block(popup_block)
                .wrap(Wrap { trim: false });
            frame.render_widget(exit_paragraph, exit_pop_up_area);
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key) => {
                if key.kind == event::KeyEventKind::Release {
                    // Skip events that are not KeyEventKind::Press
                    return Ok(());
                }
                match self.mode {
                    Mode::Normal => match key.code {
                        KeyCode::Char('e') => {
                            self.mode = Mode::Editing;
                        }
                        KeyCode::Char('q') => {
                            self.mode = Mode::Exiting;
                        }
                        _ => {}
                    },
                    Mode::Exiting => match key.code {
                        KeyCode::Char('y') => {
                            self.exit = true;
                            return Ok(());
                        }
                        KeyCode::Char('n') | KeyCode::Char('q') => {
                            self.exit = true;
                            return Ok(());
                        }
                        _ => {}
                    },
                    Mode::Editing if key.kind == KeyEventKind::Press => match key.code {
                        KeyCode::Enter => {
                            self.cursor_up_and_down_column_position_locked = false;
                            self.jump_to_new_line();
                        }
                        KeyCode::Backspace => {
                            self.cursor_up_and_down_column_position_locked = false;
                            self.delete_char();
                        }
                        KeyCode::Esc => {
                            self.cursor_up_and_down_column_position_locked = false;
                            self.mode = Mode::Normal;
                        }
                        KeyCode::Tab => {}
                        KeyCode::Char('z') => {
                            if key.modifiers == KeyModifiers::CONTROL {
                                self.undo();
                            } else {
                                self.cursor_up_and_down_column_position_locked = false;
                                self.add_char('z');
                            }
                        }
                        KeyCode::Char('y') => {
                            if key.modifiers == KeyModifiers::CONTROL {
                                self.redo();
                            } else {
                                self.cursor_up_and_down_column_position_locked = false;
                                self.add_char('y');
                            }
                        }
                        KeyCode::Char(value) => {
                            self.cursor_up_and_down_column_position_locked = false;
                            self.add_char(value);
                        }
                        KeyCode::Left => {
                            self.cursor_up_and_down_column_position_locked = false;
                            self.move_cursor_left(0, self.index.saturating_sub(1));
                        }
                        KeyCode::Right => {
                            self.cursor_up_and_down_column_position_locked = false;
                            self.move_cursor_right(self.index + 1)
                        }
                        KeyCode::Up => {
                            self.move_line_up();
                        }
                        KeyCode::Down => {
                            self.move_line_down();
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
            Event::Paste(pasted_string) => {
                if let Mode::Editing = self.mode {
                    self.paste(pasted_string);
                }
            }
            _ => (),
        }
        Ok(())
    }
    fn insert(&mut self, new_content: &str) {
        self.piece_tree.insert(new_content, self.index);
        self.index += new_content.len();
        self.buffer.clear();
        self.piece_tree.get_lines_text(
            self.start_line_number,
            self.window_height + 1,
            &mut self.buffer,
        );
        compute_line_starts(&self.buffer, &mut self.lines);
    }
    fn compute_row_and_col_from_index(&mut self,previous_number_of_lf:usize) {
        let mut remainder = self.index - self.start_index;
        let mut rows=0;
        for (i, line) in self.lines.iter().enumerate() {
            if remainder > line.length {
                match line.line_type {
                    LineType::Independent => {
                        remainder -= line.length + 1;
                    }
                    LineType::Start => {
                        remainder -= line.length;
                    }
                    LineType::Between => {
                        remainder -= line.length;
                    }
                    LineType::End => { 
                        remainder -= line.length+1;
                    },
                }
                rows +=1;
            }else{
                self.row_number =rows;
                self.column_number = remainder;
                return;
            }
        }
    }
}
fn compute_line_starts(buf: &str, lines: &mut Vec<TextEditorLine>) {
    if buf.is_empty() {
        return;
    }
    lines.clear();
    lines.push(TextEditorLine {
        start_in_buffer: 0,
        length: 0,
        line_type: LineType::Independent,
    });
    for (i, b) in buf.bytes().enumerate() {
        if b == b'\n' {
            if i + 1 < buf.len() {
                lines.push(TextEditorLine {
                    start_in_buffer: i + 1,
                    length: 0,
                    line_type: LineType::Independent,
                });
            }
        }
    }
    for i in 0..lines.len() {
        if i + 1 < lines.len() {
            lines[i].length = lines[i + 1].start_in_buffer - lines[i].start_in_buffer - 1;
        } else {
            lines[i].length = buf.len() - lines[i].start_in_buffer;
        }
    }
}

#[derive(Debug, Default)]
enum Mode {
    #[default]
    Normal,
    Editing,
    Exiting,
}

#[derive(Debug)]
enum LineType {
    Independent,
    Start,
    Between,
    End,
}
#[derive(Debug)]
struct TextEditorLine {
    start_in_buffer: usize,
    length: usize,
    line_type: LineType,
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}
