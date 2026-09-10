use std::rc::Rc;

struct PieceTree {
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
    pub fn delete(&mut self, index: usize, length: usize) {}
    fn delete_node(&self, curr_node: Rc<Node>, index: usize, length: usize) -> DeleteMetaData {
        if index < curr_node.left_subtree_len {
            let new_left =
                self.delete_node(curr_node.left.as_ref().unwrap().clone(), index, length);
            let new_current_node = Rc::new(Node::new(
                curr_node.start,
                curr_node.length,
                curr_node.buffer_type,
                curr_node.color,
                Some(new_left.new_node.clone()),
                curr_node.right.clone(),
            ));
            if new_left.new_length == 0 {
                DeleteMetaData {
                    new_node: Self::bubble(new_current_node),
                    new_index: new_left.new_index,
                    new_length: new_left.new_length,
                }
            } else {
                let new_current_node = self.delete_on_target_node(
                    new_current_node,
                    new_left.new_index,
                    new_left.new_length,
                );
                if new_current_node.new_length == 0 {
                    DeleteMetaData {
                        new_node: Self::bubble(new_current_node.new_node.clone()),
                        new_index: new_left.new_index,
                        new_length: new_left.new_length,
                    }
                } else {
                    let new_index =
                        new_current_node.new_index - curr_node.left_subtree_len - curr_node.length;
                    let new_right = self.delete_node(
                        curr_node.right.as_ref().unwrap().clone(),
                        new_index,
                        new_current_node.new_length,
                    );
                    let new_current_node = Rc::new(Node::new(
                        new_current_node.new_node.start,
                        new_current_node.new_node.length,
                        new_current_node.new_node.buffer_type,
                        new_current_node.new_node.color,
                        new_current_node.new_node.left.clone(),
                        Some(new_right.new_node.clone()),
                    ));
                    DeleteMetaData {
                        new_node: Self::bubble(new_current_node),
                        new_index: new_right.new_index,
                        new_length: new_right.new_length,
                    }
                }
            }
        } else if index >= curr_node.left_subtree_len + curr_node.length {
            let new_index = index - curr_node.left_subtree_len - curr_node.length;
            let new_right =
                self.delete_node(curr_node.right.as_ref().unwrap().clone(), new_index, length);
            let new_current_node = Rc::new(Node::new(
                curr_node.start,
                curr_node.length,
                curr_node.buffer_type,
                curr_node.color,
                curr_node.left.clone(),
                Some(new_right.new_node.clone()),
            ));
            DeleteMetaData {
                new_node: Self::bubble(new_current_node),
                new_index: new_right.new_index,
                new_length: new_right.new_length,
            }
        } else {
            self.delete_on_target_node(curr_node, index, length)
        }
    }
    fn delete_on_target_node(
        &self,
        curr_node: Rc<Node>,
        index: usize,
        length: usize,
    ) -> DeleteMetaData {
        let offset_in_node = index - curr_node.left_subtree_len;
        if offset_in_node == 0 && length >= curr_node.length {
            if curr_node.left.as_ref().unwrap().clone() == self.black_leaf
                && curr_node.right.as_ref().unwrap().clone() == self.black_leaf
            {
                if curr_node.color == Color::Black {
                    DeleteMetaData {
                        new_node: self.double_black_leaf.clone(),
                        new_index: index + curr_node.length,
                        new_length: length - curr_node.length,
                    }
                } else {
                    DeleteMetaData {
                        new_node: self.black_leaf.clone(),
                        new_index: index + curr_node.length,
                        new_length: length - curr_node.length,
                    }
                }
            } else if curr_node.left.as_ref().unwrap().clone() != self.black_leaf
                && curr_node.right.as_ref().unwrap().clone() != self.black_leaf
            {
                let left_child = curr_node.left.as_ref().unwrap().clone();
                let replacement_info = self.find_right_most(left_child.clone());
                let new_left = self.remove_right_most(left_child);
                let new_current_node = Rc::new(Node::new(
                    replacement_info.start,
                    replacement_info.length,
                    replacement_info.buffer_type,
                    replacement_info.color,
                    Some(new_left),
                    curr_node.right.clone(),
                ));
                DeleteMetaData {
                    new_node: Self::bubble(new_current_node),
                    new_index: index + curr_node.length,
                    new_length: length - curr_node.length,
                }
            } else {
                let right_child = curr_node.right.as_ref().unwrap().clone();
                let new_current_node = Rc::new(Node::new(
                    right_child.start,
                    right_child.length,
                    right_child.buffer_type,
                    Color::Black,
                    right_child.left.clone(),
                    right_child.right.clone(),
                ));

                DeleteMetaData {
                    new_node: new_current_node,
                    new_index: index + curr_node.length,
                    new_length: length - curr_node.length,
                }
            }
        } else if offset_in_node == 0 && length < curr_node.length {
            let new_current_node = Rc::new(Node::new(
                curr_node.start + length,
                curr_node.length - length,
                curr_node.buffer_type,
                curr_node.color,
                curr_node.left.clone(),
                curr_node.right.clone(),
            ));
            DeleteMetaData {
                new_node: new_current_node,
                new_index: index + curr_node.length,
                new_length: 0,
            }
        } else if offset_in_node > 0 && offset_in_node + length >= curr_node.length {
            let new_current_node = Rc::new(Node::new(
                curr_node.start,
                offset_in_node,
                curr_node.buffer_type,
                curr_node.color,
                curr_node.left.clone(),
                curr_node.right.clone(),
            ));
            DeleteMetaData {
                new_node: new_current_node,
                new_index: index + (curr_node.length - offset_in_node),
                new_length: length - (curr_node.length - offset_in_node),
            }
        } else {
            let second_part = Rc::new(Node::new(
                curr_node.start,
                offset_in_node,
                curr_node.buffer_type,
                curr_node.color,
                Some(self.black_leaf.clone()),
                Some(self.black_leaf.clone()),
            ));
            let right = curr_node.right.as_ref().unwrap().clone();

            if right == self.black_leaf {
                let first_part = Rc::new(Node::new(
                    curr_node.start,
                    offset_in_node,
                    curr_node.buffer_type,
                    curr_node.color,
                    curr_node.left.clone(),
                    Some(second_part),
                ));
                return DeleteMetaData {
                    new_node: first_part,
                    new_index: index + (curr_node.length - offset_in_node),
                    new_length: 0,
                };
            } else {
                let right = self.insert_as_successor(right, second_part);
                let first_part = Rc::new(Node::new(
                    curr_node.start,
                    offset_in_node,
                    curr_node.buffer_type,
                    curr_node.color,
                    curr_node.left.clone(),
                    Some(right),
                ));
                return DeleteMetaData {
                    new_node: Self::rebalance(first_part),
                    new_index: index + (curr_node.length - offset_in_node),
                    new_length: 0,
                };
            }
        }
    }
    fn insert_as_successor(&self, curr_node: Rc<Node>, node_to_insert: Rc<Node>) -> Rc<Node> {
        if curr_node.left.as_ref().unwrap().clone() == self.black_leaf {
            let new_node = Rc::new(Node::new(
                curr_node.start,
                curr_node.length,
                curr_node.buffer_type,
                curr_node.color,
                Some(node_to_insert),
                curr_node.right.clone(),
            ));
            return Self::rebalance(new_node);
        }
        let left_path =
            self.insert_as_successor(curr_node.left.as_ref().unwrap().clone(), node_to_insert);
        let new_node = Rc::new(Node::new(
            curr_node.start,
            curr_node.length,
            curr_node.buffer_type,
            curr_node.color,
            Some(left_path),
            curr_node.right.clone(),
        ));
        Self::rebalance(new_node)
    }
    fn remove_right_most(&self, node: Rc<Node>) -> Rc<Node> {
        if node.right.as_ref().unwrap().clone() == self.black_leaf {
            if node.color == Color::Black {
                return self.double_black_leaf.clone();
            } else {
                return self.black_leaf.clone();
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
        Self::bubble(new_current_node)
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
    fn bubble(y: Rc<Node>) -> Rc<Node> {
        if y.color == Color::Black
            && y.right_color() == Color::DoubleBlack
            && y.left_color() == Color::Black
        {
            let x = y.left.as_ref().unwrap().clone();
            let z = y.right.as_ref().unwrap().clone();
            let new_x = Rc::new(Node::new(
                x.start,
                x.length,
                x.buffer_type,
                Color::Red,
                x.left.clone(),
                x.right.clone(),
            ));
            let new_z = Rc::new(Node::new(
                z.start,
                z.length,
                z.buffer_type,
                Color::Black,
                z.left.clone(),
                z.right.clone(),
            ));

            let new_y = Rc::new(Node::new(
                z.start,
                z.length,
                z.buffer_type,
                Color::DoubleBlack,
                Some(new_x),
                Some(new_z),
            ));
            let new_y = Self::rebalance(new_y);
            return new_y;
        } else if y.color == Color::Red
            && y.right_color() == Color::DoubleBlack
            && y.left_color() == Color::Black
        {
            let x = y.left.as_ref().unwrap().clone();
            let z = y.right.as_ref().unwrap().clone();
            let new_x = Rc::new(Node::new(
                x.start,
                x.length,
                x.buffer_type,
                Color::Red,
                x.left.clone(),
                x.right.clone(),
            ));
            let new_z = Rc::new(Node::new(
                z.start,
                z.length,
                z.buffer_type,
                Color::Black,
                z.left.clone(),
                z.right.clone(),
            ));

            let new_y = Rc::new(Node::new(
                z.start,
                z.length,
                z.buffer_type,
                Color::Black,
                Some(new_x),
                Some(new_z),
            ));
            let new_y = Self::rebalance(new_y);
            return new_y;
        } else if y.color == Color::Black
            && y.right_color() == Color::DoubleBlack
            && y.left_color() == Color::Red
        {
            let x = y.left.as_ref().unwrap().clone();
            let z = y.right.as_ref().unwrap().clone();
            let new_x = Rc::new(Node::new(
                x.start,
                x.length,
                x.buffer_type,
                Color::NegativeBlack,
                x.left.clone(),
                x.right.clone(),
            ));
            let new_z = Rc::new(Node::new(
                z.start,
                z.length,
                z.buffer_type,
                Color::Black,
                z.left.clone(),
                z.right.clone(),
            ));

            let new_y = Rc::new(Node::new(
                z.start,
                z.length,
                z.buffer_type,
                Color::DoubleBlack,
                Some(new_x),
                Some(new_z),
            ));
            let new_y = Self::rebalance(new_y);
            return new_y;
        } else if y.color == Color::Black
            && y.left_color() == Color::DoubleBlack
            && y.right_color() == Color::Black
        {
            let x = y.left.as_ref().unwrap().clone();
            let z = y.right.as_ref().unwrap().clone();
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
                Color::Red,
                z.left.clone(),
                z.right.clone(),
            ));

            let new_y = Rc::new(Node::new(
                z.start,
                z.length,
                z.buffer_type,
                Color::DoubleBlack,
                Some(new_x),
                Some(new_z),
            ));
            let new_y = Self::rebalance(new_y);
            return new_y;
        } else if y.color == Color::Red
            && y.left_color() == Color::DoubleBlack
            && y.right_color() == Color::Black
        {
            let x = y.left.as_ref().unwrap().clone();
            let z = y.right.as_ref().unwrap().clone();
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
                Color::Red,
                z.left.clone(),
                z.right.clone(),
            ));

            let new_y = Rc::new(Node::new(
                z.start,
                z.length,
                z.buffer_type,
                Color::Black,
                Some(new_x),
                Some(new_z),
            ));
            let new_y = Self::rebalance(new_y);
            return new_y;
        } else if y.color == Color::Black
            && y.left_color() == Color::DoubleBlack
            && y.right_color() == Color::Red
        {
            let x = y.left.as_ref().unwrap().clone();
            let z = y.right.as_ref().unwrap().clone();
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
                Color::NegativeBlack,
                z.left.clone(),
                z.right.clone(),
            ));

            let new_y = Rc::new(Node::new(
                z.start,
                z.length,
                z.buffer_type,
                Color::DoubleBlack,
                Some(new_x),
                Some(new_z),
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
            && z.left_right_color() == Color::Black
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
            && z.right_left_color() == Color::Black
        {
            let x = z;
            let z = x.right.as_ref().unwrap().clone();
            let y = z.left.as_ref().unwrap().clone();
            let w = x.left.as_ref().unwrap().clone();
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
        }
        z
    }
}
#[derive(PartialEq)]
struct Node {
    start: usize,
    length: usize,
    buffer_type: BufferType,
    color: Color,
    left_subtree_len: usize,
    subtree_len: usize,
    left: Option<Rc<Node>>,
    right: Option<Rc<Node>>,
}

struct NodeInfo {
    start: usize,
    length: usize,
    color: Color,
    buffer_type: BufferType,
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
        let left_subtree_len = match left.as_ref() {
            Some(l) => l.subtree_len,
            None => 0,
        };
        let right_subtree_len = match right.as_ref() {
            Some(r) => r.subtree_len,
            None => 0,
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
    fn are_both_left_grandchildren_black(&self) -> bool {
        self.left_left_color() == Color::Black && self.left_right_color() == Color::Black
    }
    fn are_both_right_grandchildren_black(&self) -> bool {
        self.right_right_color() == Color::Black && self.right_left_color() == Color::Black
    }
}

#[derive(Clone, Copy, PartialEq)]
enum BufferType {
    Original,
    Add,
}

#[derive(Clone, Copy, PartialEq)]
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

struct DeleteMetaData {
    new_node: Rc<Node>,
    new_index: usize,
    new_length: usize,
}
