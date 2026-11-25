use std::{
    cmp::{max, min},
    io,
    ops::Range,
    usize,
};

use crossterm::{cursor::SetCursorStyle, execute};
use icu_segmenter::GraphemeClusterSegmenter;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    layout::{Alignment, Constraint, Direction, Layout, Position},
    prelude::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text, ToLine},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use textwrap::{Options, wrap};
use tree_sitter::{
    InputEdit, Node as TSNode, Parser, Point, Query, QueryCursor, StreamingIterator, Tree,
};
use unicode_truncate::UnicodeTruncateStr;
use unicode_width::UnicodeWidthStr;

use crate::piece_tree::PieceTree;

pub struct App {
    exit: bool,
    mode: Mode,
    column_number: usize,
    row_number: usize,
    index: usize,
    buffer_start_index: usize,
    buffer: String,
    piece_tree: PieceTree,
    render_lines: Vec<TextEditorLine>,
    no_of_hard_lines_queried: usize,
    wrap: bool,
    window_height: usize,
    window_width: usize,
    line_column: usize,
    col_hint: usize,
    start_of_line: usize,
    undo_states: Vec<UndoRedoState>,
    redo_states: Vec<UndoRedoState>,
    top_visible_line: TopVisibleLine,
    parser: Parser,
    syntax_tree: Tree,
    highlights_query: Query,
    file_path: String,
    highlight_styles: Vec<Style>,
}

#[derive(Default, Clone, Copy, PartialEq)]
struct TopVisibleLine {
    pos_into_render_lines: usize,
    hard_line_info: HardLineInfo,
}

#[derive(Default, Clone, Copy, PartialEq)]
struct HardLineInfo {
    start_idx_in_global_buf: usize,
    hard_line_no: usize,
    pos_into_render_lines: usize,
}

impl App {
    /// runs the application's main loop until the user quits
    pub fn new(
        initial_content: &str,
        window_width: usize,
        window_height: usize,
        wrap: bool,
        mut parser: Parser,
        file_path: String,
        highlights_query: Query,
        highlight_styles: Vec<Style>,
    ) -> Self {
        let piece_tree = PieceTree::new(initial_content);
        let mut buffer = String::with_capacity(window_height * window_width);
        piece_tree.get_lines_text(0, window_height, &mut buffer);
        let lines = Vec::with_capacity(window_height);
        let syntax_tree = parser.parse(initial_content, None).unwrap();
        Self {
            piece_tree,
            buffer,
            render_lines: lines,
            window_height,
            window_width,
            wrap,
            start_of_line: 0,
            parser,
            syntax_tree,
            exit: false,
            mode: Mode::Normal,
            column_number: 0,
            row_number: 0,
            index: 0,
            buffer_start_index: 0,
            no_of_hard_lines_queried: 0,
            line_column: 0,
            col_hint: 0,
            undo_states: Vec::with_capacity(100),
            redo_states: Vec::with_capacity(100),
            top_visible_line: TopVisibleLine::default(),
            file_path,
            highlights_query,
            highlight_styles,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        if self.wrap {
            self.compute_line_starts_with_wrap();
        } else {
            self.compute_line_starts_with_no_wrap();
        }
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
        if matches!(self.mode, Mode::Editing | Mode::Normal) {
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

        let mut no_of_hard_lines_passed = 0;
        for render_line in self
            .render_lines
            .iter()
            .skip(self.top_visible_line.pos_into_render_lines)
            .take(self.row_number)
        {
            match render_line.line_type {
                LineType::Independent => {
                    no_of_hard_lines_passed += 1;
                }
                LineType::End => {
                    no_of_hard_lines_passed += 1;
                }
                LineType::Empty => {
                    no_of_hard_lines_passed += 1;
                }
                _ => {}
            }
        }

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
                    self.line_column + 1,
                    no_of_hard_lines_passed + self.top_visible_line.hard_line_info.hard_line_no + 1,
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
                            self.undo_states.push(self.snapshot());
                            self.redo_states.clear();
                            self.insert(self.index, "\n");
                        }
                        KeyCode::Backspace => {
                            if self.index > 0 {
                                self.undo_states.push(self.snapshot());
                                self.redo_states.clear();
                                let local_index = self.index - self.buffer_start_index - 1;
                                let starting_of_this_char =
                                    self.buffer.floor_char_boundary(local_index);
                                let byte_width = local_index - starting_of_this_char + 1;
                                self.delete(self.index - byte_width, byte_width);
                            }
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
                                self.undo_states.push(self.snapshot());
                                self.redo_states.clear();
                                self.insert(self.index, "z");
                            }
                        }
                        KeyCode::Char('y') => {
                            if key.modifiers == KeyModifiers::CONTROL {
                                self.redo();
                            } else {
                                self.undo_states.push(self.snapshot());
                                self.redo_states.clear();
                                self.insert(self.index, "y");
                            }
                        }
                        KeyCode::Char(value) => {
                            let mut buf = [0u8; 4];
                            let s = value.encode_utf8(&mut buf);
                            self.undo_states.push(self.snapshot());
                            self.redo_states.clear();
                            self.insert(self.index, s);
                        }
                        KeyCode::Left => {
                            if self.index == 0 {
                                return Ok(());
                            }
                            if self.row_number == 0
                                && self.line_column == 0
                                && self.top_visible_line.pos_into_render_lines == 0
                                && (self.top_visible_line.hard_line_info.hard_line_no as i32 - 1)
                                    >= 0
                            {
                                self.top_visible_line.hard_line_info.hard_line_no -= 1;
                                self.no_of_hard_lines_queried += 1;
                                self.buffer_start_index =
                                    self.piece_tree.get_start_offset_of_a_line(
                                        self.top_visible_line.hard_line_info.hard_line_no,
                                    );
                                self.top_visible_line.hard_line_info.pos_into_render_lines = 0;
                                self.top_visible_line.hard_line_info.start_idx_in_global_buf =
                                    self.buffer_start_index;
                                self.refresh_content_for_current_visible_window();
                                if matches!(self.render_lines[0].line_type, LineType::Independent) {
                                    self.line_column = self.render_lines[0].width;
                                } else {
                                    for (i, line) in self.render_lines.iter().enumerate() {
                                        if matches!(line.line_type, LineType::End) {
                                            self.top_visible_line.pos_into_render_lines = i;
                                            break;
                                        }
                                    }
                                    self.line_column = self.render_lines
                                        [self.top_visible_line.pos_into_render_lines]
                                        .width;
                                }
                                self.index -= 1;
                                let local_index = self.index - self.buffer_start_index;
                                if self.buffer.as_bytes()[local_index - 1] == b'\r' {
                                    self.index -= 1;
                                }
                            } else if self.row_number == 0
                                && self.line_column == 0
                                && (self.top_visible_line.pos_into_render_lines as i32 - 1) >= 0
                            {
                                self.top_visible_line.pos_into_render_lines -= 1;
                                self.line_column = self.render_lines
                                    [self.top_visible_line.pos_into_render_lines]
                                    .width;
                                if matches!(
                                    self.render_lines[self.top_visible_line.pos_into_render_lines]
                                        .line_type,
                                    LineType::Independent
                                ) || matches!(
                                    self.render_lines[self.top_visible_line.pos_into_render_lines]
                                        .line_type,
                                    LineType::End
                                ) {
                                    self.top_visible_line.hard_line_info.hard_line_no -= 1;
                                    self.no_of_hard_lines_queried += 1;
                                    self.top_visible_line.hard_line_info.start_idx_in_global_buf =
                                        self.piece_tree.get_start_offset_of_a_line(
                                            self.top_visible_line.hard_line_info.hard_line_no,
                                        );
                                    if matches!(
                                        self.render_lines
                                            [self.top_visible_line.pos_into_render_lines]
                                            .line_type,
                                        LineType::End
                                    ) {
                                        let mut i = self.top_visible_line.pos_into_render_lines;
                                        loop {
                                            match self.render_lines[i].line_type {
                                                LineType::Independent => {
                                                    break;
                                                }
                                                LineType::Start => {
                                                    break;
                                                }
                                                LineType::Between => {
                                                    i -= 1;
                                                }
                                                LineType::End => {
                                                    i -= 1;
                                                }
                                                LineType::Empty => {
                                                    break;
                                                }
                                            }
                                        }
                                        self.top_visible_line
                                            .hard_line_info
                                            .pos_into_render_lines = i;
                                    } else {
                                        self.top_visible_line
                                            .hard_line_info
                                            .pos_into_render_lines =
                                            self.top_visible_line.pos_into_render_lines;
                                    }

                                    self.index -= 1;
                                    let local_index = self.index - self.buffer_start_index;
                                    if self.buffer.as_bytes()[local_index - 1] == b'\r' {
                                        self.index -= 1;
                                    }
                                }
                            } else if self.line_column == 0 {
                                self.row_number -= 1;
                                self.line_column = self.render_lines
                                    [self.top_visible_line.pos_into_render_lines + self.row_number]
                                    .width;
                                if matches!(
                                    self.render_lines[self.top_visible_line.pos_into_render_lines
                                        + self.row_number]
                                        .line_type,
                                    LineType::End
                                ) || matches!(
                                    self.render_lines[self.top_visible_line.pos_into_render_lines
                                        + self.row_number]
                                        .line_type,
                                    LineType::Independent
                                ) {
                                    self.index -= 1;
                                    let local_index = self.index - self.buffer_start_index;
                                    if self.buffer.as_bytes()[local_index - 1] == b'\r' {
                                        self.index -= 1;
                                    }
                                }
                            } else {
                                let local_index = self.index - self.buffer_start_index - 1;
                                let starting_of_this_char =
                                    self.buffer.floor_char_boundary(local_index);
                                let byte_width = local_index - starting_of_this_char + 1;
                                self.index -= byte_width;
                                let char_to_be_pass =
                                    &self.buffer[starting_of_this_char..=local_index];
                                self.line_column -= char_to_be_pass.width();
                            }
                            self.col_hint = self.line_column;
                            self.compute_new_horizontal_start_line();
                        }
                        // i did this because my right arrow button is damaged,so am i using the page down to simulate it
                        KeyCode::Right | KeyCode::PageDown => {
                            if self.index == self.piece_tree.get_tree_len() {
                                return Ok(());
                            }
                            if self.line_column
                                >= self.render_lines
                                    [self.top_visible_line.pos_into_render_lines + self.row_number]
                                    .width
                                && self.row_number + 1 >= self.window_height
                            {
                                if self.top_visible_line.pos_into_render_lines + self.window_height
                                    < self.render_lines.len()
                                {
                                    self.line_column = 0;
                                    self.top_visible_line.pos_into_render_lines += 1;
                                    if matches!(
                                        self.render_lines[self
                                            .top_visible_line
                                            .pos_into_render_lines
                                            + self.row_number
                                            - 1]
                                        .line_type,
                                        LineType::End
                                    ) || matches!(
                                        self.render_lines[self
                                            .top_visible_line
                                            .pos_into_render_lines
                                            + self.row_number
                                            - 1]
                                        .line_type,
                                        LineType::Independent
                                    ) {
                                        let local_index = self.index - self.buffer_start_index;
                                        if self.buffer.as_bytes()[local_index] == b'\r' {
                                            self.index += 2;
                                        } else {
                                            self.index += 1;
                                        }
                                    }
                                    if matches!(
                                        self.render_lines
                                            [self.top_visible_line.pos_into_render_lines]
                                            .line_type,
                                        LineType::Independent
                                    ) || matches!(
                                        self.render_lines
                                            [self.top_visible_line.pos_into_render_lines]
                                            .line_type,
                                        LineType::Start
                                    ) {
                                        self.top_visible_line.hard_line_info.hard_line_no += 1;
                                        self.no_of_hard_lines_queried =
                                            self.no_of_hard_lines_queried.saturating_sub(1);
                                        self.top_visible_line
                                            .hard_line_info
                                            .start_idx_in_global_buf =
                                            self.piece_tree.get_start_offset_of_a_line(
                                                self.top_visible_line.hard_line_info.hard_line_no,
                                            );
                                        self.top_visible_line
                                            .hard_line_info
                                            .pos_into_render_lines =
                                            self.top_visible_line.pos_into_render_lines;
                                    }
                                } else {
                                    let total_lines = self.piece_tree.get_line_feed_count() + 1;
                                    if self.top_visible_line.hard_line_info.hard_line_no
                                        + self.no_of_hard_lines_queried
                                        < total_lines
                                    {
                                        self.request_more_lines();
                                        self.line_column = 0;
                                        self.top_visible_line.pos_into_render_lines += 1;
                                        let local_index = self.index - self.buffer_start_index;
                                        if self.buffer.as_bytes()[local_index] == b'\r' {
                                            self.index += 2;
                                        } else {
                                            self.index += 1;
                                        }
                                        if matches!(
                                            self.render_lines
                                                [self.top_visible_line.pos_into_render_lines]
                                                .line_type,
                                            LineType::Independent
                                        ) || matches!(
                                            self.render_lines
                                                [self.top_visible_line.pos_into_render_lines]
                                                .line_type,
                                            LineType::Start
                                        ) {
                                            self.top_visible_line.hard_line_info.hard_line_no += 1;
                                            self.no_of_hard_lines_queried =
                                                self.no_of_hard_lines_queried.saturating_sub(1);
                                            self.top_visible_line
                                                .hard_line_info
                                                .start_idx_in_global_buf =
                                                self.piece_tree.get_start_offset_of_a_line(
                                                    self.top_visible_line
                                                        .hard_line_info
                                                        .hard_line_no,
                                                );
                                            self.top_visible_line
                                                .hard_line_info
                                                .pos_into_render_lines =
                                                self.top_visible_line.pos_into_render_lines;
                                        }
                                    }
                                }
                            } else if self.line_column
                                >= self.render_lines
                                    [self.top_visible_line.pos_into_render_lines + self.row_number]
                                    .width
                            {
                                if self.top_visible_line.pos_into_render_lines + self.row_number + 1
                                    < self.render_lines.len()
                                {
                                    self.row_number += 1;
                                    self.line_column = 0;
                                    if matches!(
                                        self.render_lines[self
                                            .top_visible_line
                                            .pos_into_render_lines
                                            + self.row_number
                                            - 1]
                                        .line_type,
                                        LineType::End
                                    ) || matches!(
                                        self.render_lines[self
                                            .top_visible_line
                                            .pos_into_render_lines
                                            + self.row_number
                                            - 1]
                                        .line_type,
                                        LineType::Independent
                                    ) {
                                        let local_index = self.index - self.buffer_start_index;
                                        if self.buffer.as_bytes()[local_index] == b'\r' {
                                            self.index += 2;
                                        } else {
                                            self.index += 1;
                                        }
                                    }
                                } else {
                                    return Ok(());
                                }
                            } else {
                                let content_len = self.piece_tree.get_tree_len();
                                let local_index = self.index - self.buffer_start_index;
                                let ending_of_this_char =
                                    self.buffer.ceil_char_boundary(local_index + 1);
                                let byte_width = ending_of_this_char - local_index;
                                self.index = min(content_len, self.index + byte_width);
                                let char_to_be_pass =
                                    &self.buffer[local_index..ending_of_this_char];
                                self.line_column += char_to_be_pass.width();
                            }

                            self.col_hint = self.line_column;

                            self.compute_new_horizontal_start_line();
                        }

                        KeyCode::Up => {
                            if self.row_number == 0 {
                                if self.top_visible_line.pos_into_render_lines == 0 {
                                    if (self.top_visible_line.hard_line_info.hard_line_no as i32
                                        - 1)
                                        >= 0
                                    {
                                        self.top_visible_line.hard_line_info.hard_line_no -= 1;
                                        self.no_of_hard_lines_queried += 1;
                                        self.buffer_start_index =
                                            self.piece_tree.get_start_offset_of_a_line(
                                                self.top_visible_line.hard_line_info.hard_line_no,
                                            );
                                        self.top_visible_line
                                            .hard_line_info
                                            .start_idx_in_global_buf = self.buffer_start_index;
                                        self.top_visible_line
                                            .hard_line_info
                                            .pos_into_render_lines = 0;
                                        self.refresh_content_for_current_visible_window();
                                        if matches!(self.render_lines[0].line_type, LineType::Start)
                                        {
                                            for (i, line) in self.render_lines.iter().enumerate() {
                                                if matches!(line.line_type, LineType::End) {
                                                    self.top_visible_line.pos_into_render_lines = i;
                                                    break;
                                                }
                                            }
                                        }
                                        self.compute_next_line_col();
                                        self.compute_index_from_row_and_col();
                                    }
                                } else {
                                    self.top_visible_line.pos_into_render_lines -= 1;
                                    if matches!(
                                        self.render_lines
                                            [self.top_visible_line.pos_into_render_lines]
                                            .line_type,
                                        LineType::Independent
                                    ) || matches!(
                                        self.render_lines
                                            [self.top_visible_line.pos_into_render_lines]
                                            .line_type,
                                        LineType::End
                                    ) {
                                        self.top_visible_line.hard_line_info.hard_line_no -= 1;
                                        self.no_of_hard_lines_queried += 1;
                                        self.top_visible_line
                                            .hard_line_info
                                            .start_idx_in_global_buf =
                                            self.piece_tree.get_start_offset_of_a_line(
                                                self.top_visible_line.hard_line_info.hard_line_no,
                                            );
                                        if matches!(
                                            self.render_lines
                                                [self.top_visible_line.pos_into_render_lines]
                                                .line_type,
                                            LineType::End
                                        ) {
                                            let mut i = self.top_visible_line.pos_into_render_lines;
                                            loop {
                                                match self.render_lines[i].line_type {
                                                    LineType::Independent => {
                                                        break;
                                                    }
                                                    LineType::Start => {
                                                        break;
                                                    }
                                                    LineType::Between => {
                                                        i -= 1;
                                                    }
                                                    LineType::End => {
                                                        i -= 1;
                                                    }
                                                    LineType::Empty => {
                                                        break;
                                                    }
                                                }
                                            }
                                            self.top_visible_line
                                                .hard_line_info
                                                .pos_into_render_lines = i;
                                        } else {
                                            self.top_visible_line
                                                .hard_line_info
                                                .pos_into_render_lines =
                                                self.top_visible_line.pos_into_render_lines;
                                        }
                                    }
                                    self.compute_next_line_col();
                                    self.compute_index_from_row_and_col();
                                }
                            } else {
                                self.row_number -= 1;
                                self.compute_next_line_col();
                                self.compute_index_from_row_and_col();
                            }
                        }
                        KeyCode::Down => {
                            if self.row_number + 1 >= self.window_height {
                                if self.top_visible_line.pos_into_render_lines + self.window_height
                                    < self.render_lines.len()
                                {
                                    self.top_visible_line.pos_into_render_lines += 1;
                                    if matches!(
                                        self.render_lines
                                            [self.top_visible_line.pos_into_render_lines]
                                            .line_type,
                                        LineType::Independent
                                    ) || matches!(
                                        self.render_lines
                                            [self.top_visible_line.pos_into_render_lines]
                                            .line_type,
                                        LineType::Start
                                    ) {
                                        self.top_visible_line.hard_line_info.hard_line_no += 1;
                                        self.no_of_hard_lines_queried =
                                            self.no_of_hard_lines_queried.saturating_sub(1);
                                        self.top_visible_line
                                            .hard_line_info
                                            .start_idx_in_global_buf =
                                            self.piece_tree.get_start_offset_of_a_line(
                                                self.top_visible_line.hard_line_info.hard_line_no,
                                            );
                                        self.top_visible_line
                                            .hard_line_info
                                            .pos_into_render_lines =
                                            self.top_visible_line.pos_into_render_lines;
                                    }
                                    self.compute_next_line_col();
                                    self.compute_index_from_row_and_col();
                                } else {
                                    let total_lines = self.piece_tree.get_line_feed_count() + 1;
                                    if self.top_visible_line.hard_line_info.hard_line_no
                                        + self.no_of_hard_lines_queried
                                        < total_lines
                                    {
                                        self.top_visible_line.pos_into_render_lines += 1;
                                        self.request_more_lines();
                                        if matches!(
                                            self.render_lines
                                                [self.top_visible_line.pos_into_render_lines]
                                                .line_type,
                                            LineType::Independent
                                        ) || matches!(
                                            self.render_lines
                                                [self.top_visible_line.pos_into_render_lines]
                                                .line_type,
                                            LineType::Start
                                        ) {
                                            self.top_visible_line.hard_line_info.hard_line_no += 1;
                                            self.no_of_hard_lines_queried =
                                                self.no_of_hard_lines_queried.saturating_sub(1);
                                            self.top_visible_line
                                                .hard_line_info
                                                .start_idx_in_global_buf =
                                                self.piece_tree.get_start_offset_of_a_line(
                                                    self.top_visible_line
                                                        .hard_line_info
                                                        .hard_line_no,
                                                );
                                            self.top_visible_line
                                                .hard_line_info
                                                .pos_into_render_lines =
                                                self.top_visible_line.pos_into_render_lines;
                                        }
                                        self.compute_next_line_col();
                                        self.compute_index_from_row_and_col();
                                    }
                                }
                            } else {
                                self.row_number =
                                    min(self.row_number + 1, self.render_lines.len() - 1);
                                self.compute_next_line_col();
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
                    self.undo_states.push(self.snapshot());
                    self.redo_states.clear();
                    self.insert(self.index, &pasted_string);
                }
            }
            _ => (),
        }
        Ok(())
    }
    fn compute_next_line_col(&mut self) {
        let previous_line_column = self.line_column;
        self.line_column = min(self.col_hint, self.render_lines[self.row_number].width);
        if self.line_column != previous_line_column {
            self.start_of_line = self.line_column.saturating_sub(self.window_width);
        }
        self.column_number = self.line_column - self.start_of_line;
    }
    fn get_line_numbers_gutter(&self) -> String {
        let mut line_number_display = self.top_visible_line.hard_line_info.hard_line_no + 1;
        let mut gutter = String::with_capacity(self.render_lines.len() * 2);

        for line in self
            .render_lines
            .iter()
            .skip(self.top_visible_line.pos_into_render_lines)
            .take(self.window_height)
        {
            match line.line_type {
                LineType::Independent => {
                    gutter.push_str(&line_number_display.to_string());
                    line_number_display += 1;
                    gutter.push('\n');
                }
                LineType::Start => {
                    gutter.push_str(&line_number_display.to_line().to_string());
                    gutter.push('\n');
                }
                LineType::Between => {
                    gutter.push_str(" \n");
                }
                LineType::End => {
                    line_number_display += 1;
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
    fn get_display_content(&self) -> Vec<Line<'_>> {
        let (mut query_cursor, mut curr_idx) = self.get_query_cursor();
        let mut binding = |node: TSNode<'_>| -> CursorTextIter {
            let range_of_node = node.byte_range();
            CursorTextIter {
                piece_tree: &self.piece_tree,
                current_pos: range_of_node.start,
                end: range_of_node.end,
            }
        };
        let mut captures = query_cursor.captures(
            &self.highlights_query,
            self.syntax_tree.root_node(),
            &mut binding,
        );
        let mut styles_and_ranges =
            Vec::with_capacity((self.window_height * self.window_width) / 5);

        while let Some((mat, capture_index)) = captures.next() {
            let capture = mat.captures()[*capture_index];
            let highlight_style = self.highlight_styles[capture.index as usize];
            let byte_range = capture.node.byte_range();
            styles_and_ranges.push((highlight_style, byte_range));
        }
        styles_and_ranges.sort_by(|a, b| {
            a.1.start
                .cmp(&b.1.start)
                .then_with(|| b.1.end.cmp(&a.1.end))
        });
        let mut visible_lines = Vec::with_capacity(self.render_lines.len());
        let mut i = 0;
        let mut active_styles: Vec<(Style, usize)> = Vec::new();
        for line in self
            .render_lines
            .iter()
            .skip(self.top_visible_line.pos_into_render_lines)
            .take(self.window_height)
        {
            match line.line_type {
                LineType::Empty => {
                    visible_lines.push(Line::from(""));
                }
                _ => {
                    if self.wrap {
                        let highlighted_line = self.get_highlighted_render_line(
                            &mut i,
                            &mut active_styles,
                            &styles_and_ranges,
                            curr_idx,
                            curr_idx + line.length,
                        );
                        visible_lines.push(highlighted_line);
                        curr_idx += line.length;
                        continue;
                    }
                    let end = line.start_in_buffer + line.length;
                    if line.width <= self.start_of_line {
                        visible_lines.push(Line::from(""));
                        curr_idx += line.length;
                        continue;
                    }
                    let suffix_width = line.width - self.start_of_line;
                    let (suffix, _) = (&self.buffer[line.start_in_buffer..end])
                        .unicode_truncate_start(suffix_width);
                    let prefix_difference = line.length - suffix.len();

                    let (final_trim, _) = suffix.unicode_truncate(self.window_width);
                    let highlighted_line = self.get_highlighted_render_line(
                        &mut i,
                        &mut active_styles,
                        &styles_and_ranges,
                        curr_idx + prefix_difference,
                        curr_idx + prefix_difference + final_trim.len(),
                    );
                    visible_lines.push(highlighted_line);
                    curr_idx += line.length;
                }
            }
        }
        visible_lines
    }
    fn get_highlighted_render_line<'a>(
        &'a self,
        i: &mut usize,
        active_styles: &mut Vec<(Style, usize)>,
        styles_and_ranges: &[(Style, Range<usize>)],
        start_curr_idx: usize,
        end_idx: usize,
    ) -> Line<'a> {
        let mut curr_idx = start_curr_idx;
        let mut spans: Vec<Span<'_>> = Vec::with_capacity(10);

        while curr_idx < end_idx {
            active_styles.retain(|&(_, end)| end > curr_idx);

            while *i < styles_and_ranges.len() && styles_and_ranges[*i].1.start <= curr_idx {
                let (style, ref range) = styles_and_ranges[*i];
                if range.end > curr_idx {
                    active_styles.push((style, range.end));
                }
                *i += 1;
            }

            let mut next_stop = end_idx;
            if let Some(&(_, min_end)) = active_styles.iter().min_by_key(|&&(_, end)| end) {
                next_stop = min(next_stop, min_end);
            }
            if *i < styles_and_ranges.len() {
                next_stop = min(next_stop, styles_and_ranges[*i].1.start);
            }

            let local_start = curr_idx - self.buffer_start_index;
            let local_end = next_stop - self.buffer_start_index;
            let slice = &self.buffer[local_start..local_end];
            let span = match active_styles.last() {
                Some(&(style, _)) => Span::styled(slice, style),
                None => Span::from(slice),
            };
            spans.push(span);
            curr_idx = next_stop;
        }

        Line::from(spans)
    }
    fn snapshot(&self) -> UndoRedoState {
        UndoRedoState {
            index: self.index,
            buffer_start_index: self.buffer_start_index,
            top_visible_line: self.top_visible_line,
        }
    }
    fn undo(&mut self) {
        self.piece_tree.undo();
        if let Some(previous) = self.undo_states.pop() {
            // The current state is the post-edit state, so it is what a
            // subsequent redo must restore.
            self.redo_states.push(self.snapshot());
            if previous.buffer_start_index >= self.buffer_start_index
                && previous.top_visible_line.hard_line_info.hard_line_no
                    < self.top_visible_line.hard_line_info.hard_line_no
                        + self.no_of_hard_lines_queried
            {
                self.buffer_start_index = previous.buffer_start_index;
                self.index = previous.index;
                self.top_visible_line = previous.top_visible_line;
                self.refresh_to_append_content(0);
                self.compute_position_from_index_due_to_insertion();
                self.compute_new_horizontal_start_line();
            } else {
                self.top_visible_line = previous.top_visible_line;
                self.buffer_start_index = previous
                    .top_visible_line
                    .hard_line_info
                    .start_idx_in_global_buf;
                self.index = previous.index;
                self.refresh_content_for_current_visible_window();
                self.compute_position_from_index_due_to_insertion();
                self.compute_new_horizontal_start_line();
            }
        }
    }
    fn redo(&mut self) {
        self.piece_tree.redo();
        if let Some(next) = self.redo_states.pop() {
            // The current state is the pre-edit state, so it is what a
            // subsequent undo must restore.
            self.undo_states.push(self.snapshot());
            if next.buffer_start_index >= self.buffer_start_index
                && next.top_visible_line.hard_line_info.hard_line_no
                    < self.top_visible_line.hard_line_info.hard_line_no
                        + self.no_of_hard_lines_queried
            {
                self.buffer_start_index = next.buffer_start_index;
                self.index = next.index;
                self.top_visible_line = next.top_visible_line;
                self.refresh_to_append_content(0);
                self.compute_position_from_index_due_to_insertion();
                self.compute_new_horizontal_start_line();
            } else {
                self.top_visible_line = next.top_visible_line;
                self.buffer_start_index =
                    next.top_visible_line.hard_line_info.start_idx_in_global_buf;
                self.index = next.index;
                self.refresh_content_for_current_visible_window();
                self.compute_position_from_index_due_to_insertion();
                self.compute_new_horizontal_start_line();
            }
        }
    }
    fn delete(&mut self, offset: usize, length: usize) {
        let previous_lf_count = self.piece_tree.get_line_feed_count();
        self.piece_tree.delete(offset, length);
        let current_lf_count = self.piece_tree.get_line_feed_count();
        let removed_lines_count = previous_lf_count - current_lf_count;
        self.index = offset;
        if self.index >= self.buffer_start_index {
            if self.index >= self.top_visible_line.hard_line_info.start_idx_in_global_buf {
                self.refresh_to_append_content(0);
                self.compute_position_from_index_due_to_deletion();
            } else {
                self.compute_position_from_index_due_to_deletion();
                self.refresh_to_append_content(0);
            }
        } else {
            let mut no_of_hard_lines_passed = 0;
            for render_line in self
                .render_lines
                .iter()
                .skip(self.top_visible_line.pos_into_render_lines)
                .take(self.row_number)
            {
                match render_line.line_type {
                    LineType::Independent => {
                        no_of_hard_lines_passed += 1;
                    }
                    LineType::End => {
                        no_of_hard_lines_passed += 1;
                    }
                    LineType::Empty => {
                        no_of_hard_lines_passed += 1;
                    }
                    _ => {}
                }
            }
            let new_top_line = get_new_top_line(
                self.top_visible_line.hard_line_info.hard_line_no as i32,
                self.window_height as i32,
                (self.top_visible_line.hard_line_info.hard_line_no + no_of_hard_lines_passed
                    - removed_lines_count) as i32,
                0,
            );
            self.buffer_start_index = self.piece_tree.get_start_offset_of_a_line(new_top_line);
            self.top_visible_line = TopVisibleLine {
                pos_into_render_lines: 0,
                hard_line_info: HardLineInfo {
                    start_idx_in_global_buf: self.buffer_start_index,
                    hard_line_no: new_top_line,
                    pos_into_render_lines: 0,
                },
            };
            self.refresh_content_for_current_visible_window();
            self.compute_position_from_index_due_to_deletion();
        }
        self.compute_new_horizontal_start_line();
        self.syntax_tree.edit(&InputEdit {
            start_byte: offset,
            old_end_byte: offset + length,
            new_end_byte: offset,
            start_position: Point::new(0, 0),
            old_end_position: Point::new(0, 0),
            new_end_position: Point::new(0, 0),
        });
        self.syntax_tree = self
            .parser
            .parse_with_options(
                &mut |byte_offset: usize, _position: Point| -> &[u8] {
                    self.piece_tree.get_bytes_starting_at(byte_offset)
                },
                Some(&self.syntax_tree),
                None,
            )
            .unwrap();
    }
    fn insert(&mut self, offset: usize, new_content: &str) {
        let previous_lf_count = self.piece_tree.get_line_feed_count();
        self.piece_tree.insert(new_content, offset);
        let current_lf_count = self.piece_tree.get_line_feed_count();
        let new_hard_lines_added = current_lf_count - previous_lf_count;
        self.index = offset + new_content.len();
        self.refresh_to_append_content(new_hard_lines_added);
        self.compute_position_from_index_due_to_insertion();
        self.compute_new_horizontal_start_line();
        self.syntax_tree.edit(&InputEdit {
            start_byte: offset,
            old_end_byte: offset,
            new_end_byte: offset + new_content.len(),
            start_position: Point::new(0, 0),
            old_end_position: Point::new(0, 0),
            new_end_position: Point::new(0, 0),
        });
        self.syntax_tree = self
            .parser
            .parse_with_options(
                &mut |byte_offset: usize, _position: Point| -> &[u8] {
                    self.piece_tree.get_bytes_starting_at(byte_offset)
                },
                Some(&self.syntax_tree),
                None,
            )
            .unwrap();
    }
    fn refresh_to_append_content(&mut self, new_hard_lines_created: usize) {
        self.buffer.truncate(
            self.top_visible_line.hard_line_info.start_idx_in_global_buf - self.buffer_start_index,
        );
        let buffer_before_append_len = self.buffer.len();
        self.piece_tree.get_lines_text(
            self.top_visible_line.hard_line_info.hard_line_no,
            self.window_height + new_hard_lines_created,
            &mut self.buffer,
        );
        self.render_lines
            .truncate(self.top_visible_line.hard_line_info.pos_into_render_lines);
        self.no_of_hard_lines_queried = 0;
        if self.wrap {
            self.compute_extra_line_starts_with_wrap(buffer_before_append_len);
        } else {
            self.compute_extra_line_starts_with_no_wrap(buffer_before_append_len);
        }
    }
    fn refresh_content_for_current_visible_window(&mut self) {
        self.buffer.clear();
        self.piece_tree.get_lines_text(
            self.top_visible_line.hard_line_info.hard_line_no,
            self.window_height,
            &mut self.buffer,
        );
        self.render_lines.clear();
        if self.wrap {
            self.compute_line_starts_with_wrap();
        } else {
            self.compute_line_starts_with_no_wrap();
        }
    }

    fn get_query_cursor(&self) -> (QueryCursor, usize) {
        let sum_b4_visbiblity: usize = self
            .render_lines
            .iter()
            .skip(self.top_visible_line.hard_line_info.pos_into_render_lines)
            .take(
                self.top_visible_line.pos_into_render_lines
                    - self.top_visible_line.hard_line_info.pos_into_render_lines,
            )
            .map(|line| line.length)
            .sum();
        let start =
            self.top_visible_line.hard_line_info.start_idx_in_global_buf + sum_b4_visbiblity;
        let mut end = start;
        let mut no_of_hard_lines_crossed = 0;
        let mut i = self.top_visible_line.pos_into_render_lines;
        while no_of_hard_lines_crossed < self.window_height && end < self.piece_tree.get_tree_len()
        {
            match &self.render_lines[i].line_type {
                LineType::Independent => {
                    no_of_hard_lines_crossed += 1;
                }
                LineType::End => {
                    no_of_hard_lines_crossed += 1;
                }
                LineType::Empty => {
                    no_of_hard_lines_crossed += 1;
                }
                _ => {}
            }

            end += self.render_lines[i].length;
            i += 1;
        }
        let mut cursor = QueryCursor::new();
        cursor.set_byte_range(start..end);
        (cursor, start)
    }

    fn request_more_lines(&mut self) {
        let previous_buffer_len = self.buffer.len();
        self.piece_tree.get_lines_text(
            self.top_visible_line.hard_line_info.hard_line_no + self.no_of_hard_lines_queried,
            self.window_height,
            &mut self.buffer,
        );
        if self.wrap {
            self.compute_extra_line_starts_with_wrap(previous_buffer_len);
        } else {
            self.compute_extra_line_starts_with_no_wrap(previous_buffer_len);
        }
    }
    fn compute_index_from_row_and_col(&mut self) {
        let mut sum = 0;
        let current_pos_into_lines = self.top_visible_line.pos_into_render_lines + self.row_number;
        let mut i = self.top_visible_line.hard_line_info.pos_into_render_lines;
        while i < current_pos_into_lines {
            sum += self.render_lines[i].length;
            i += 1;
        }
        let current_line = self.render_lines[current_pos_into_lines];
        let (before_cursor, before_cursor_width) = (&self.buffer
            [current_line.start_in_buffer..current_line.start_in_buffer + current_line.length])
            .unicode_truncate(self.line_column);
        sum += before_cursor.len();
        let end = current_line.start_in_buffer + current_line.length;
        let suffix_width = current_line.width - self.start_of_line;
        let (_, suffix_width_truncated) =
            (&self.buffer[current_line.start_in_buffer..end]).unicode_truncate_start(suffix_width);
        if suffix_width_truncated < suffix_width {
            self.column_number -= suffix_width - suffix_width_truncated;
        }
        self.line_column = before_cursor_width;
        self.index = sum + self.top_visible_line.hard_line_info.start_idx_in_global_buf;
    }
    fn compute_position_from_index_due_to_insertion(&mut self) {
        let mut sum = 0;
        let mut current_position = 0;
        let local_index = self.index - self.top_visible_line.hard_line_info.start_idx_in_global_buf;
        let mut hard_lines_passed = 0;
        let last_pos = self.render_lines.len() - 1;
        for (pos, line) in self
            .render_lines
            .iter()
            .enumerate()
            .skip(self.top_visible_line.hard_line_info.pos_into_render_lines)
        {
            let line_end = sum + line.length;
            if local_index < line_end {
                break;
            }
            if local_index == line_end && pos == last_pos {
                break;
            }
            sum = line_end;
            if matches!(line.line_type, LineType::Independent | LineType::End) {
                hard_lines_passed += 1;
            }
            current_position += 1;
        }
        let curr_pos_into_lines =
            self.top_visible_line.hard_line_info.pos_into_render_lines + current_position;
        self.row_number = curr_pos_into_lines - self.top_visible_line.pos_into_render_lines;
        let current_line = self.render_lines[curr_pos_into_lines];
        let offset_in_line = local_index - sum;
        if matches!(current_line.line_type, LineType::Empty) {
            self.line_column = 0;
        } else {
            self.line_column = (&self.buffer
                [current_line.start_in_buffer..current_line.start_in_buffer + offset_in_line])
                .width();
        }

        if self.row_number >= self.window_height {
            self.top_visible_line.pos_into_render_lines =
                self.row_number + self.top_visible_line.pos_into_render_lines;
            self.top_visible_line.hard_line_info.hard_line_no += hard_lines_passed;
            self.top_visible_line.hard_line_info.start_idx_in_global_buf = self
                .piece_tree
                .get_start_offset_of_a_line(self.top_visible_line.hard_line_info.hard_line_no);
            let mut i = self.top_visible_line.pos_into_render_lines;
            loop {
                match self.render_lines[i].line_type {
                    LineType::Independent => {
                        break;
                    }
                    LineType::Start => {
                        break;
                    }
                    LineType::Between => {
                        i -= 1;
                    }
                    LineType::End => {
                        i -= 1;
                    }
                    LineType::Empty => {
                        break;
                    }
                }
            }
            self.top_visible_line.hard_line_info.pos_into_render_lines = i;

            self.row_number = 0;
        }
        self.col_hint = self.line_column;
    }
    fn compute_position_from_index_due_to_deletion(&mut self) {
        let mut sum = 0;
        let mut current_position = 0;
        let mut hard_lines_passed = 0;
        let mut i = self.top_visible_line.hard_line_info.pos_into_render_lines;
        if self.index < self.top_visible_line.hard_line_info.start_idx_in_global_buf {
            let mut back_offset = self.top_visible_line.hard_line_info.start_idx_in_global_buf;
            match self.render_lines[i].line_type {
                LineType::Independent => {
                    hard_lines_passed += 1;
                }
                LineType::Start => {
                    hard_lines_passed += 1;
                }
                LineType::Between => {}
                LineType::End => {}
                LineType::Empty => {
                    hard_lines_passed += 1;
                }
            }
            i -= 1;
            loop {
                back_offset -= self.render_lines[i].length;
                if back_offset <= self.index {
                    break;
                }
                match self.render_lines[i].line_type {
                    LineType::Independent => {
                        hard_lines_passed += 1;
                    }
                    LineType::Start => {
                        hard_lines_passed += 1;
                    }
                    LineType::Between => {}
                    LineType::End => {}
                    LineType::Empty => {
                        hard_lines_passed += 1;
                    }
                }
                i -= 1;
            }
            loop {
                match self.render_lines[i].line_type {
                    LineType::Independent => {
                        break;
                    }
                    LineType::Start => {
                        break;
                    }
                    LineType::Between => {
                        i -= 1;
                    }
                    LineType::End => {
                        i -= 1;
                    }
                    LineType::Empty => {
                        break;
                    }
                }
            }
            self.top_visible_line.hard_line_info.hard_line_no -= hard_lines_passed;
            self.top_visible_line.hard_line_info.pos_into_render_lines = i;
            self.top_visible_line.hard_line_info.start_idx_in_global_buf = self
                .piece_tree
                .get_start_offset_of_a_line(self.top_visible_line.hard_line_info.hard_line_no);
        }
        let local_index = self.index - self.top_visible_line.hard_line_info.start_idx_in_global_buf;
        hard_lines_passed = 0;
        let last_pos = self.render_lines.len() - 1;
        for (pos, line) in self
            .render_lines
            .iter()
            .enumerate()
            .skip(self.top_visible_line.hard_line_info.pos_into_render_lines)
        {
            let line_end = sum + line.length;
            if local_index < line_end {
                break;
            }
            if local_index == line_end && pos == last_pos {
                break;
            }
            sum = line_end;
            if matches!(line.line_type, LineType::Independent | LineType::End) {
                hard_lines_passed += 1;
            }
            current_position += 1;
        }

        let curr_pos_into_lines = i + current_position;
        let current_line = self.render_lines[curr_pos_into_lines];
        let offset_in_line = local_index - sum;
        if matches!(current_line.line_type, LineType::Empty) {
            self.line_column = 0;
        } else {
            self.line_column = (&self.buffer
                [current_line.start_in_buffer..current_line.start_in_buffer + offset_in_line])
                .width();
        }
        let candidate_row_number =
            curr_pos_into_lines as i32 - self.top_visible_line.pos_into_render_lines as i32;
        if candidate_row_number < 0 {
            self.top_visible_line.pos_into_render_lines -= candidate_row_number.abs() as usize;
        }
        self.row_number = curr_pos_into_lines - self.top_visible_line.pos_into_render_lines;
        if self.row_number >= self.window_height {
            self.top_visible_line.pos_into_render_lines =
                self.row_number + self.top_visible_line.pos_into_render_lines;
            self.top_visible_line.hard_line_info.hard_line_no += hard_lines_passed;
            self.top_visible_line.hard_line_info.start_idx_in_global_buf = self
                .piece_tree
                .get_start_offset_of_a_line(self.top_visible_line.hard_line_info.hard_line_no);
            let mut i = self.top_visible_line.pos_into_render_lines;
            loop {
                match self.render_lines[i].line_type {
                    LineType::Independent => {
                        break;
                    }
                    LineType::Start => {
                        break;
                    }
                    LineType::Between => {
                        i -= 1;
                    }
                    LineType::End => {
                        i -= 1;
                    }
                    LineType::Empty => {
                        break;
                    }
                }
            }
            self.top_visible_line.hard_line_info.pos_into_render_lines = i;

            self.row_number = 0;
        }
        self.col_hint = self.line_column;
    }
    fn compute_new_horizontal_start_line(&mut self) {
        let end_of_line = self.start_of_line + self.window_width;
        if self.line_column < self.start_of_line {
            let gap = self.start_of_line - self.line_column;
            self.start_of_line -= gap;
        } else if self.line_column > end_of_line {
            let gap = self.line_column - end_of_line;
            self.start_of_line += gap;
        }
        self.column_number = self.line_column - self.start_of_line;
        let line = self.render_lines[self.top_visible_line.pos_into_render_lines + self.row_number];
        let end = line.start_in_buffer + line.length;
        if line.width > self.start_of_line {
            let suffix_width = line.width - self.start_of_line;
            let (_, suffix_width_truncated) =
                (&self.buffer[line.start_in_buffer..end]).unicode_truncate_start(suffix_width);
            if suffix_width_truncated < suffix_width {
                self.column_number -= suffix_width - suffix_width_truncated;
            }
        }
    }
    fn compute_line_starts_with_no_wrap(&mut self) {
        if self.buffer.is_empty() {
            self.render_lines.push(TextEditorLine {
                start_in_buffer: 0,
                length: 0,
                line_type: LineType::Empty,
                width: 0,
            });
            return;
        }
        let mut offset = 0;
        let mut no_of_hard_lines = 0;
        for line in self.buffer.lines() {
            no_of_hard_lines += 1;
            let txt_line = TextEditorLine {
                start_in_buffer: offset,
                length: line.len(),
                line_type: LineType::Independent,
                width: line.width(),
            };
            self.render_lines.push(txt_line);
            offset += line.len();
            if offset < self.buffer.len() && self.buffer.as_bytes()[offset] == b'\n' {
                if let Some(last) = self.render_lines.last_mut() {
                    last.length += 1;
                }
                offset += 1;
            } else if offset < self.buffer.len() && self.buffer.as_bytes()[offset] == b'\r' {
                offset += 2;
                if let Some(last) = self.render_lines.last_mut() {
                    last.length += 2;
                }
            }
        }
        self.no_of_hard_lines_queried = no_of_hard_lines;
        if self.buffer.as_bytes()[self.buffer.len() - 1] == b'\n'
            && self.buffer_start_index + self.buffer.len() == self.piece_tree.get_tree_len()
        {
            let txt_line = TextEditorLine {
                start_in_buffer: 0,
                length: 0,
                line_type: LineType::Empty,
                width: 0,
            };
            self.render_lines.push(txt_line);
        }
    }

    fn compute_line_starts_with_wrap(&mut self) {
        if self.buffer.is_empty() {
            self.render_lines.push(TextEditorLine {
                start_in_buffer: 0,
                length: 0,
                line_type: LineType::Empty,
                width: 0,
            });
            return;
        }
        let mut offset = 0;
        let mut no_of_hard_lines = 0;
        for line in self.buffer.lines() {
            no_of_hard_lines += 1;
            let line_width = line.width();
            if line_width <= self.window_width {
                let txt_line = TextEditorLine {
                    start_in_buffer: offset,
                    length: line.len(),
                    line_type: LineType::Independent,
                    width: line.width(),
                };
                self.render_lines.push(txt_line);
                offset += line.len();
            } else {
                let options = Options::new(self.window_width).preserve_trailing_space(true);
                let broken_lines = wrap(line, &options);
                let mut inner_offset = offset;
                let broken_lines_len = broken_lines.len();
                for (i, broken_line) in broken_lines.iter().enumerate() {
                    let line_type = if i == 0 {
                        LineType::Start
                    } else if i == broken_lines_len - 1 {
                        LineType::End
                    } else {
                        LineType::Between
                    };
                    let txt_line = TextEditorLine {
                        start_in_buffer: inner_offset,
                        length: broken_line.len(),
                        line_type,
                        width: broken_line.width(),
                    };
                    self.render_lines.push(txt_line);
                    inner_offset += broken_line.len();
                }
                offset += line.len();
            }

            if offset < self.buffer.len() && self.buffer.as_bytes()[offset] == b'\n' {
                if let Some(last) = self.render_lines.last_mut() {
                    last.length += 1;
                }
                offset += 1;
            } else if offset < self.buffer.len() && self.buffer.as_bytes()[offset] == b'\r' {
                offset += 2;
                if let Some(last) = self.render_lines.last_mut() {
                    last.length += 2;
                }
            }
        }
        self.no_of_hard_lines_queried = no_of_hard_lines;
        if self.buffer.as_bytes()[self.buffer.len() - 1] == b'\n'
            && self.buffer_start_index + self.buffer.len() == self.piece_tree.get_tree_len()
        {
            let txt_line = TextEditorLine {
                start_in_buffer: 0,
                length: 0,
                line_type: LineType::Empty,
                width: 0,
            };
            self.render_lines.push(txt_line);
        }
    }
    fn compute_extra_line_starts_with_no_wrap(&mut self, previous_buffer_len: usize) {
        let mut offset = previous_buffer_len;
        let mut no_of_hard_lines = 0;
        let buf = &self.buffer[previous_buffer_len..];
        if buf.is_empty() {
            self.render_lines.push(TextEditorLine {
                start_in_buffer: 0,
                length: 0,
                line_type: LineType::Empty,
                width: 0,
            });
            return;
        }
        for line in buf.lines() {
            no_of_hard_lines += 1;
            let txt_line = TextEditorLine {
                start_in_buffer: offset,
                length: line.len(),
                line_type: LineType::Independent,
                width: line.width(),
            };
            self.render_lines.push(txt_line);
            offset += line.len();
            if offset < self.buffer.len() && self.buffer.as_bytes()[offset] == b'\n' {
                if let Some(last) = self.render_lines.last_mut() {
                    last.length += 1;
                }
                offset += 1;
            } else if offset < self.buffer.len() && self.buffer.as_bytes()[offset] == b'\r' {
                offset += 2;
                if let Some(last) = self.render_lines.last_mut() {
                    last.length += 2;
                }
            }
        }
        self.no_of_hard_lines_queried += no_of_hard_lines;
        if self.buffer.as_bytes()[self.buffer.len() - 1] == b'\n'
            && self.top_visible_line.hard_line_info.start_idx_in_global_buf + buf.len()
                == self.piece_tree.get_tree_len()
        {
            let txt_line = TextEditorLine {
                start_in_buffer: 0,
                length: 0,
                line_type: LineType::Empty,
                width: 0,
            };
            self.render_lines.push(txt_line);
        }
    }
    fn compute_extra_line_starts_with_wrap(&mut self, previous_buffer_len: usize) {
        let mut offset = previous_buffer_len;
        let mut no_of_hard_lines = 0;
        let buf = &self.buffer[previous_buffer_len..];
        if buf.is_empty() {
            self.render_lines.push(TextEditorLine {
                start_in_buffer: 0,
                length: 0,
                line_type: LineType::Empty,
                width: 0,
            });
            return;
        }
        for line in buf.lines() {
            no_of_hard_lines += 1;
            let line_width = line.width();
            if line_width <= self.window_width {
                let txt_line = TextEditorLine {
                    start_in_buffer: offset,
                    length: line.len(),
                    line_type: LineType::Independent,
                    width: line.width(),
                };
                self.render_lines.push(txt_line);
                offset += line.len();
            } else {
                let options = Options::new(self.window_width).preserve_trailing_space(true);
                let broken_lines = wrap(line, &options);
                let mut inner_offset = offset;
                let broken_lines_len = broken_lines.len();
                for (i, broken_line) in broken_lines.iter().enumerate() {
                    let line_type = if i == 0 {
                        LineType::Start
                    } else if i == broken_lines_len - 1 {
                        LineType::End
                    } else {
                        LineType::Between
                    };
                    let line_width = broken_line.width();
                    let txt_line = TextEditorLine {
                        start_in_buffer: inner_offset,
                        length: broken_line.len(),
                        line_type,
                        width: line_width,
                    };
                    self.render_lines.push(txt_line);
                    inner_offset += broken_line.len();
                }
                offset += line.len();
            }

            if offset < self.buffer.len() && self.buffer.as_bytes()[offset] == b'\n' {
                if let Some(last) = self.render_lines.last_mut() {
                    last.length += 1;
                }
                offset += 1;
            } else if offset < self.buffer.len() && self.buffer.as_bytes()[offset] == b'\r' {
                offset += 2;
                if let Some(last) = self.render_lines.last_mut() {
                    last.length += 2;
                }
            }
        }
        self.no_of_hard_lines_queried += no_of_hard_lines;
        if self.buffer.as_bytes()[self.buffer.len() - 1] == b'\n'
            && self.top_visible_line.hard_line_info.start_idx_in_global_buf + buf.len()
                == self.piece_tree.get_tree_len()
        {
            let txt_line = TextEditorLine {
                start_in_buffer: 0,
                length: 0,
                line_type: LineType::Empty,
                width: 0,
            };
            self.render_lines.push(txt_line);
        }
    }
    fn text_wrap<'a>(&self, line: &'a str) -> Vec<&'a str> {
        let segmenter = GraphemeClusterSegmenter::new();
        let mut breakpoints = segmenter.segment_str(line);
        let mut previous_end_line = 0;
        let mut end_of_current_line = 0 + self.window_width - 1;
        let mut broken_lines: Vec<&str> = Vec::with_capacity((line.len() / self.window_width) + 1);
        let mut previous_break_point = 0;
        while let Some(breakpoint) = breakpoints.next() {
            if breakpoint == line.len() - 1 {
                broken_lines.push(&line[previous_end_line + 1..line.len()]);
                break;
            }
            if breakpoint <= end_of_current_line {
                continue;
            }
            previous_end_line = end_of_current_line;
            end_of_current_line = breakpoint + self.window_width - 1;
            broken_lines.push(&line[breakpoint - self.window_width..breakpoint]);
        }
        broken_lines
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

#[derive(Debug, Default)]
enum Mode {
    #[default]
    Normal,
    Editing,
    Exiting,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum LineType {
    Independent,
    Start,
    Between,
    End,
    Empty,
}

struct UndoRedoState {
    top_visible_line: TopVisibleLine,
    buffer_start_index: usize,
    index: usize,
}
#[derive(Clone, Copy, Debug)]
pub struct TextEditorLine {
    start_in_buffer: usize,
    length: usize,
    line_type: LineType,
    width: usize,
}

struct CursorTextIter<'a> {
    piece_tree: &'a PieceTree,
    current_pos: usize,
    end: usize,
}
impl<'a> Iterator for CursorTextIter<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_pos < self.end {
            let piece_bytes = self.piece_tree.get_bytes_starting_at(self.current_pos);
            if piece_bytes.is_empty() {
                return None;
            }
            let distance_to_end = self.end - self.current_pos;
            let distance_taken = min(distance_to_end, piece_bytes.len());
            self.current_pos += distance_taken;
            Some(&piece_bytes[..distance_taken])
        } else {
            return None;
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

pub const DEFAULT_STYLE: Style = Style::new().fg(Color::Rgb(0xCD, 0xD6, 0xF4));
pub const EDITOR_BG: Color = Color::Rgb(0x1E, 0x1E, 0x2E);

pub fn style_for_capture(name: &str) -> Style {
    match name {
        "comment" => Style::new()
            .fg(Color::Rgb(0x6C, 0x70, 0x86))
            .add_modifier(Modifier::ITALIC),
        "comment.documentation" => Style::new()
            .fg(Color::Rgb(0xA6, 0xAD, 0xC8))
            .add_modifier(Modifier::ITALIC),
        "keyword" => Style::new()
            .fg(Color::Rgb(0xCB, 0xA6, 0xF7))
            .add_modifier(Modifier::BOLD),
        "function" | "function.method" => Style::new().fg(Color::Rgb(0x89, 0xB4, 0xFA)),
        "function.builtin" => Style::new().fg(Color::Rgb(0x74, 0xC7, 0xEC)),
        "function.macro" => Style::new().fg(Color::Rgb(0x94, 0xE2, 0xD5)),
        "type" | "type.builtin" => Style::new().fg(Color::Rgb(0xF9, 0xE2, 0xAF)),
        "constructor" | "variable.builtin" => Style::new().fg(Color::Rgb(0xF3, 0x8B, 0xA8)),
        "constant" | "constant.builtin" => Style::new().fg(Color::Rgb(0xFA, 0xB3, 0x87)),
        "string" => Style::new().fg(Color::Rgb(0xA6, 0xE3, 0xA1)),
        "escape" | "attribute" | "punctuation.special" => {
            Style::new().fg(Color::Rgb(0xF5, 0xC2, 0xE7))
        }
        "property" => Style::new().fg(Color::Rgb(0xB4, 0xBE, 0xFE)),
        "variable.parameter" => Style::new()
            .fg(Color::Rgb(0xEB, 0xA0, 0xAC))
            .add_modifier(Modifier::ITALIC),
        "label" => Style::new()
            .fg(Color::Rgb(0xF2, 0xCD, 0xCD))
            .add_modifier(Modifier::ITALIC),
        "operator" => Style::new().fg(Color::Rgb(0x89, 0xDC, 0xEB)),
        "punctuation.bracket" | "punctuation.delimiter" => {
            Style::new().fg(Color::Rgb(0x93, 0x99, 0xB2))
        }
        _ => match name.split('.').next() {
            Some(base) if base != name => style_for_capture(base),
            _ => DEFAULT_STYLE,
        },
    }
}

pub fn build_capture_styles(query: &Query) -> Vec<Style> {
    query
        .capture_names()
        .iter()
        .map(|name| style_for_capture(name))
        .collect()
}
