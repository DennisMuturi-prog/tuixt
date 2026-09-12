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
    pub fn insert(&mut self, content: &str, index: usize) {
        // A zero-length insertion has nothing to insert. Doing the work anyway
        // splits the piece it lands in and leaves a zero-length piece behind,
        // which the delete machinery then trips over.
        if content.is_empty() {
            return;
        }
        let start = self.add.len();
        self.add.push_str(content);
        let node_info_to_insert = NodeInfo {
            start,
            length: content.len(),
            color: Color::Red,
            buffer_type: BufferType::Add,
        };
        if let Some(root_node) = self.root.as_ref().cloned() {
            self.undo_stack.push(root_node);
        }

        // An insertion point that lands strictly inside a piece has to split that piece
        // before anything can be inserted. The split is done by shrinking the piece to
        // its leading part, which changes only its span: the node keeps its colour and
        // both of its children, so the tree keeps its shape and its black heights and
        // needs no repair. The trailing part is then inserted as a second, ordinary
        // insertion. Both insertions therefore go through the piece-boundary path in
        // `insert_node`, which is what keeps the red-black invariants intact: building
        // the two halves and the new node by hand is what used to break the
        // equal-black-height rule.
        let trailing_part = self.shrink_piece_containing(index);

        self.insert_info(node_info_to_insert, index);

        if let Some(trailing_part) = trailing_part {
            self.insert_info(trailing_part, index + content.len());
        }
    }

    /// Insert one piece at a position that is a piece boundary, i.e. at the very
    /// start of an existing piece, or at the very end of the document.
    fn insert_info(&mut self, node_info_to_insert: NodeInfo, index: usize) {
        if let Some(root_node) = self.root.take() {
            let node_to_insert = Rc::new(Node::new(
                node_info_to_insert.start,
                node_info_to_insert.length,
                node_info_to_insert.buffer_type,
                node_info_to_insert.color,
                Some(self.black_leaf.clone()),
                Some(self.black_leaf.clone()),
            ));
            if index >= root_node.subtree_len {
                let right = root_node.right.as_ref().unwrap().clone();
                if right == self.black_leaf {
                    let new_current_node = Rc::new(Node::new(
                        root_node.start,
                        root_node.length,
                        root_node.buffer_type,
                        root_node.color,
                        root_node.left.clone(),
                        Some(node_to_insert),
                    ));
                    self.root = Some(new_current_node);
                } else {
                    let new_right = self.insert_as_predecessor(right, node_to_insert);
                    let new_current_node = Rc::new(Node::new(
                        root_node.start,
                        root_node.length,
                        root_node.buffer_type,
                        root_node.color,
                        root_node.left.clone(),
                        Some(new_right),
                    ));
                    let new_root = Self::rebalance(new_current_node);
                    let new_current_node = Rc::new(Node::new(
                        new_root.start,
                        new_root.length,
                        new_root.buffer_type,
                        Color::Black,
                        new_root.left.clone(),
                        new_root.right.clone(),
                    ));
                    self.root = Some(new_current_node);
                }
            } else {
                let new_root = self.insert_node(root_node.clone(), node_info_to_insert, index);
                let new_current_node = Rc::new(Node::new(
                    new_root.start,
                    new_root.length,
                    new_root.buffer_type,
                    Color::Black,
                    new_root.left.clone(),
                    new_root.right.clone(),
                ));
                self.root = Some(new_current_node);
            }
        } else {
            let node_to_insert = Rc::new(Node::new(
                node_info_to_insert.start,
                node_info_to_insert.length,
                node_info_to_insert.buffer_type,
                Color::Black,
                Some(self.black_leaf.clone()),
                Some(self.black_leaf.clone()),
            ));
            self.root = Some(node_to_insert);
        }
    }
    /// If `index` falls strictly inside one piece, shrink that piece to its leading
    /// part and return the trailing part, ready to be inserted after the new text.
    /// Shrinking changes only the piece's span: the node keeps its colour and both of
    /// its children, so every red-black invariant is preserved and nothing needs to
    /// be rebalanced. Returns `None` when `index` is already a piece boundary or
    /// beyond the end of the document, in which case there is nothing to split.
    fn shrink_piece_containing(&mut self, index: usize) -> Option<NodeInfo> {
        let root = self.root.as_ref()?.clone();
        let (shrunk_root, trailing_part) = self.shrink_piece(root, index)?;
        self.root = Some(shrunk_root);
        Some(trailing_part)
    }

    /// Walks down to the piece that strictly contains `index`, if there is one, and
    /// returns the rebuilt path together with the trailing part of the split piece.
    fn shrink_piece(&self, curr_node: Rc<Node>, index: usize) -> Option<(Rc<Node>, NodeInfo)> {
        if index >= curr_node.subtree_len {
            return None;
        }
        if index < curr_node.left_subtree_len {
            let (new_left, trailing_part) =
                self.shrink_piece(curr_node.left.as_ref().unwrap().clone(), index)?;
            let rebuilt = Rc::new(Node::new(
                curr_node.start,
                curr_node.length,
                curr_node.buffer_type,
                curr_node.color,
                Some(new_left),
                curr_node.right.clone(),
            ));
            Some((rebuilt, trailing_part))
        } else if index > curr_node.left_subtree_len + curr_node.length {
            let inner_index = index - curr_node.left_subtree_len - curr_node.length;
            let (new_right, trailing_part) =
                self.shrink_piece(curr_node.right.as_ref().unwrap().clone(), inner_index)?;
            let rebuilt = Rc::new(Node::new(
                curr_node.start,
                curr_node.length,
                curr_node.buffer_type,
                curr_node.color,
                curr_node.left.clone(),
                Some(new_right),
            ));
            Some((rebuilt, trailing_part))
        } else if index == curr_node.left_subtree_len
            || index == curr_node.left_subtree_len + curr_node.length
        {
            // already a piece boundary
            None
        } else {
            let offset_in_node = index - curr_node.left_subtree_len;
            let leading_part = Rc::new(Node::new(
                curr_node.start,
                offset_in_node,
                curr_node.buffer_type,
                curr_node.color,
                curr_node.left.clone(),
                curr_node.right.clone(),
            ));
            let trailing_part = NodeInfo {
                start: curr_node.start + offset_in_node,
                length: curr_node.length - offset_in_node,
                color: Color::Red,
                buffer_type: curr_node.buffer_type,
            };
            Some((leading_part, trailing_part))
        }
    }

    fn insert_node(&self, curr_node: Rc<Node>, node_to_insert: NodeInfo, index: usize) -> Rc<Node> {
        if index < curr_node.left_subtree_len {
            let new_left = self.insert_node(
                curr_node.left.as_ref().unwrap().clone(),
                node_to_insert,
                index,
            );
            let new_current_node = Rc::new(Node::new(
                curr_node.start,
                curr_node.length,
                curr_node.buffer_type,
                curr_node.color,
                Some(new_left),
                curr_node.right.clone(),
            ));
            Self::rebalance(new_current_node)
        } else if index >= curr_node.left_subtree_len + curr_node.length {
            let new_index = index - curr_node.left_subtree_len - curr_node.length;
            let new_right = self.insert_node(
                curr_node.right.as_ref().unwrap().clone(),
                node_to_insert,
                new_index,
            );
            let new_current_node = Rc::new(Node::new(
                curr_node.start,
                curr_node.length,
                curr_node.buffer_type,
                curr_node.color,
                curr_node.left.clone(),
                Some(new_right),
            ));
            Self::rebalance(new_current_node)
        } else {
            let _offset_in_node = index - curr_node.left_subtree_len;
            // `insert` splits any piece that the insertion point falls inside before
            // calling here, so `index` is always on a piece boundary and the only
            // remaining mid-piece case is "insert immediately in front of this node".
            debug_assert_eq!(
                _offset_in_node, 0,
                "insert_node reached an index inside a piece (offset {})",
                _offset_in_node
            );
            {
                let node_to_insert = Rc::new(Node::new(
                    node_to_insert.start,
                    node_to_insert.length,
                    node_to_insert.buffer_type,
                    node_to_insert.color,
                    Some(self.black_leaf.clone()),
                    Some(self.black_leaf.clone()),
                ));
                let left = curr_node.left.as_ref().unwrap().clone();
                if left == self.black_leaf {
                    let new_current_node = Rc::new(Node::new(
                        curr_node.start,
                        curr_node.length,
                        curr_node.buffer_type,
                        curr_node.color,
                        Some(node_to_insert),
                        curr_node.right.clone(),
                    ));
                    Self::rebalance(new_current_node)
                } else {
                    let new_left = self.insert_as_predecessor(left, node_to_insert);
                    let new_current_node = Rc::new(Node::new(
                        curr_node.start,
                        curr_node.length,
                        curr_node.buffer_type,
                        curr_node.color,
                        Some(new_left),
                        curr_node.right.clone(),
                    ));
                    Self::rebalance(new_current_node)
                }
            }
        }
    }
    pub fn delete(&mut self, index: usize, length: usize) {
        if let Some(root_node) = self.root.take() {
            if index >= root_node.subtree_len {
                self.root = Some(root_node);
                return;
            }
            self.undo_stack.push(root_node.clone());
            if index == 0 {
                if length >= root_node.subtree_len {
                    self.root = None;
                    return;
                } else {
                    let (_, t2) = self.split(root_node, length);
                    let new_current_node = Rc::new(Node::new(
                        t2.start,
                        t2.length,
                        t2.buffer_type,
                        Color::Black,
                        t2.left.clone(),
                        t2.right.clone(),
                    ));
                    self.root = Some(new_current_node);
                    return;
                }
            }
            let (t1, t2) = self.split(root_node, index);
            if length >= t2.subtree_len {
                let new_root = t1.clone();
                let new_current_node = Rc::new(Node::new(
                    new_root.start,
                    new_root.length,
                    new_root.buffer_type,
                    Color::Black,
                    new_root.left.clone(),
                    new_root.right.clone(),
                ));

                self.root = Some(new_current_node);
            } else {
                let (_, t3) = self.split(t2.clone(), length);
                let new_root = self.merge(t1.clone(), t3);
                let new_current_node = Rc::new(Node::new(
                    new_root.start,
                    new_root.length,
                    new_root.buffer_type,
                    Color::Black,
                    new_root.left.clone(),
                    new_root.right.clone(),
                ));

                self.root = Some(new_current_node);
            }
        }
    }
    fn insert_at_rightmost_with_target_black_height(
        current_node: Rc<Node>,
        pivot: Rc<Node>,
        t2: Rc<Node>,
        target_black_height: i32,
    ) -> Rc<Node> {
        if current_node.black_height <= target_black_height {
            Rc::new(Node::new(
                pivot.start,
                pivot.length,
                pivot.buffer_type,
                Color::Red,
                Some(current_node),
                Some(t2),
            ))
        } else {
            let new_right = Self::insert_at_rightmost_with_target_black_height(
                current_node.right.as_ref().unwrap().clone(),
                pivot,
                t2,
                target_black_height,
            );
            Self::rebalance(Rc::new(Node::new(
                current_node.start,
                current_node.length,
                current_node.buffer_type,
                current_node.color,
                current_node.left.clone(),
                Some(new_right),
            )))
        }
    }
    fn insert_at_leftmost_with_target_black_height(
        current_node: Rc<Node>,
        pivot: Rc<Node>,
        t1: Rc<Node>,
        target_black_height: i32,
    ) -> Rc<Node> {
        if current_node.black_height <= target_black_height {
            Rc::new(Node::new(
                pivot.start,
                pivot.length,
                pivot.buffer_type,
                Color::Red,
                Some(t1),
                Some(current_node),
            ))
        } else {
            let new_left = Self::insert_at_leftmost_with_target_black_height(
                current_node.left.as_ref().unwrap().clone(),
                pivot,
                t1,
                target_black_height,
            );
            Self::rebalance(Rc::new(Node::new(
                current_node.start,
                current_node.length,
                current_node.buffer_type,
                current_node.color,
                Some(new_left),
                current_node.right.clone(),
            )))
        }
    }
    fn join(t1: Rc<Node>, pivot: Rc<Node>, t2: Rc<Node>) -> Rc<Node> {
        let joined = if t1.black_height == t2.black_height {
            // The pivot is black here, so a `rebalance` can still repair a red-red
            // violation that arrived at the root of either input. Both inputs may carry
            // one pending violation at their root (that is the contract the recursive
            // join relies on); embedding them under the new pivot without repairing
            // would push that violation one level deeper, where no later repair looks.
            Self::rebalance(Rc::new(Node::new(
                pivot.start,
                pivot.length,
                pivot.buffer_type,
                Color::Black,
                Some(t1),
                Some(t2),
            )))
        } else if t1.black_height > t2.black_height {
            let target_black_height = t2.black_height;
            Self::insert_at_rightmost_with_target_black_height(t1, pivot, t2, target_black_height)
        } else {
            let target_black_height = t1.black_height;
            Self::insert_at_leftmost_with_target_black_height(t2, pivot, t1, target_black_height)
        };

        // The two recursive insertions above build a red node at the bottom of the
        // spine they walk and repair red-red violations on the way back up, but
        // `rebalance` is a no-op on a red node, so a violation can be left sitting
        // at the *root* of the joined tree (a red node with a red child). Nothing
        // above a join is guaranteed to look at that shape again -- `split` hands
        // the joined tree straight back to its caller, and `delete` rebuilds the
        // document from the pieces it gets -- so repair it here. Blackening the root
        // always repairs it: a black node may have red children, and the children
        // keep their equal black heights. The only cost is one extra black on every
        // path, which callers cannot observe: black heights are only ever compared
        // with each other, and the tree stays valid.
        if joined.color == Color::Red
            && (joined.left_color() == Color::Red || joined.right_color() == Color::Red)
        {
            Rc::new(Node::new(
                joined.start,
                joined.length,
                joined.buffer_type,
                Color::Black,
                joined.left.clone(),
                joined.right.clone(),
            ))
        } else {
            joined
        }
    }
    fn split(&self, current_node: Rc<Node>, index: usize) -> (Rc<Node>, Rc<Node>) {
        if index < current_node.left_subtree_len {
            let (t1, t2) = self.split(current_node.left.as_ref().unwrap().clone(), index);
            let pivot = Rc::new(Node::new(
                current_node.start,
                current_node.length,
                current_node.buffer_type,
                current_node.color,
                Some(self.black_leaf.clone()),
                Some(self.black_leaf.clone()),
            ));
            (
                t1,
                Self::join(t2, pivot, current_node.right.as_ref().unwrap().clone()),
            )
        } else if index >= current_node.left_subtree_len + current_node.length {
            let index = index - current_node.left_subtree_len - current_node.length;
            let (t1, t2) = self.split(current_node.right.as_ref().unwrap().clone(), index);
            let pivot = Rc::new(Node::new(
                current_node.start,
                current_node.length,
                current_node.buffer_type,
                current_node.color,
                Some(self.black_leaf.clone()),
                Some(self.black_leaf.clone()),
            ));
            (
                Self::join(current_node.left.as_ref().unwrap().clone(), pivot, t1),
                t2,
            )
        } else {
            let offset_in_node = index - current_node.left_subtree_len;
            if offset_in_node == 0 {
                let t1 = Rc::new(Node::new(
                    current_node.start,
                    current_node.length,
                    current_node.buffer_type,
                    current_node.color,
                    Some(self.black_leaf.clone()),
                    Some(self.black_leaf.clone()),
                ));
                let t2 = self.merge(t1, current_node.right.as_ref().unwrap().clone());
                (current_node.left.as_ref().unwrap().clone(), t2)
            } else {
                let first_part = Rc::new(Node::new(
                    current_node.start,
                    offset_in_node,
                    current_node.buffer_type,
                    current_node.color,
                    Some(self.black_leaf.clone()),
                    Some(self.black_leaf.clone()),
                ));
                let second_part = Rc::new(Node::new(
                    current_node.start + offset_in_node,
                    current_node.length - offset_in_node,
                    current_node.buffer_type,
                    current_node.color,
                    Some(self.black_leaf.clone()),
                    Some(self.black_leaf.clone()),
                ));
                let t1 = self.merge(current_node.left.as_ref().unwrap().clone(), first_part);
                let t2 = self.merge(second_part, current_node.right.as_ref().unwrap().clone());
                (t1, t2)
            }
        }
    }
    fn merge(&self, t1: Rc<Node>, t2: Rc<Node>) -> Rc<Node> {
        if t1 == self.black_leaf {
            return t2;
        }
        if t2 == self.black_leaf {
            return t1;
        }
        let pivot = self.find_right_most(t1.clone());
        let t1 = self.remove_right_most(t1);
        let t1 = if t1 == self.double_black_leaf {
            self.black_leaf.clone()
        } else if t1.color == Color::DoubleBlack {
            // If the root of t1 became double-black from bubblin,absorbing it here
            Rc::new(Node::new(
                t1.start,
                t1.length,
                t1.buffer_type,
                Color::Black, // Absorb double-black at the root of t1
                t1.left.clone(),
                t1.right.clone(),
            ))
        } else {
            t1
        };
        let pivot = Rc::new(Node::new(
            pivot.start,
            pivot.length,
            pivot.buffer_type,
            Color::Red,
            Some(self.black_leaf.clone()),
            Some(self.black_leaf.clone()),
        ));
        Self::join(t1, pivot, t2)
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
            && z.left.as_ref().unwrap().has_legimate_black_internal_children()
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
            && z.right.as_ref().unwrap().has_legimate_black_internal_children()
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
            Some(l) => {l.length>0 && l.color == Color::Black},
            None => false,
        };
        let right = match self.right.as_ref() {
            Some(r) => {r.length>0 && r.color == Color::Black},
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
            report.violate(format!("negative-black node left in the tree at rest {}", at));
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
