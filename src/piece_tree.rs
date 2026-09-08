use std::rc::Rc;

struct PieceTree {
    original: String,
    add: String,
    root: Option<Rc<Node>>,
    undo_stack: Vec<Rc<Node>>,
    redo_stack: Vec<Rc<Node>>,
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
            }
        } else {
            let original = String::from(original_content);
            let root = Some(Rc::new(Node::new(
                0,
                original.len(),
                BufferType::Original,
                Color::Black,
                None,
                None,
            )));

            Self {
                original,
                add: String::new(),
                root,
                undo_stack: Vec::new(),
                redo_stack: Vec::new(),
            }
        }
    }
    pub fn delete(&mut self,index:usize,length:usize){

    }
    fn delete_node(node:Option<Rc<Node>>,index:usize,length:usize)->Option<Rc<Node>>{
        match node{
            Some(curr_node) =>{
                if index< curr_node.left_subtree_len{
                    let new_left= Self::delete_node(curr_node.left.clone(), index, length);
                    let new_current_node = Rc::new(Node::new(curr_node.start, curr_node.length, curr_node.buffer_type, curr_node.color,new_left, curr_node.right.clone()));
                    Some(Self::bubble(new_current_node))
                }else if index>= curr_node.left_subtree_len+ curr_node.length{
                    let new_index=index - curr_node.left_subtree_len-curr_node.length;
                    let new_right= Self::delete_node(curr_node.right.clone(), new_index, length);
                    let new_current_node = Rc::new(Node::new(curr_node.start, curr_node.length, curr_node.buffer_type, curr_node.color,curr_node.left.clone(),new_right ));
                    Some(Self::bubble(new_current_node))
                }else{
                    let offset_in_node=index-curr_node.left_subtree_len;
                    if offset_in_node ==0 && curr_node.length == length{
                        if curr_node.left.is_some() && curr_node.right.is_some(){
                            let replacement=Self::find_right_most(curr_node.left.as_ref().unwrap().clone());
                            let new_curr_node = Rc::new(Node::new(replacement.start, replacement.length, replacement.buffer_type, replacement.color,curr_node.left.clone(),curr_node.right.clone() ));

                        }

                    }
                    None

                }
            },
            None => None,
        }

    }
    fn find_right_most(node:Rc<Node>)->Rc<Node>{
        match node.right.as_ref(){
            Some(r) => {
                Self::find_right_most(r.clone())
            },
            None => {
                node
            },
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
        let subtree_len = left_subtree_len + length +right_subtree_len;
        Self {
            start,
            length,
            buffer_type,
            color,
            left,
            right,
            left_subtree_len,
            subtree_len
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

#[derive(Clone, Copy)]
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
