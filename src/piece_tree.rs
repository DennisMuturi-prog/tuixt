use core::panic;
use std::rc::Rc;

pub struct PieceTree {
    original: String,
    add: String,
    root: Option<Rc<Node>>,
    undo_stack: Vec<Rc<Node>>,
    redo_stack: Vec<Rc<Node>>,
    black_leaf: Rc<Node>,
    double_black_leaf: Rc<Node>,
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
                double_black_leaf: Rc::new(Node::new_double_black_leaf()),
                black_leaf: Rc::new(Node::new_black_leaf()),
            }
        } else {
            let original = String::from(original_content);
            let black_leaf = Rc::new(Node::new_black_leaf());
            let root = Some(Rc::new(Node::new(
                0,
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
            }
        }
    }
    pub fn get_text(&self, content: &mut String) {
        if let Some(root_node) = self.root.as_ref() {
            self.in_order_traversal(root_node, content);
        }
    }

    fn in_order_traversal(&self, node: &Rc<Node>, content: &mut String) {
        if node == &self.black_leaf {
            return;
        }
        self.in_order_traversal(node.left.as_ref().unwrap(), content);
        let content_str = match node.buffer_type {
            BufferType::Original => &self.original[node.start..node.start + node.length],
            BufferType::Add => &self.add[node.start..node.start + node.length],
        };
        content.push_str(content_str);
        self.in_order_traversal(node.right.as_ref().unwrap(), content);
    }

    fn insert_as_predecessor(&self, curr_node: Rc<Node>, node_to_insert: Rc<Node>) -> Rc<Node> {
        if curr_node.right.as_ref().unwrap().clone() == self.black_leaf {
            let new_node = Rc::new(Node::new(
                curr_node.start,
                curr_node.length,
                curr_node.buffer_type,
                curr_node.color,
                curr_node.left.clone(),
                Some(node_to_insert),
            ));
            return Self::rebalance(new_node);
        }
        let right_path =
            self.insert_as_predecessor(curr_node.right.as_ref().unwrap().clone(), node_to_insert);
        let new_node = Rc::new(Node::new(
            curr_node.start,
            curr_node.length,
            curr_node.buffer_type,
            curr_node.color,
            curr_node.left.clone(),
            Some(right_path),
        ));
        Self::rebalance(new_node)
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
            length: dest.length,
            color: dest.color,
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
    fn insert_at_beginning(&self, current_node: Rc<Node>, node_to_insert: Rc<Node>) -> Rc<Node> {
        if current_node == self.black_leaf {
            return node_to_insert;
        }
        let left_path =
            self.insert_at_beginning(current_node.left.as_ref().unwrap().clone(), node_to_insert);
        let new_current_node = Rc::new(Node::new(
            current_node.start,
            current_node.length,
            current_node.buffer_type,
            current_node.color,
            Some(left_path),
            current_node.right.clone(),
        ));
        Self::rebalance(new_current_node)
    }
    fn insert_at_end(&self, current_node: Rc<Node>, node_to_insert: Rc<Node>) -> Rc<Node> {
        if current_node == self.black_leaf {
            return node_to_insert;
        }
        let right_path =
            self.insert_at_end(current_node.right.as_ref().unwrap().clone(), node_to_insert);
        let new_current_node = Rc::new(Node::new(
            current_node.start,
            current_node.length,
            current_node.buffer_type,
            current_node.color,
            current_node.left.clone(),
            Some(right_path),
        ));
        Self::rebalance(new_current_node)
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
            self.undo_stack.push(root_node.clone());
            if offset == 0 && length >= root_node.subtree_len {
                self.root = None;
                return;
            }
            let mut start_node_pos = self.node_at(root_node.clone(), offset);
            let end_node_pos = self.node_at(root_node.clone(), offset + length);
            let start_piece = Self::last(&start_node_pos.path);
            let end_piece = Self::last(&end_node_pos.path);
            if start_piece == end_piece {
                if start_node_pos.remainder == 0 && end_node_pos.remainder == start_piece.length 
                {
                    let new_root = self.delete_at(root_node, offset);
                    self.root = Some(self.blacken(new_root));
                } else if start_node_pos.remainder == 0
                    && end_node_pos.remainder < start_piece.length 
                {
                    let suffix = NodeInfo {
                        start: start_piece.start + end_node_pos.remainder,
                        length: start_piece.length - end_node_pos.remainder,
                        buffer_type: start_piece.buffer_type,
                        color: start_piece.color,
                    };
                    let new_root = Self::replace_at(root_node, start_node_pos.start_offset, suffix);
                    self.root = Some(new_root);
                } else if start_node_pos.remainder > 0
                    && end_node_pos.remainder == start_piece.length
                {
                    let prefix = NodeInfo {
                        start: start_piece.start,
                        length: start_node_pos.remainder,
                        buffer_type: start_piece.buffer_type,
                        color: start_piece.color,
                    };
                    let new_root = Self::replace_at(root_node, start_node_pos.start_offset, prefix);
                    self.root = Some(new_root);
                } else {
                    let prefix = NodeInfo {
                        start: start_piece.start,
                        length: start_node_pos.remainder,
                        buffer_type: start_piece.buffer_type,
                        color: start_piece.color,
                    };
                    let new_root = Self::replace_at(root_node, start_node_pos.start_offset, prefix);
                    let suffix = Rc::new(Node::new(
                        start_piece.start + end_node_pos.remainder,
                        start_piece.length - end_node_pos.remainder,
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
                let left = NodeInfo {
                    start: start_piece.start,
                    length: start_node_pos.remainder,
                    buffer_type: start_piece.buffer_type,
                    color: start_piece.color,
                };
                let right = NodeInfo {
                    start: end_piece.start + end_node_pos.remainder,
                    length: end_piece.length - end_node_pos.remainder,
                    buffer_type: end_piece.buffer_type,
                    color: end_piece.color,
                };
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
                current_node.length,
                current_node.buffer_type,
                current_node.color,
                current_node.left.clone(),
                Some(right_path),
            ))
        } else {
            Rc::new(Node::new(
                node_that_replaces.start,
                node_that_replaces.length,
                node_that_replaces.buffer_type,
                current_node.color,
                current_node.left.clone(),
                current_node.right.clone(),
            ))
        }
    }
    fn pre_insert(&mut self, content: &str) -> Rc<Node> {
        let previous_len = self.add.len();
        self.add.push_str(content);
        Rc::new(Node::new(
            previous_len,
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
            self.undo_stack.push(root_node.clone());
            let node_to_insert = self.pre_insert(content);

            if offset == 0 {
                let new_root = self.insert_at_beginning(root_node.clone(), node_to_insert);
                self.root = Some(self.blacken(new_root));
                return;
            }
            if offset == root_node.subtree_len {
                let new_root = self.insert_at_end(root_node.clone(), node_to_insert);
                self.root = Some(self.blacken(new_root));
                return;
            }
            let node_position = self.node_at(root_node.clone(), offset);
            let piece = Self::last(&node_position.path);
            if node_position.start_offset + piece.length > offset
                && offset != node_position.start_offset
            {
                let first_part = NodeInfo {
                    start: piece.start,
                    length: node_position.remainder,
                    buffer_type: piece.buffer_type,
                    color: piece.color,
                };
                let new_root = Self::replace_at(root_node, node_position.start_offset, first_part);
                let second_part = Rc::new(Node::new(
                    piece.start + node_position.remainder,
                    piece.length - node_position.remainder,
                    piece.buffer_type,
                    Color::Red,
                    Some(self.black_leaf.clone()),
                    Some(self.black_leaf.clone()),
                ));
                let new_root = self.insert_at(new_root, offset, node_to_insert);
                let new_root = self.insert_at(new_root, offset + content.len(), second_part);
                self.root = Some(self.blacken(new_root));
            } else {
                let new_root = self.insert_at(root_node.clone(), offset, node_to_insert);
                self.root = Some(self.blacken(new_root));
            }
        } else {
            let node_to_insert = self.pre_insert(content);
            self.root = Some(self.blacken(node_to_insert));
        }
    }
    fn blacken(&self, node: Rc<Node>) -> Rc<Node> {
        if node == self.black_leaf {
            panic!("black leaf blacken");
        }
        Rc::new(Node::new(
            node.start,
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
                    x.length,
                    x.buffer_type,
                    Color::Black,
                    x.left.clone(),
                    y.left.clone(),
                ));
                let new_z = Rc::new(Node::new(
                    z.start,
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
                    x.length,
                    x.buffer_type,
                    Color::Black,
                    x.left.clone(),
                    x.right.clone(),
                ));
                let new_z = Rc::new(Node::new(
                    z.start,
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
                    x.length,
                    x.buffer_type,
                    Color::Black,
                    x.left.clone(),
                    y.left.clone(),
                ));
                let new_z = Rc::new(Node::new(
                    z.start,
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
                    x.length,
                    x.buffer_type,
                    Color::Black,
                    x.left.clone(),
                    y.left.clone(),
                ));
                let new_z = Rc::new(Node::new(
                    z.start,
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
                w.length,
                w.buffer_type,
                Color::Red,
                w.left.clone(),
                w.right.clone(),
            ));
            let new_x = Rc::new(Node::new(
                x.start,
                x.length,
                x.buffer_type,
                Color::Black,
                Some(new_w),
                y.left.clone(),
            ));
            let new_x = Self::rebalance(new_x);
            let new_z = Rc::new(Node::new(
                z.start,
                z.length,
                z.buffer_type,
                Color::Black,
                y.right.clone(),
                z.right.clone(),
            ));
            let new_y = Rc::new(Node::new(
                y.start,
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
                w.length,
                w.buffer_type,
                Color::Red,
                w.left.clone(),
                w.right.clone(),
            ));
            let new_x = Rc::new(Node::new(
                x.start,
                x.length,
                x.buffer_type,
                Color::Black,
                x.left.clone(),
                y.left.clone(),
            ));
            let new_z = Rc::new(Node::new(
                z.start,
                z.length,
                z.buffer_type,
                Color::Black,
                y.right.clone(),
                Some(new_w),
            ));
            let new_z = Self::rebalance(new_z);
            let new_y = Rc::new(Node::new(
                y.start,
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
            node.length,
            node.buffer_type,
            node.color.minus_black(),
            node.left.clone(),
            node.right.clone(),
        ))
    }
    fn blacker(&self, node: &Rc<Node>) -> Rc<Node> {
        if node == &self.black_leaf {
            return self.double_black_leaf.clone();
        }
        if node == &self.double_black_leaf {
            panic!("cannot blacken a double black leaf");
        }

        Rc::new(Node::new(
            node.start,
            node.length,
            node.buffer_type,
            node.color.plus_black(),
            node.right.clone(),
            node.right.clone(),
        ))
    }
}
#[derive(PartialEq, Debug)]
struct Node {
    start: usize,
    length: usize,
    buffer_type: BufferType,
    color: Color,
    left_subtree_len: usize,
    subtree_len: usize,
    black_height: i32,
    left: Option<Rc<Node>>,
    right: Option<Rc<Node>>,
}

impl Node {
    fn new(
        start: usize,
        length: usize,
        buffer_type: BufferType,
        color: Color,
        left: Option<Rc<Node>>,
        right: Option<Rc<Node>>,
    ) -> Self {
        // Only the two leaf constructors may omit children; every other node must
        // point at real subtrees. Copying a leaf's `None` children into a regular
        // node is what produced the "phantom" nodes that broke traversal.
        debug_assert!(
            left.is_some() && right.is_some(),
            "Node::new called without children (only the black/double-black leaf constructors may do that)"
        );
        let left_subtree_len = match left.as_ref() {
            Some(l) => l.subtree_len,
            None => 0,
        };
        let right_subtree_len = match right.as_ref() {
            Some(r) => r.subtree_len,
            None => 0,
        };
        let child_blk_h = match left.as_ref() {
            Some(l) => l.black_height,
            None => 0,
        };
        let black_height = match color {
            Color::NegativeBlack => child_blk_h - 1,
            Color::Red => child_blk_h,
            Color::Black => child_blk_h + 1,
            Color::DoubleBlack => child_blk_h + 2,
        };
        let subtree_len = left_subtree_len + length + right_subtree_len;
        Self {
            start,
            length,
            buffer_type,
            color,
            left,
            right,
            left_subtree_len,
            subtree_len,
            black_height,
        }
    }
    fn new_black_leaf() -> Self {
        Self {
            start: 0,
            length: 0,
            buffer_type: BufferType::Original,
            color: Color::Black,
            left_subtree_len: 0,
            subtree_len: 0,
            left: None,
            right: None,
            black_height: 0,
        }
    }
    fn new_double_black_leaf() -> Self {
        Self {
            start: 0,
            length: 0,
            buffer_type: BufferType::Original,
            color: Color::DoubleBlack,
            left_subtree_len: 0,
            subtree_len: 0,
            left: None,
            right: None,
            black_height: 0,
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
    start: usize,
    length: usize,
    color: Color,
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
const MAX_REPORTED_VIOLATIONS: usize = 50;

#[cfg(test)]
impl InvariantReport {
    /// True when the tree satisfies every red-black invariant.
    pub fn is_ok(&self) -> bool {
        self.violation_count == 0
    }

    fn violate(&mut self, message: String) {
        self.violation_count += 1;
        if self.violations.len() < MAX_REPORTED_VIOLATIONS {
            self.violations.push(message);
        }
    }

    /// Human readable report, for assertion messages.
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

#[cfg(test)]
impl PieceTree {
    /// Test-only: walk the tree and report every red-black invariant it breaks.
    ///
    /// Checked: the root is black; no double-black or negative-black marker
    /// survives an operation; a red node has no red child; both children of a
    /// node have equal black height; each stored `black_height` matches the
    /// color and children it was derived from; `subtree_len` and
    /// `left_subtree_len` match the actual subtrees; and no non-sentinel node
    /// is childless (the "phantom node" failure mode).
    pub fn invariant_report(&self) -> InvariantReport {
        let mut report = InvariantReport::default();
        if let Some(root) = self.root.as_ref() {
            report.root_black_height = self.check_node(Some(root), true, 0, &mut report);
        }
        report
    }

    /// Returns the subtree's black height (external leaf = 0).
    fn check_node(
        &self,
        node: Option<&Rc<Node>>,
        is_root: bool,
        depth: usize,
        report: &mut InvariantReport,
    ) -> i32 {
        let node = match node {
            None => {
                report.violate(
                    "child pointer is None; an absent child must be the black leaf".to_string(),
                );
                return 0;
            }
            Some(node) => node,
        };

        if std::rc::Rc::ptr_eq(node, &self.black_leaf) {
            return 0; // external leaf: black by definition, black height 0
        }
        if std::rc::Rc::ptr_eq(node, &self.double_black_leaf) {
            report.violate("the double-black leaf is reachable from the root".to_string());
            return 0;
        }

        report.nodes += 1;
        if depth + 1 > report.max_depth {
            report.max_depth = depth + 1;
        }
        let at = format!("(start={}, len={})", node.start, node.length);

        if node.color == Color::DoubleBlack {
            report.violate(format!("double-black node left in the tree at rest {}", at));
        }
        if node.color == Color::NegativeBlack {
            report.violate(format!(
                "negative-black node left in the tree at rest {}",
                at
            ));
        }
        if is_root && node.color != Color::Black {
            report.violate(format!("root is not black (color={:?})", node.color));
        }
        if node.left.is_none() || node.right.is_none() {
            report.violate(format!(
                "childless node that is not the black leaf {} (\"phantom\" node)",
                at
            ));
            return 0;
        }

        let left = node.left.as_ref().unwrap();
        let right = node.right.as_ref().unwrap();
        let left_bh = self.check_node(Some(left), false, depth + 1, report);
        let right_bh = self.check_node(Some(right), false, depth + 1, report);

        if node.color == Color::Red && (left.color == Color::Red || right.color == Color::Red) {
            report.violate(format!(
                "red node with a red child (red-red violation) {}",
                at
            ));
        }
        if left_bh != right_bh {
            report.violate(format!(
                "children have different black heights {}: left={}, right={}",
                at, left_bh, right_bh
            ));
        }
        let derived_bh = left_bh + if node.color == Color::Black { 1 } else { 0 };
        if node.black_height != derived_bh {
            report.violate(format!(
                "stored black_height is inconsistent {}: stored={}, derived from children={}",
                at, node.black_height, derived_bh
            ));
        }
        if node.left_subtree_len != left.subtree_len {
            report.violate(format!(
                "left_subtree_len stale {}: stored={}, actual={}",
                at, node.left_subtree_len, left.subtree_len
            ));
        }
        let derived_len = left.subtree_len + node.length + right.subtree_len;
        if node.subtree_len != derived_len {
            report.violate(format!(
                "subtree_len stale {}: stored={}, actual={}",
                at, node.subtree_len, derived_len
            ));
        }

        derived_bh
    }
}
