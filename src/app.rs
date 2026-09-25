use std::{
    cmp::{max, min},
    io,
};

use crossterm::{cursor::SetCursorStyle, execute};
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    layout::{Alignment, Constraint, Direction, Layout, Position},
    prelude::Rect,
    style::{Color, Style},
    text::{Line, Span, Text, ToLine},
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
        piece_tree.get_lines_text(0, window_height, &mut buffer);
        let mut lines = Vec::with_capacity(window_height);
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
            .borders(Borders::BOTTOM)
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

        let line_numbers = Paragraph::new(self.get_line_numbers_gutter())
            .block(line_numbers_block)
            .alignment(Alignment::Right);
        let text_content = Paragraph::new(self.get_display_content());
        frame.render_widget(text_content, text_section[1]);
        frame.render_widget(line_numbers, text_section[0]);
        if matches!(self.mode,Mode::Editing | Mode::Normal){
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
                    self.column_number,
                    self.row_number + self.start_line_number + 1,
                    self.index
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
                            execute!(std::io::stdout(), SetCursorStyle::BlinkingBar)?;
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
                            self.insert("\n");
                            // panic!("buffer is {}",self.buffer);
                        }
                        KeyCode::Backspace => {
                            self.delete_char();
                        }
                        KeyCode::Esc => {
                            self.mode = Mode::Normal;
                            execute!(std::io::stdout(), SetCursorStyle::SteadyBlock)?;
                        }
                        KeyCode::Tab => {}
                        KeyCode::Char('z') => {
                            if key.modifiers == KeyModifiers::CONTROL {
                                self.undo();
                            } else {
                                self.insert_char('z');
                            }
                        }
                        KeyCode::Char('y') => {
                            if key.modifiers == KeyModifiers::CONTROL {
                                self.redo();
                            } else {
                                self.insert_char('y');
                            }
                        }
                        KeyCode::Char(value) => {
                            self.insert_char(value);
                        }
                        KeyCode::Left => {
                            if self.row_number == 0
                                && self.column_number == 0
                                && (self.start_line_number as i32 - 1) >= 0
                            {
                                self.start_line_number -= 1;
                                self.start_index = self
                                    .piece_tree
                                    .get_start_offset_of_a_line(self.start_line_number);
                                self.refresh_content();
                            }
                            self.index = self.index.saturating_sub(1);
                            self.compute_row_and_col_from_index();
                        }
                        KeyCode::Right => {
                            if self.row_number + 1 >= self.window_height
                                && self.column_number + 1 >= self.lines[self.row_number].length()
                            {
                                let total_lines = self.piece_tree.get_line_feed_count() + 1;
                                if self.start_line_number + self.row_number + 1 < total_lines {
                                    self.start_line_number += 1;
                                    self.start_index = self
                                        .piece_tree
                                        .get_start_offset_of_a_line(self.start_line_number);
                                    self.refresh_content();
                                }
                            }

                            let content_len = self.piece_tree.get_tree_len();
                            self.index = min(content_len, self.index + 1);
                            self.compute_row_and_col_from_index();
                        }
                        KeyCode::Up => {
                            if self.row_number == 0 {
                                if (self.start_line_number as i32 - 1) >= 0 {
                                    self.start_line_number -= 1;
                                    self.start_index = self
                                        .piece_tree
                                        .get_start_offset_of_a_line(self.start_line_number);
                                    self.refresh_content();
                                    self.compute_index_from_row_and_col();
                                }
                            } else {
                                self.row_number -= 1;
                                self.compute_index_from_row_and_col();
                            }
                        }
                        KeyCode::Down => {
                            if self.row_number + 1 >= self.window_height {
                                let total_lines = self.piece_tree.get_line_feed_count() + 1;
                                if self.start_line_number + self.row_number + 1 < total_lines {
                                    self.start_line_number += 1;
                                    self.start_index = self
                                        .piece_tree
                                        .get_start_offset_of_a_line(self.start_line_number);
                                    self.refresh_content();
                                    self.compute_index_from_row_and_col();
                                }
                            } else {
                                self.row_number = min(self.row_number + 1, self.lines.len() - 1);
                                self.compute_index_from_row_and_col();
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
            Event::Paste(pasted_string) => {
                if let Mode::Editing = self.mode {
                    self.insert(&pasted_string);
                }
            }
            _ => (),
        }
        Ok(())
    }
    fn get_line_numbers_gutter(&self) -> String {
        let mut line_number_display = self.start_line_number + 1;
        let mut gutter = String::with_capacity(self.lines.len() * 2);

        for line in self.lines.iter().take(self.window_height) {
            match line.line_type {
                LineType::Independent => {
                    gutter.push_str(&line_number_display.to_string());
                    line_number_display += 1;
                    gutter.push('\n');
                }
                LineType::Start => {
                    gutter.push_str(&line_number_display.to_line().to_string());
                    line_number_display += 1;
                    gutter.push('\n');
                }
                LineType::Between => {
                    gutter.push_str(" \n");
                }
                LineType::End => {
                    gutter.push_str(" \n");
                }
                LineType::Empty => {
                    gutter.push_str(&line_number_display.to_string());
                    line_number_display += 1;
                    gutter.push('\n');
                }
            }
        }
        gutter
    }
    fn get_display_content(&self) -> Vec<Line> {
        let mut visible_lines = Vec::with_capacity(self.lines.len());
        let mut offset = 0;
        if self.row_number >= self.window_width {
            offset = self.row_number - self.window_width + 1;
        }
        for line in self.lines.iter() {
            match line.line_type {
                LineType::Empty => {
                    visible_lines.push(Line::from(""));
                }
                _ => {
                    let line = Line::from(
                        &self.buffer
                            [line.start_in_buffer + offset..line.start_in_buffer + line.length],
                    );
                    visible_lines.push(line);
                }
            }
        }
        visible_lines
    }
    fn delete_char(&mut self) {}
    fn undo(&mut self) {}
    fn redo(&mut self) {}
    fn insert_char(&mut self, new_content: char) {
        self.piece_tree.insert_char(new_content, self.index);
        self.index += 1;
        self.refresh_content();
        self.compute_row_and_col_from_index();
    }
    fn insert(&mut self, new_content: &str) {
        let previous_lf_count = self.piece_tree.get_line_feed_count();
        self.piece_tree.insert(new_content, self.index);
        let current_lf_count = self.piece_tree.get_line_feed_count();
        let new_lines_count = current_lf_count - previous_lf_count;
        self.index += new_content.len();
        let new_top_line = get_new_top_line(
            self.start_line_number as i32,
            self.window_height as i32,
            (self.start_line_number + self.row_number + new_lines_count) as i32,
            0,
        );
        if new_top_line != self.start_line_number {
            self.start_index = self.piece_tree.get_start_offset_of_a_line(new_top_line);
            self.start_line_number = new_top_line;
        }
        self.refresh_content();
        self.compute_row_and_col_from_index();
    }
    fn refresh_content(&mut self) {
        self.buffer.clear();
        self.piece_tree.get_lines_text(
            self.start_line_number,
            self.window_height,
            &mut self.buffer,
        );
        compute_line_starts(&self.buffer, &mut self.lines);
    }
    fn compute_index_from_row_and_col(&mut self) {
        let mut sum = 0;
        for line in self.lines.iter().take(self.row_number) {
            match line.line_type {
                LineType::Independent => {
                    sum += line.length();
                }
                LineType::Start => {
                    sum += line.length();
                }
                LineType::Between => {
                    sum -= line.length();
                }
                LineType::End => {
                    sum += line.length();
                }
                LineType::Empty => {
                    sum += line.length();
                }
            }
        }
        sum += self.column_number;
        self.index = sum + self.start_index;
    }
    fn compute_row_and_col_from_index(&mut self) {
        let mut remainder = self.index - self.start_index;
        let mut rows = 0;
        for line in self.lines.iter() {
            if remainder > line.length {
                match line.line_type {
                    LineType::Independent => {
                        remainder -= line.length();
                    }
                    LineType::Start => {
                        remainder -= line.length();
                    }
                    LineType::Between => {
                        remainder -= line.length();
                    }
                    LineType::End => {
                        remainder -= line.length();
                    }
                    LineType::Empty => {
                        remainder -= line.length();
                    }
                }
                rows += 1;
            } else {
                self.row_number = rows;
                self.column_number = remainder;
                return;
            }
        }
    }
}
fn get_new_top_line(
    top_line_old: i32,
    window_height: i32,
    cursor_line_new: i32,
    scroll_off: i32,
) -> usize {
    let min_top = cursor_line_new - window_height + 1 + scroll_off;
    let max_top = cursor_line_new - scroll_off;
    let top_line_new = max(min_top, min(top_line_old, max_top));
    (max(0, top_line_new)) as usize
}
pub fn compute_line_starts(buf: &str, lines: &mut Vec<TextEditorLine>) {
    if buf.is_empty() {
        lines.push(TextEditorLine {
            start_in_buffer: 0,
            length: 0,
            line_type: LineType::Empty,
        });
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
            if buf.as_bytes()[buf.len() - 1] == b'\n' {
                lines[i].length = buf.len() - lines[i].start_in_buffer - 1;
                lines.push(TextEditorLine {
                    start_in_buffer: i + 1,
                    length: 0,
                    line_type: LineType::Empty,
                });
            } else {
                lines[i].length = buf.len() - lines[i].start_in_buffer;
            }
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
    Empty,
}
#[derive(Debug)]
pub struct TextEditorLine {
    start_in_buffer: usize,
    length: usize,
    line_type: LineType,
}
impl TextEditorLine {
    fn length(&self) -> usize {
        match self.line_type {
            LineType::Independent => self.length + 1,
            LineType::Start => self.length,
            LineType::Between => self.length,
            LineType::End => self.length + 1,
            LineType::Empty => 0,
        }
    }
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
