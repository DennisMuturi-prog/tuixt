use core::panic;
use std::{cmp::min, rc::Rc};

#[derive(Debug)]
pub struct PieceTree {
    original: String,
    original_line_starts: Vec<usize>,
    add: String,
    add_line_starts: Vec<usize>,
    root: Option<Rc<Node>>,
    undo_stack: Vec<Option<Rc<Node>>>,
    redo_stack: Vec<Option<Rc<Node>>>,
    black_leaf: Rc<Node>,
    double_black_leaf: Rc<Node>,
    last_change_in_buffer: BufferPosition,
}
impl Default for PieceTree {
    fn default() -> Self {
        Self::new("")
    }
}
impl PieceTree {
    pub fn new(original_content: &str) -> Self {
        if original_content.is_empty() {
            Self {
                original: String::new(),
                add: String::new(),
                root: None,
                undo_stack: Vec::new(),
                redo_stack: Vec::new(),
                original_line_starts: Vec::new(),
                add_line_starts: vec![0],
                double_black_leaf: Rc::new(Node::new_double_black_leaf()),
                black_leaf: Rc::new(Node::new_black_leaf()),
                last_change_in_buffer: BufferPosition::default(),
            }
        } else {
            let mut original_line_starts = Vec::new();
            original_line_starts.push(0);
            let start = BufferPosition { line: 0, column: 0 };
            find_line_starts(original_content, 0, &mut original_line_starts);
            let original = String::from(original_content);
            let end = BufferPosition {
                line: original_line_starts.len() - 1,
                column: original.len() - original_line_starts[original_line_starts.len() - 1],
            };
            let line_feed_count =
                calculate_line_feed_count(&original, &original_line_starts, start, end);
            let black_leaf = Rc::new(Node::new_black_leaf());
            let root = Some(Rc::new(Node::new(
                start,
                end,
                line_feed_count,
                original.len(),
                BufferType::Original,
                Color::Black,
                Some(black_leaf.clone()),
                Some(black_leaf.clone()),
            )));

            Self {
                original,
                add: String::new(),
                root,
                undo_stack: Vec::new(),
                redo_stack: Vec::new(),
                black_leaf,
                double_black_leaf: Rc::new(Node::new_double_black_leaf()),
                last_change_in_buffer: BufferPosition::default(),
                original_line_starts,
                add_line_starts: vec![0],
            }
        }
    }
    pub fn get_text(&self, content: &mut String) {
        if let Some(root_node) = self.root.as_ref() {
            self.in_order_traversal(root_node, content);
        }
    }
    pub fn get_sub_text(&self, content: &mut String, start_index: usize, length: usize) {
        if let Some(root_node) = self.root.as_ref() {
            self.get_sub_text_helper(root_node, content, start_index, length);
        }
    }
    pub fn get_line_text(&self, line_number: usize, content: &mut String) {
        if let Some(root_node) = self.root.as_ref() {
            let mut descent_to_line =
                self.node_at_start_of_line(root_node.clone(), line_number, content);
            if !descent_to_line.should_continue {
                return;
            }
            let mut has_next = self.next(&mut descent_to_line.path);
            let mut current_node = Self::last(&descent_to_line.path);
            while has_next {
                let buffer = match current_node.buffer_type {
                    BufferType::Original => &self.original,
                    BufferType::Add => &self.add,
                };
                let line_starts = match current_node.buffer_type {
                    BufferType::Original => &self.original_line_starts,
                    BufferType::Add => &self.add_line_starts,
                };
                if current_node.line_feed_count == 0 {
                    let line_start =
                        line_starts[current_node.start.line] + current_node.start.column;
                    let line_end = line_starts[current_node.end.line] + current_node.end.column;
                    content.push_str(&buffer[line_start..line_end]);
                    has_next = self.next(&mut descent_to_line.path);
                    current_node = Self::last(&descent_to_line.path);
                } else {
                    let line_start =
                        line_starts[current_node.start.line] + current_node.start.column;
                    let line_end = if current_node.start.line + 1 < line_starts.len() {
                        line_starts[current_node.start.line + 1]
                    } else {
                        buffer.len()
                    };
                    content.push_str(&buffer[line_start..line_end]);
                    break;
                }
            }
        }
    }
    fn node_at_start_of_line(
        &self,
        root: Rc<Node>,
        line_number: usize,
        content: &mut String,
    ) -> LineDescent {
        let mut line_number = line_number;
        let mut current_node = root;
        let mut path: Vec<Rc<Node>> = Vec::new();
        if line_number == 0 {
            let mut dest = current_node.clone();
            while current_node != self.black_leaf {
                path.push(current_node.clone());
                dest = current_node.clone();
                current_node = current_node.left.as_ref().unwrap().clone();
            }
            let buffer = match dest.buffer_type {
                BufferType::Original => &self.original,
                BufferType::Add => &self.add,
            };
            let line_starts = match dest.buffer_type {
                BufferType::Original => &self.original_line_starts,
                BufferType::Add => &self.add_line_starts,
            };
            if dest.line_feed_count == 0 {
                let line_start = line_starts[dest.start.line] + dest.start.column;
                let line_end = line_starts[dest.end.line] + dest.end.column;
                content.push_str(&buffer[line_start..line_end]);
                return LineDescent {
                    should_continue: true,
                    path,
                };
            } else {
                let line_start = line_starts[dest.start.line] + dest.start.column;
                let line_end = if dest.start.line + 1 < line_starts.len() {
                    line_starts[dest.start.line + 1]
                } else {
                    buffer.len()
                };
                content.push_str(&buffer[line_start..line_end]);
                return LineDescent {
                    should_continue: false,
                    path,
                };
            }
        }
        while current_node != self.black_leaf {
            path.push(current_node.clone());
            if line_number <= current_node.left_subtree_line_feed_count {
                current_node = current_node.left.as_ref().unwrap().clone();
            } else if current_node.left_subtree_line_feed_count + current_node.line_feed_count
                >= line_number
            {
                let offset_in_node = line_number - current_node.left_subtree_line_feed_count;
                let line = current_node.start.line + offset_in_node;
                let buffer = match current_node.buffer_type {
                    BufferType::Original => &self.original,
                    BufferType::Add => &self.add,
                };
                let line_starts = match current_node.buffer_type {
                    BufferType::Original => &self.original_line_starts,
                    BufferType::Add => &self.add_line_starts,
                };
                if line_number
                    < current_node.left_subtree_line_feed_count + current_node.line_feed_count
                {
                    let line_start = line_starts[line];
                    let line_end = if line + 1 < line_starts.len() {
                        line_starts[line + 1]
                    } else {
                        buffer.len()
                    };
                    content.push_str(&buffer[line_start..line_end]);
                    return LineDescent {
                        should_continue: false,
                        path,
                    };
                } else {
                    let line_start = line_starts[line];
                    let line_end = line_starts[current_node.end.line] + current_node.end.column;
                    content.push_str(&buffer[line_start..line_end]);
                    return LineDescent {
                        should_continue: true,
                        path,
                    };
                }
            } else {
                line_number -=
                    current_node.left_subtree_line_feed_count + current_node.line_feed_count;
                current_node = current_node.right.as_ref().unwrap().clone();
            }
        }

        LineDescent {
            should_continue: false,
            path,
        }
    }
    pub fn get_lines_text(&self,start_line_num:usize,num_of_lines:usize,content: &mut String){

    }
    fn get_sub_text_helper(
        &self,
        current_node: &Rc<Node>,
        content: &mut String,
        index: usize,
        length: usize,
    ) {
        if current_node == &self.black_leaf {
            return;
        }
        let mut index = index;
        let mut length = length;
        if index == 0 && length >= current_node.left_subtree_len {
            self.in_order_traversal(current_node.left.as_ref().unwrap(), content);
            length = length.saturating_sub(current_node.left_subtree_len);
            index = current_node.left_subtree_len;
        } else if index < current_node.left_subtree_len {
            self.get_sub_text_helper(current_node.left.as_ref().unwrap(), content, index, length);
            let length_taken = min(current_node.left_subtree_len - index, length);
            length -= length_taken;
            index = current_node.left_subtree_len;
        }
        if length == 0 {
            return;
        }
        if index < current_node.left_subtree_len + current_node.length {
            let offset_in_node = index - current_node.left_subtree_len;
            let content_str = match current_node.buffer_type {
                BufferType::Original => {
                    let start_offset = self.original_line_starts[current_node.start.line]
                        + current_node.start.column;
                    &self.original[start_offset + offset_in_node
                        ..min(
                            start_offset + offset_in_node + length,
                            start_offset + current_node.length,
                        )]
                }
                BufferType::Add => {
                    let start_offset =
                        self.add_line_starts[current_node.start.line] + current_node.start.column;
                    &self.add[start_offset + offset_in_node
                        ..min(
                            start_offset + offset_in_node + length,
                            start_offset + current_node.length,
                        )]
                }
            };
            content.push_str(content_str);
            if length <= current_node.length - offset_in_node {
                return;
            }
            length -= current_node.length - offset_in_node;
            index = current_node.left_subtree_len + current_node.length;
        }
        if length == 0 {
            return;
        }

        index -= current_node.left_subtree_len + current_node.length;
        let right_subtree_len =
            current_node.subtree_len - current_node.left_subtree_len - current_node.length;
        if index == 0 && length >= right_subtree_len {
            self.in_order_traversal(current_node.right.as_ref().unwrap(), content);
        } else {
            self.get_sub_text_helper(current_node.right.as_ref().unwrap(), content, index, length);
        }
    }
    pub fn undo(&mut self) {
        if let Some(replacement_root) = self.undo_stack.pop() {
            self.redo_stack.push(self.root.clone());
            self.root = replacement_root;
        };
    }
    pub fn redo(&mut self) {
        if let Some(replacement_root) = self.redo_stack.pop() {
            self.undo_stack.push(self.root.clone());
            self.root = replacement_root;
        };
    }

    fn in_order_traversal(&self, node: &Rc<Node>, content: &mut String) {
        if node == &self.black_leaf {
            return;
        }
        self.in_order_traversal(node.left.as_ref().unwrap(), content);
        let content_str = match node.buffer_type {
            BufferType::Original => {
                let start_offset = self.original_line_starts[node.start.line] + node.start.column;
                let end_offset = self.original_line_starts[node.end.line] + node.end.column;
                &self.original[start_offset..end_offset]
            }
            BufferType::Add => {
                let start_offset = self.add_line_starts[node.start.line] + node.start.column;
                let end_offset = self.add_line_starts[node.end.line] + node.end.column;
                &self.add[start_offset..end_offset]
            }
        };
        content.push_str(content_str);
        self.in_order_traversal(node.right.as_ref().unwrap(), content);
    }
    fn remove_right_most(&self, node: Rc<Node>) -> Rc<Node> {
        if node.right.as_ref().unwrap().clone() == self.black_leaf {
            let left = node.left.as_ref().unwrap().clone();
            if left == self.black_leaf {
                if node.color == Color::Black {
                    return self.double_black_leaf.clone();
                } else {
                    return self.black_leaf.clone();
                }
            } else {
                if node.color == Color::Black {
                    let new_node = Rc::new(Node::new(
                        left.start,
                        left.end,
                        left.line_feed_count,
                        left.length,
                        left.buffer_type,
                        left.color.plus_black(),
                        left.left.clone(),
                        left.right.clone(),
                    ));
                    return new_node;
                } else {
                    return left;
                }
            }
        }
        let new_right = self.remove_right_most(node.right.as_ref().unwrap().clone());
        let new_current_node = Rc::new(Node::new(
            node.start,
            node.end,
            node.line_feed_count,
            node.length,
            node.buffer_type,
            node.color,
            node.left.clone(),
            Some(new_right),
        ));
        self.bubble(new_current_node)
    }
    fn find_right_most(&self, node: Rc<Node>) -> NodeInfo {
        let mut current = node;
        let mut dest = current.clone();
        while current != self.black_leaf {
            dest = current.clone();
            current = current.right.as_ref().unwrap().clone();
        }
        NodeInfo {
            start: dest.start,
            end: dest.end,
            line_feed_count: dest.line_feed_count,
            length: dest.length,
            buffer_type: dest.buffer_type,
        }
    }
    fn node_at(&self, root: Rc<Node>, offset: usize) -> NodePosition {
        let mut offset = offset;
        let mut start_offset = 0;
        let mut current_node = root;
        let mut path: Vec<Rc<Node>> = Vec::new();
        while current_node != self.black_leaf {
            path.push(current_node.clone());
            if offset < current_node.left_subtree_len {
                current_node = current_node.left.as_ref().unwrap().clone();
            } else if current_node.left_subtree_len + current_node.length >= offset {
                let offset_in_node = offset - current_node.left_subtree_len;
                return NodePosition {
                    start_offset: start_offset + current_node.left_subtree_len,
                    remainder: offset_in_node,
                    path,
                };
            } else {
                offset -= current_node.left_subtree_len + current_node.length;
                start_offset += current_node.left_subtree_len + current_node.length;
                current_node = current_node.right.as_ref().unwrap().clone();
            }
        }
        NodePosition {
            start_offset,
            remainder: 0,
            path,
        }
    }
    fn insert_at(
        &self,
        current_node: Rc<Node>,
        offset: usize,
        node_to_insert: Rc<Node>,
    ) -> Rc<Node> {
        if current_node == self.black_leaf {
            return node_to_insert;
        }
        if offset <= current_node.left_subtree_len {
            let left_path = self.insert_at(
                current_node.left.as_ref().unwrap().clone(),
                offset,
                node_to_insert,
            );
            let new_current_node = Rc::new(Node::new(
                current_node.start,
                current_node.end,
                current_node.line_feed_count,
                current_node.length,
                current_node.buffer_type,
                current_node.color,
                Some(left_path),
                current_node.right.clone(),
            ));
            Self::rebalance(new_current_node)
        } else {
            let right_offset = offset.checked_sub(current_node.left_subtree_len + current_node.length).expect("in our current insert inserts only happen at boundaries ,so it this is occurring there is an issue");
            let right_path = self.insert_at(
                current_node.right.as_ref().unwrap().clone(),
                right_offset,
                node_to_insert,
            );
            let new_current_node = Rc::new(Node::new(
                current_node.start,
                current_node.end,
                current_node.line_feed_count,
                current_node.length,
                current_node.buffer_type,
                current_node.color,
                current_node.left.clone(),
                Some(right_path),
            ));
            Self::rebalance(new_current_node)
        }
    }
    fn delete_at(&self, current_node: Rc<Node>, offset: usize) -> Rc<Node> {
        if offset < current_node.left_subtree_len {
            let left_path = self.delete_at(current_node.left.as_ref().unwrap().clone(), offset);

            self.bubble(Rc::new(Node::new(
                current_node.start,
                current_node.end,
                current_node.line_feed_count,
                current_node.length,
                current_node.buffer_type,
                current_node.color,
                Some(left_path),
                current_node.right.clone(),
            )))
        } else if offset >= current_node.left_subtree_len + current_node.length {
            let right_offset = offset - (current_node.left_subtree_len + current_node.length);
            let right_path =
                self.delete_at(current_node.right.as_ref().unwrap().clone(), right_offset);

            self.bubble(Rc::new(Node::new(
                current_node.start,
                current_node.end,
                current_node.line_feed_count,
                current_node.length,
                current_node.buffer_type,
                current_node.color,
                current_node.left.clone(),
                Some(right_path),
            )))
        } else {
            self.remove_node(current_node)
        }
    }
    fn remove_node(&self, node: Rc<Node>) -> Rc<Node> {
        let left = node.left.clone().unwrap();
        let right = node.right.clone().unwrap();
        if left != self.black_leaf && right != self.black_leaf {
            let node_info = self.find_right_most(left.clone());
            let new_left = self.remove_right_most(left);
            let new_node = Rc::new(Node::new(
                node_info.start,
                node_info.end,
                node_info.line_feed_count,
                node_info.length,
                node_info.buffer_type,
                node.color,
                Some(new_left),
                Some(right),
            ));
            self.bubble(new_node)
        } else if left == self.black_leaf && right == self.black_leaf {
            match node.color {
                Color::NegativeBlack => self.black_leaf.clone(),
                Color::Red => self.black_leaf.clone(),
                Color::Black => self.double_black_leaf.clone(),
                Color::DoubleBlack => self.double_black_leaf.clone(),
            }
        } else {
            if right == self.black_leaf {
                if node.color == Color::Black {
                    Rc::new(Node::new(
                        left.start,
                        left.end,
                        left.line_feed_count,
                        left.length,
                        left.buffer_type,
                        left.color.plus_black(),
                        left.left.clone(),
                        left.right.clone(),
                    ))
                } else {
                    left
                }
            } else {
                if node.color == Color::Black {
                    Rc::new(Node::new(
                        right.start,
                        right.end,
                        right.line_feed_count,
                        right.length,
                        right.buffer_type,
                        right.color.plus_black(),
                        right.left.clone(),
                        right.right.clone(),
                    ))
                } else {
                    right
                }
            }
        }
    }
    fn shrink_piece_to_suffix(&self, node: &Rc<Node>, end_node_pos_remainder: usize) -> NodeInfo {
        let start_offset = match node.buffer_type {
            BufferType::Original => {
                self.original_line_starts[node.start.line]
                    + node.start.column
                    + end_node_pos_remainder
            }
            BufferType::Add => {
                self.add_line_starts[node.start.line] + node.start.column + end_node_pos_remainder
            }
        };
        let start = match node.buffer_type {
            BufferType::Original => find_nearest_start_and_column_in_it(
                &self.original_line_starts,
                node.start.line,
                start_offset,
            ),
            BufferType::Add => find_nearest_start_and_column_in_it(
                &self.add_line_starts,
                node.start.line,
                start_offset,
            ),
        };
        let buffer = match node.buffer_type {
            BufferType::Original => &self.original,
            BufferType::Add => &self.add,
        };
        let line_starts = match node.buffer_type {
            BufferType::Original => &self.original_line_starts,
            BufferType::Add => &self.add_line_starts,
        };
        let line_feed_count = calculate_line_feed_count(buffer, line_starts, start, node.end);

        NodeInfo {
            start,
            end: node.end,
            line_feed_count,
            length: node.length - end_node_pos_remainder,
            buffer_type: node.buffer_type,
        }
    }
    fn shrink_piece_to_prefix(&self, node: &Rc<Node>, start_node_pos_remainder: usize) -> NodeInfo {
        let end_offset = match node.buffer_type {
            BufferType::Original => {
                self.original_line_starts[node.start.line]
                    + node.start.column
                    + start_node_pos_remainder
            }
            BufferType::Add => {
                self.add_line_starts[node.start.line] + node.start.column + start_node_pos_remainder
            }
        };
        let end = match node.buffer_type {
            BufferType::Original => find_nearest_start_and_column_in_it(
                &self.original_line_starts,
                node.start.line,
                end_offset,
            ),
            BufferType::Add => find_nearest_start_and_column_in_it(
                &self.add_line_starts,
                node.start.line,
                end_offset,
            ),
        };
        let buffer = match node.buffer_type {
            BufferType::Original => &self.original,
            BufferType::Add => &self.add,
        };
        let line_starts = match node.buffer_type {
            BufferType::Original => &self.original_line_starts,
            BufferType::Add => &self.add_line_starts,
        };
        let line_feed_count = calculate_line_feed_count(buffer, line_starts, node.start, end);
        NodeInfo {
            start: node.start,
            end,
            line_feed_count,
            length: start_node_pos_remainder,
            buffer_type: node.buffer_type,
        }
    }
    fn expand_piece(&self, node: &Rc<Node>, start_node_pos_remainder: usize) -> NodeInfo {
        let end_offset = match node.buffer_type {
            BufferType::Original => {
                self.original_line_starts[node.end.line]
                    + node.end.column
                    + start_node_pos_remainder
            }
            BufferType::Add => {
                self.add_line_starts[node.end.line] + node.end.column + start_node_pos_remainder
            }
        };
        let end = match node.buffer_type {
            BufferType::Original => find_nearest_start_and_column_in_it(
                &self.original_line_starts,
                node.start.line,
                end_offset,
            ),
            BufferType::Add => find_nearest_start_and_column_in_it(
                &self.add_line_starts,
                node.start.line,
                end_offset,
            ),
        };
        let buffer = match node.buffer_type {
            BufferType::Original => &self.original,
            BufferType::Add => &self.add,
        };
        let line_starts = match node.buffer_type {
            BufferType::Original => &self.original_line_starts,
            BufferType::Add => &self.add_line_starts,
        };
        let line_feed_count = calculate_line_feed_count(buffer, line_starts, node.start, end);
        NodeInfo {
            start: node.start,
            end,
            line_feed_count,
            length: node.length + start_node_pos_remainder,
            buffer_type: node.buffer_type,
        }
    }

    pub fn delete(&mut self, offset: usize, length: usize) {
        if length == 0 {
            return;
        }
        if let Some(root_node) = self.root.take() {
            if offset >= root_node.subtree_len {
                self.root = Some(root_node);
                return;
            }
            let length = length.min(root_node.subtree_len - offset);
            self.undo_stack.push(Some(root_node.clone()));
            if offset == 0 && length >= root_node.subtree_len {
                self.redo_stack.clear();
                self.root = None;
                return;
            }
            let mut start_node_pos = self.node_at(root_node.clone(), offset);
            let end_node_pos = self.node_at(root_node.clone(), offset + length);
            let start_piece = Self::last(&start_node_pos.path);
            let end_piece = Self::last(&end_node_pos.path);
            if start_piece == end_piece {
                if start_node_pos.remainder == 0 && end_node_pos.remainder == start_piece.length {
                    let new_root = self.delete_at(root_node, offset);
                    self.root = Some(self.blacken(new_root));
                } else if start_node_pos.remainder == 0
                    && end_node_pos.remainder < start_piece.length
                {
                    let suffix = self.shrink_piece_to_suffix(&start_piece, end_node_pos.remainder);
                    let new_root = Self::replace_at(root_node, start_node_pos.start_offset, suffix);
                    self.root = Some(new_root);
                } else if start_node_pos.remainder > 0
                    && end_node_pos.remainder == start_piece.length
                {
                    let prefix =
                        self.shrink_piece_to_prefix(&start_piece, start_node_pos.remainder);
                    let new_root = Self::replace_at(root_node, start_node_pos.start_offset, prefix);
                    self.root = Some(new_root);
                } else {
                    let prefix =
                        self.shrink_piece_to_prefix(&start_piece, start_node_pos.remainder);
                    let new_root = Self::replace_at(root_node, start_node_pos.start_offset, prefix);
                    let suffix_info =
                        self.shrink_piece_to_suffix(&start_piece, end_node_pos.remainder);
                    let suffix = Rc::new(Node::new(
                        suffix_info.start,
                        suffix_info.end,
                        suffix_info.line_feed_count,
                        suffix_info.length,
                        start_piece.buffer_type,
                        Color::Red,
                        Some(self.black_leaf.clone()),
                        Some(self.black_leaf.clone()),
                    ));
                    let new_root = self.insert_at(
                        new_root,
                        start_node_pos.start_offset + (start_node_pos.remainder),
                        suffix,
                    );
                    self.root = Some(self.blacken(new_root));
                }
            } else {
                let left = self.shrink_piece_to_prefix(&start_piece, start_node_pos.remainder);

                let right = self.shrink_piece_to_suffix(&end_piece, end_node_pos.remainder);
                let mut between: Vec<usize> = Vec::new();
                let path = &mut start_node_pos.path;
                let mut at = start_node_pos.start_offset + start_piece.length;
                while self.next(path) && Self::last(path) != end_piece {
                    between.push(at);
                    at += Self::last(path).length;
                }
                let mut new_root = if right.length == 0 {
                    self.blacken(self.delete_at(root_node, end_node_pos.start_offset))
                } else {
                    Self::replace_at(root_node, end_node_pos.start_offset, right)
                };

                let mut i = between.len() as i32 - 1;
                while i >= 0 {
                    new_root = self.blacken(self.delete_at(new_root, between[i as usize]));
                    i -= 1;
                }
                new_root = if left.length == 0 {
                    self.blacken(self.delete_at(new_root, start_node_pos.start_offset))
                } else {
                    Self::replace_at(new_root, start_node_pos.start_offset, left)
                };
                self.root = Some(self.blacken(new_root));
            }
            self.redo_stack.clear();
        }
    }
    fn last(path: &[Rc<Node>]) -> Rc<Node> {
        path[path.len() - 1].clone()
    }
    fn next(&self, path: &mut Vec<Rc<Node>>) -> bool {
        let mut node = Self::last(path);
        let right = node.right.clone().unwrap();
        if right != self.black_leaf {
            let mut n = right;
            while n != self.black_leaf {
                path.push(n.clone());
                n = n.left.clone().unwrap();
            }
            return true;
        }
        while path.len() > 1 {
            path.pop();
            let parent = Self::last(path);
            if parent.left.clone().unwrap() == node {
                return true;
            }
            node = parent;
        }

        false
    }
    fn replace_at(current_node: Rc<Node>, offset: usize, node_that_replaces: NodeInfo) -> Rc<Node> {
        if offset < current_node.left_subtree_len {
            let left_path = Self::replace_at(
                current_node.left.as_ref().unwrap().clone(),
                offset,
                node_that_replaces,
            );

            Rc::new(Node::new(
                current_node.start,
                current_node.end,
                current_node.line_feed_count,
                current_node.length,
                current_node.buffer_type,
                current_node.color,
                Some(left_path),
                current_node.right.clone(),
            ))
        } else if offset >= current_node.left_subtree_len + current_node.length {
            let right_offset = offset - (current_node.left_subtree_len + current_node.length);
            let right_path = Self::replace_at(
                current_node.right.as_ref().unwrap().clone(),
                right_offset,
                node_that_replaces,
            );

            Rc::new(Node::new(
                current_node.start,
                current_node.end,
                current_node.line_feed_count,
                current_node.length,
                current_node.buffer_type,
                current_node.color,
                current_node.left.clone(),
                Some(right_path),
            ))
        } else {
            Rc::new(Node::new(
                node_that_replaces.start,
                node_that_replaces.end,
                node_that_replaces.line_feed_count,
                node_that_replaces.length,
                node_that_replaces.buffer_type,
                current_node.color,
                current_node.left.clone(),
                current_node.right.clone(),
            ))
        }
    }
    fn pre_insert(&mut self, content: &str) -> Rc<Node> {
        let start = BufferPosition {
            line: self.add_line_starts.len() - 1,
            column: self.add.len() - self.add_line_starts[self.add_line_starts.len() - 1],
        };
        find_line_starts(content, self.add.len(), &mut self.add_line_starts);
        self.add.push_str(content);
        let end = BufferPosition {
            line: self.add_line_starts.len() - 1,
            column: self.add.len() - self.add_line_starts[self.add_line_starts.len() - 1],
        };
        let line_feed_count =
            calculate_line_feed_count(&self.add, &self.add_line_starts, start, end);
        Rc::new(Node::new(
            start,
            end,
            line_feed_count,
            content.len(),
            BufferType::Add,
            Color::Red,
            Some(self.black_leaf.clone()),
            Some(self.black_leaf.clone()),
        ))
    }
    pub fn insert(&mut self, content: &str, offset: usize) {
        if content.is_empty() {
            return;
        }
        if let Some(root_node) = self.root.take() {
            self.undo_stack.push(Some(root_node.clone()));
            let node_to_insert = self.pre_insert(content);
            let node_position = self.node_at(root_node.clone(), offset);
            let piece = Self::last(&node_position.path);
            if piece.end == self.last_change_in_buffer
                && node_position.start_offset + piece.length == offset
                && piece.buffer_type == BufferType::Add
            {
                let replacement = self.expand_piece(&piece, content.len());
                self.last_change_in_buffer = replacement.end;
                let new_root = Self::replace_at(root_node, node_position.start_offset, replacement);
                self.root = Some(self.blacken(new_root));
                self.redo_stack.clear();
                return;
            }
            if node_position.start_offset + piece.length > offset
                && offset != node_position.start_offset
            {
                let first_part = self.shrink_piece_to_prefix(&piece, node_position.remainder);
                let new_root = Self::replace_at(root_node, node_position.start_offset, first_part);
                let second_part_info = self.shrink_piece_to_suffix(&piece, node_position.remainder);
                let second_part = Rc::new(Node::new(
                    second_part_info.start,
                    second_part_info.end,
                    second_part_info.line_feed_count,
                    second_part_info.length,
                    piece.buffer_type,
                    Color::Red,
                    Some(self.black_leaf.clone()),
                    Some(self.black_leaf.clone()),
                ));
                self.last_change_in_buffer = node_to_insert.end;
                let new_root = self.insert_at(new_root, offset, node_to_insert);
                let new_root = self.insert_at(new_root, offset + content.len(), second_part);
                self.root = Some(self.blacken(new_root));
            } else {
                self.last_change_in_buffer = node_to_insert.end;
                let new_root = self.insert_at(root_node.clone(), offset, node_to_insert);
                self.root = Some(self.blacken(new_root));
            }
        } else {
            self.undo_stack.push(None);
            let node_to_insert = self.pre_insert(content);
            self.last_change_in_buffer = node_to_insert.end;
            self.root = Some(self.blacken(node_to_insert));
        }
        self.redo_stack.clear();
    }
    fn blacken(&self, node: Rc<Node>) -> Rc<Node> {
        if node == self.black_leaf {
            panic!("black leaf blacken");
        }
        Rc::new(Node::new(
            node.start,
            node.end,
            node.line_feed_count,
            node.length,
            node.buffer_type,
            Color::Black,
            node.left.clone(),
            node.right.clone(),
        ))
    }

    fn bubble(&self, y: Rc<Node>) -> Rc<Node> {
        if y.left_color() == Color::DoubleBlack || y.right_color() == Color::DoubleBlack {
            let x = self.redder(y.left.as_ref().unwrap());
            let z = self.redder(y.right.as_ref().unwrap());
            let new_y = Rc::new(Node::new(
                y.start,
                y.end,
                y.line_feed_count,
                y.length,
                y.buffer_type,
                y.color.plus_black(),
                Some(x),
                Some(z),
            ));
            let new_y = Self::rebalance(new_y);
            return new_y;
        }
        Self::rebalance(y)
    }
    fn rebalance(z: Rc<Node>) -> Rc<Node> {
        if z.color == Color::Black || z.color == Color::DoubleBlack {
            let z_original_color = z.color;
            if z.left_color() == Color::Red && z.left_right_color() == Color::Red {
                let x = z.left.as_ref().unwrap().clone();
                let y = x.right.as_ref().unwrap().clone();
                let new_x = Rc::new(Node::new(
                    x.start,
                    x.end,
                    x.line_feed_count,
                    x.length,
                    x.buffer_type,
                    Color::Black,
                    x.left.clone(),
                    y.left.clone(),
                ));
                let new_z = Rc::new(Node::new(
                    z.start,
                    z.end,
                    z.line_feed_count,
                    z.length,
                    z.buffer_type,
                    Color::Black,
                    y.right.clone(),
                    z.right.clone(),
                ));
                let new_y_color = if z_original_color == Color::Black {
                    Color::Red
                } else {
                    Color::Black
                };
                let new_y = Rc::new(Node::new(
                    y.start,
                    y.end,
                    y.line_feed_count,
                    y.length,
                    y.buffer_type,
                    new_y_color,
                    Some(new_x),
                    Some(new_z),
                ));
                return new_y;
            } else if z.left_color() == Color::Red && z.left_left_color() == Color::Red {
                let y = z.left.as_ref().unwrap().clone();
                let x = y.left.as_ref().unwrap().clone();
                let new_x = Rc::new(Node::new(
                    x.start,
                    x.end,
                    x.line_feed_count,
                    x.length,
                    x.buffer_type,
                    Color::Black,
                    x.left.clone(),
                    x.right.clone(),
                ));
                let new_z = Rc::new(Node::new(
                    z.start,
                    z.end,
                    z.line_feed_count,
                    z.length,
                    z.buffer_type,
                    Color::Black,
                    y.right.clone(),
                    z.right.clone(),
                ));
                let new_y_color = if z_original_color == Color::Black {
                    Color::Red
                } else {
                    Color::Black
                };
                let new_y = Rc::new(Node::new(
                    y.start,
                    y.end,
                    y.line_feed_count,
                    y.length,
                    y.buffer_type,
                    new_y_color,
                    Some(new_x),
                    Some(new_z),
                ));
                return new_y;
            } else if z.right_color() == Color::Red && z.right_left_color() == Color::Red {
                let x = z;
                let z = x.right.as_ref().unwrap().clone();
                let y = z.left.as_ref().unwrap().clone();
                let new_x = Rc::new(Node::new(
                    x.start,
                    x.end,
                    x.line_feed_count,
                    x.length,
                    x.buffer_type,
                    Color::Black,
                    x.left.clone(),
                    y.left.clone(),
                ));
                let new_z = Rc::new(Node::new(
                    z.start,
                    z.end,
                    z.line_feed_count,
                    z.length,
                    z.buffer_type,
                    Color::Black,
                    y.right.clone(),
                    z.right.clone(),
                ));
                let new_y_color = if z_original_color == Color::Black {
                    Color::Red
                } else {
                    Color::Black
                };
                let new_y = Rc::new(Node::new(
                    y.start,
                    y.end,
                    y.line_feed_count,
                    y.length,
                    y.buffer_type,
                    new_y_color,
                    Some(new_x),
                    Some(new_z),
                ));
                return new_y;
            } else if z.right_color() == Color::Red && z.right_right_color() == Color::Red {
                let x = z;
                let y = x.right.as_ref().unwrap().clone();
                let z = y.right.as_ref().unwrap().clone();
                let new_x = Rc::new(Node::new(
                    x.start,
                    x.end,
                    x.line_feed_count,
                    x.length,
                    x.buffer_type,
                    Color::Black,
                    x.left.clone(),
                    y.left.clone(),
                ));
                let new_z = Rc::new(Node::new(
                    z.start,
                    z.end,
                    z.line_feed_count,
                    z.length,
                    z.buffer_type,
                    Color::Black,
                    z.left.clone(),
                    z.right.clone(),
                ));
                let new_y_color = if z_original_color == Color::Black {
                    Color::Red
                } else {
                    Color::Black
                };
                let new_y = Rc::new(Node::new(
                    y.start,
                    y.end,
                    y.line_feed_count,
                    y.length,
                    y.buffer_type,
                    new_y_color,
                    Some(new_x),
                    Some(new_z),
                ));
                return new_y;
            }
        }
        if z.color == Color::DoubleBlack
            && z.left_color() == Color::NegativeBlack
            && z.left
                .as_ref()
                .unwrap()
                .has_legimate_black_internal_children()
        {
            let x = z.left.as_ref().unwrap().clone();
            let w = x.left.as_ref().unwrap().clone();
            let y = x.right.as_ref().unwrap().clone();
            let new_w = Rc::new(Node::new(
                w.start,
                w.end,
                w.line_feed_count,
                w.length,
                w.buffer_type,
                Color::Red,
                w.left.clone(),
                w.right.clone(),
            ));
            let new_x = Rc::new(Node::new(
                x.start,
                x.end,
                x.line_feed_count,
                x.length,
                x.buffer_type,
                Color::Black,
                Some(new_w),
                y.left.clone(),
            ));
            let new_x = Self::rebalance(new_x);
            let new_z = Rc::new(Node::new(
                z.start,
                z.end,
                z.line_feed_count,
                z.length,
                z.buffer_type,
                Color::Black,
                y.right.clone(),
                z.right.clone(),
            ));
            let new_y = Rc::new(Node::new(
                y.start,
                y.end,
                y.line_feed_count,
                y.length,
                y.buffer_type,
                Color::Black,
                Some(new_x),
                Some(new_z),
            ));
            return new_y;
        } else if z.color == Color::DoubleBlack
            && z.right_color() == Color::NegativeBlack
            && z.right
                .as_ref()
                .unwrap()
                .has_legimate_black_internal_children()
        {
            let x = z;
            let z = x.right.as_ref().unwrap().clone();
            let y = z.left.as_ref().unwrap().clone();
            let w = z.right.as_ref().unwrap().clone();
            let new_w = Rc::new(Node::new(
                w.start,
                w.end,
                w.line_feed_count,
                w.length,
                w.buffer_type,
                Color::Red,
                w.left.clone(),
                w.right.clone(),
            ));
            let new_x = Rc::new(Node::new(
                x.start,
                x.end,
                x.line_feed_count,
                x.length,
                x.buffer_type,
                Color::Black,
                x.left.clone(),
                y.left.clone(),
            ));
            let new_z = Rc::new(Node::new(
                z.start,
                z.end,
                z.line_feed_count,
                z.length,
                z.buffer_type,
                Color::Black,
                y.right.clone(),
                Some(new_w),
            ));
            let new_z = Self::rebalance(new_z);
            let new_y = Rc::new(Node::new(
                y.start,
                y.end,
                y.line_feed_count,
                y.length,
                y.buffer_type,
                Color::Black,
                Some(new_x),
                Some(new_z),
            ));
            return new_y;
        }
        z
    }
    fn redder(&self, node: &Rc<Node>) -> Rc<Node> {
        if node == &self.black_leaf {
            return self.black_leaf.clone();
        }
        if node == &self.double_black_leaf {
            return self.black_leaf.clone();
        }

        Rc::new(Node::new(
            node.start,
            node.end,
            node.line_feed_count,
            node.length,
            node.buffer_type,
            node.color.minus_black(),
            node.left.clone(),
            node.right.clone(),
        ))
    }
}
#[derive(PartialEq, Debug)]
struct Node {
    start: BufferPosition,
    end: BufferPosition,
    length: usize,
    buffer_type: BufferType,
    line_feed_count: usize,
    subtree_line_feed_count: usize,
    left_subtree_line_feed_count: usize,
    color: Color,
    left_subtree_len: usize,
    subtree_len: usize,
    left: Option<Rc<Node>>,
    right: Option<Rc<Node>>,
}
#[derive(Clone, Copy, PartialEq, Debug, Default)]
struct BufferPosition {
    line: usize,
    column: usize,
}

impl Node {
    fn new(
        start: BufferPosition,
        end: BufferPosition,
        line_feed_count: usize,
        length: usize,
        buffer_type: BufferType,
        color: Color,
        left: Option<Rc<Node>>,
        right: Option<Rc<Node>>,
    ) -> Self {
        let left_subtree_len = match left.as_ref() {
            Some(l) => l.subtree_len,
            None => 0,
        };
        let right_subtree_len = match right.as_ref() {
            Some(r) => r.subtree_len,
            None => 0,
        };
        let left_subtree_line_feed_count = match left.as_ref() {
            Some(l) => l.subtree_line_feed_count,
            None => 0,
        };
        let right_subtree_line_feed_count = match right.as_ref() {
            Some(r) => r.subtree_line_feed_count,
            None => 0,
        };
        let subtree_len = left_subtree_len + length + right_subtree_len;
        let subtree_line_feed_count =
            left_subtree_line_feed_count + line_feed_count + right_subtree_line_feed_count;
        Self {
            start,
            end,
            length,
            line_feed_count,
            subtree_line_feed_count,
            left_subtree_line_feed_count,
            buffer_type,
            color,
            left,
            right,
            left_subtree_len,
            subtree_len,
        }
    }
    fn new_black_leaf() -> Self {
        Self {
            start: BufferPosition::default(),
            length: 0,
            buffer_type: BufferType::Original,
            color: Color::Black,
            left_subtree_len: 0,
            subtree_len: 0,
            left: None,
            right: None,
            end: BufferPosition::default(),
            line_feed_count: 0,
            subtree_line_feed_count: 0,
            left_subtree_line_feed_count: 0,
        }
    }
    fn new_double_black_leaf() -> Self {
        Self {
            start: BufferPosition::default(),
            length: 0,
            buffer_type: BufferType::Original,
            color: Color::DoubleBlack,
            left_subtree_len: 0,
            subtree_len: 0,
            left: None,
            right: None,
            end: BufferPosition::default(),
            line_feed_count: 0,
            subtree_line_feed_count: 0,
            left_subtree_line_feed_count: 0,
        }
    }

    fn left_color(&self) -> Color {
        match self.left.as_ref() {
            Some(l) => l.color,
            None => Color::Black,
        }
    }
    fn left_right_color(&self) -> Color {
        match self.left.as_ref() {
            Some(l) => match l.right.as_ref() {
                Some(lr) => lr.color,
                None => Color::Black,
            },
            None => Color::Black,
        }
    }
    fn left_left_color(&self) -> Color {
        match self.left.as_ref() {
            Some(l) => match l.left.as_ref() {
                Some(ll) => ll.color,
                None => Color::Black,
            },
            None => Color::Black,
        }
    }
    fn right_color(&self) -> Color {
        match self.right.as_ref() {
            Some(r) => r.color,
            None => Color::Black,
        }
    }

    fn right_left_color(&self) -> Color {
        match self.right.as_ref() {
            Some(r) => match r.left.as_ref() {
                Some(rl) => rl.color,
                None => Color::Black,
            },
            None => Color::Black,
        }
    }
    fn right_right_color(&self) -> Color {
        match self.right.as_ref() {
            Some(r) => match r.right.as_ref() {
                Some(rr) => rr.color,
                None => Color::Black,
            },
            None => Color::Black,
        }
    }
    fn has_legimate_black_internal_children(&self) -> bool {
        let left = match self.left.as_ref() {
            Some(l) => l.length > 0 && l.color == Color::Black,
            None => false,
        };
        let right = match self.right.as_ref() {
            Some(r) => r.length > 0 && r.color == Color::Black,
            None => false,
        };
        left && right
    }
}

struct NodeInfo {
    start: BufferPosition,
    end: BufferPosition,
    line_feed_count: usize,
    length: usize,
    buffer_type: BufferType,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum BufferType {
    Original,
    Add,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Color {
    NegativeBlack,
    Red,
    Black,
    DoubleBlack,
}

impl Color {
    fn minus_black(&self) -> Self {
        match self {
            Color::NegativeBlack => {
                panic!("minusing a negative black should not possible")
            }
            Color::Red => Color::NegativeBlack,
            Color::Black => Color::Red,
            Color::DoubleBlack => Color::Black,
        }
    }
    fn plus_black(&self) -> Self {
        match self {
            Color::NegativeBlack => Color::Red,
            Color::Red => Color::Black,
            Color::Black => Color::DoubleBlack,
            Color::DoubleBlack => {
                panic!("plusing a double black should not possible")
            }
        }
    }
}
struct NodePosition {
    start_offset: usize,
    remainder: usize,
    path: Vec<Rc<Node>>,
}
struct LineDescent {
    should_continue: bool,
    path: Vec<Rc<Node>>,
}
fn find_line_starts(content: &str, offset: usize, line_starts: &mut Vec<usize>) {
    let mut i = 0;
    while let Some(pos) = content[i..].find('\n') {
        i += pos + 1;
        line_starts.push(i + offset);
    }
}

fn calculate_line_feed_count(
    buffer: &str,
    line_starts: &[usize],
    start: BufferPosition,
    end: BufferPosition,
) -> usize {
    if end.column == 0 {
        return end.line - start.line;
    }
    if end.line == line_starts.len() - 1 {
        return end.line - start.line;
    }
    let next_line_start_offset = line_starts[end.line + 1];
    let end_offset = line_starts[end.line] + end.column;
    if next_line_start_offset > end_offset + 1 {
        return end.line - start.line;
    }
    let previous_char_offset = end_offset - 1;
    if buffer.as_bytes()[previous_char_offset] == b'\n' {
        end.line - start.line + 1
    } else {
        end.line - start.line
    }
}
fn find_nearest_start_and_column_in_it(
    line_starts: &[usize],
    index: usize,
    start_offset: usize,
) -> BufferPosition {
    let mut index = index + 1;
    while index < line_starts.len() {
        if line_starts[index] > start_offset {
            let line = index - 1;
            let column = start_offset - line_starts[line];
            return BufferPosition { line, column };
        } else {
            index += 1;
        }
    }
    let line = index - 1;
    let column = start_offset - line_starts[line];
    BufferPosition { line, column }
}

// ============================================================================
// Test-only structural accessors.
//
// These exist so tests can assert the red-black invariants by inspecting the
// tree, instead of only comparing the document text (which is unaffected by
// colors and black heights, and therefore cannot see a broken tree). They are
// compiled only under `cfg(test)` and add no behavior to the library.
// ============================================================================
#[cfg(test)]
#[derive(Debug, Clone, Default)]
pub struct InvariantReport {
    /// number of internal nodes visited
    pub nodes: usize,
    /// deepest internal node count along any path
    pub max_depth: usize,
    /// black height of the root (0 when the tree is empty)
    pub root_black_height: i32,
    /// every invariant that was found broken (capped; see `violation_count`)
    pub violations: Vec<String>,
    /// total number of violations, including any not listed in `violations`
    pub violation_count: usize,
}

#[cfg(test)]
impl InvariantReport {
    /// True when the tree satisfies every red-black invariant.
    pub fn is_ok(&self) -> bool {
        self.violation_count == 0
    }

    pub fn summary(&self) -> String {
        let mut out = format!(
            "{} violation(s), {} node(s), max depth {}, root black height {}\n",
            self.violation_count, self.nodes, self.max_depth, self.root_black_height
        );
        for v in &self.violations {
            out.push_str("  - ");
            out.push_str(v);
            out.push('\n');
        }
        if self.violation_count > self.violations.len() {
            out.push_str(&format!(
                "  ... and {} more\n",
                self.violation_count - self.violations.len()
            ));
        }
        out
    }
}
