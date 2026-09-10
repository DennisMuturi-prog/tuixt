pub mod piece_tree;

#[cfg(test)]
mod tests {
    use super::piece_tree;

    #[test]
    fn testing_piece_tree_operations() {
        let mut my_piece_tree = piece_tree::PieceTree::new("hello");
        my_piece_tree.insert(" ", 5);
        my_piece_tree.insert("w", 6);
        my_piece_tree.insert("r", 7);
        my_piece_tree.insert("l", 8);
        my_piece_tree.insert("d", 9);
        my_piece_tree.insert("o", 7);
        let mut content = String::new();
        my_piece_tree.get_text(&mut content);
        assert_eq!("hello world", content);
        my_piece_tree.delete(6, 3);
        content.clear();
        my_piece_tree.get_text(&mut content);
        assert_eq!("hello ld", content);
    }
}
