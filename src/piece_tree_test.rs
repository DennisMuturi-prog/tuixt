#[cfg(test)]
mod tests {
    use crate::piece_tree::PieceTree;
    // ------------------------------------------------------------
    // Test helpers
    // ------------------------------------------------------------

    fn text(tree: &PieceTree) -> String {
        let mut result = String::new();
        tree.get_text(&mut result);
        result
    }

    fn sub_text(tree: &PieceTree, start: usize, length: usize) -> String {
        let mut result = String::new();
        tree.get_sub_text(&mut result, start, length);
        result
    }

    fn line_text(tree: &PieceTree, line_number: usize) -> String {
        let mut result = String::new();
        tree.get_line_text(line_number, &mut result);
        result
    }

    // ============================================================
    // Construction
    // ============================================================

    #[test]
    fn new_empty_tree() {
        let tree = PieceTree::new("");

        assert_eq!(text(&tree), "");
    }

    #[test]
    fn new_tree_contains_original_content() {
        let tree = PieceTree::new("Hello, world!");

        assert_eq!(text(&tree), "Hello, world!");
    }

    #[test]
    fn new_tree_preserves_multiline_content() {
        let tree = PieceTree::new("one\ntwo\nthree");

        assert_eq!(text(&tree), "one\ntwo\nthree");
    }

    #[test]
    fn new_tree_preserves_trailing_newline() {
        let tree = PieceTree::new("one\ntwo\n");

        assert_eq!(text(&tree), "one\ntwo\n");
    }

    // ============================================================
    // Basic insertion
    // ============================================================

    #[test]
    fn insert_into_empty_tree() {
        let mut tree = PieceTree::new("");

        tree.insert("hello", 0);

        assert_eq!(text(&tree), "hello");
    }

    #[test]
    fn insert_at_beginning() {
        let mut tree = PieceTree::new("world");

        tree.insert("hello ", 0);

        assert_eq!(text(&tree), "hello world");
    }

    #[test]
    fn insert_at_end() {
        let mut tree = PieceTree::new("hello");

        tree.insert(" world", 5);

        assert_eq!(text(&tree), "hello world");
    }

    #[test]
    fn insert_in_middle() {
        let mut tree = PieceTree::new("helo");

        tree.insert("l", 3);

        assert_eq!(text(&tree), "hello");
    }

    #[test]
    fn insert_empty_string() {
        let mut tree = PieceTree::new("hello");

        tree.insert("", 2);

        assert_eq!(text(&tree), "hello");
    }

    #[test]
    fn insert_newline() {
        let mut tree = PieceTree::new("hello world");

        tree.insert("\n", 5);

        assert_eq!(text(&tree), "hello\n world");
    }

    #[test]
    fn insert_multiple_lines() {
        let mut tree = PieceTree::new("hello");

        tree.insert("\none\ntwo\nthree", 5);

        assert_eq!(text(&tree), "hello\none\ntwo\nthree");
    }

    // ============================================================
    // Creating piece boundaries
    //
    // These tests are particularly important for a PieceTree.
    // They force insertions to split existing pieces and create
    // adjacent pieces.
    // ============================================================

    #[test]
    fn insert_in_middle_creates_piece_boundary() {
        let mut tree = PieceTree::new("abcdefghij");

        tree.insert("XXX", 5);

        assert_eq!(text(&tree), "abcdeXXXfghij");
    }

    #[test]
    fn insert_at_piece_boundary() {
        let mut tree = PieceTree::new("abcdefghij");

        tree.insert("XXX", 5);
        tree.insert("YYY", 5);

        assert_eq!(text(&tree), "abcdeYYYXXXfghij");
    }

    #[test]
    fn insert_at_end_of_inserted_piece() {
        let mut tree = PieceTree::new("abcdefghij");

        tree.insert("XXX", 5);
        tree.insert("YYY", 8);

        assert_eq!(text(&tree), "abcdeXXXYYYfghij");
    }

    #[test]
    fn multiple_insertions_create_many_pieces() {
        let mut tree = PieceTree::new("abcdefghij");

        tree.insert("111", 2);
        tree.insert("222", 6);
        tree.insert("333", 12);

        assert_eq!(text(&tree), "ab111c222def333ghij");
    }

    #[test]
    fn repeated_middle_insertions() {
        let mut tree = PieceTree::new("0123456789");

        tree.insert("A", 5);
        tree.insert("B", 5);
        tree.insert("C", 5);
        tree.insert("D", 5);
        tree.insert("E", 5);

        assert_eq!(text(&tree), "01234EDCBA56789");
    }

    #[test]
    fn many_small_pieces() {
        let mut tree = PieceTree::new("abcdefghij");

        tree.insert("1", 1);
        tree.insert("2", 2);
        tree.insert("3", 3);
        tree.insert("4", 4);
        tree.insert("5", 5);
        tree.insert("6", 6);
        tree.insert("7", 7);
        tree.insert("8", 8);
        tree.insert("9", 9);

        assert_eq!(text(&tree), "a123456789bcdefghij");
    }

    // ============================================================
    // get_text across pieces
    // ============================================================

    #[test]
    fn get_text_after_multiple_piece_splits() {
        let mut tree = PieceTree::new("abcdefghij");

        tree.insert("111", 2);
        tree.insert("222", 6);
        tree.insert("333", 12);

        assert_eq!(text(&tree), "ab111c222def333ghij");
    }

    // ============================================================
    // get_sub_text across piece boundaries
    // ============================================================

    #[test]
    fn sub_text_inside_single_piece() {
        let tree = PieceTree::new("abcdefghij");

        assert_eq!(sub_text(&tree, 2, 4), "cdef");
    }

    #[test]
    fn sub_text_zero_length() {
        let tree = PieceTree::new("abcdefghij");

        assert_eq!(sub_text(&tree, 5, 0), "");
    }

    #[test]
    fn sub_text_entire_text() {
        let tree = PieceTree::new("abcdefghij");

        assert_eq!(sub_text(&tree, 0, 10), "abcdefghij");
    }

    #[test]
    fn sub_text_crosses_original_and_inserted_piece() {
        let mut tree = PieceTree::new("abcdefghij");

        tree.insert("XXX", 5);

        // abcdeXXXfghij
        //
        //      deXXXf
        //      ^^^^^^
        assert_eq!(sub_text(&tree, 3, 6), "deXXXf");
    }

    #[test]
    fn sub_text_crosses_inserted_and_original_piece() {
        let mut tree = PieceTree::new("abcdefghij");

        tree.insert("XXX", 5);

        assert_eq!(sub_text(&tree, 6, 5), "XXfgh");
    }

    #[test]
    fn sub_text_starts_at_piece_boundary() {
        let mut tree = PieceTree::new("abcdefghij");

        tree.insert("XXX", 5);

        assert_eq!(sub_text(&tree, 5, 3), "XXX");
        assert_eq!(sub_text(&tree, 8, 3), "fgh");
    }

    #[test]
    fn sub_text_ends_at_piece_boundary() {
        let mut tree = PieceTree::new("abcdefghij");

        tree.insert("XXX", 5);

        assert_eq!(sub_text(&tree, 3, 5), "deXXX");
        assert_eq!(sub_text(&tree, 5, 3), "XXX");
    }

    // ============================================================
    // Basic deletion
    // ============================================================

    #[test]
    fn delete_entire_text() {
        let mut tree = PieceTree::new("hello");

        tree.delete(0, 5);

        assert_eq!(text(&tree), "");
    }

    #[test]
    fn delete_from_beginning() {
        let mut tree = PieceTree::new("hello world");

        tree.delete(0, 6);

        assert_eq!(text(&tree), "world");
    }

    #[test]
    fn delete_from_end() {
        let mut tree = PieceTree::new("hello world");

        tree.delete(5, 6);

        assert_eq!(text(&tree), "hello");
    }

    #[test]
    fn delete_from_middle() {
        let mut tree = PieceTree::new("hello world");

        tree.delete(5, 1);

        assert_eq!(text(&tree), "helloworld");
    }

    #[test]
    fn delete_zero_length() {
        let mut tree = PieceTree::new("hello");

        tree.delete(2, 0);

        assert_eq!(text(&tree), "hello");
    }

    // ============================================================
    // Delete inside a piece
    // ============================================================

    #[test]
    fn delete_middle_of_original_piece() {
        let mut tree = PieceTree::new("abcdefghij");

        tree.delete(3, 3);

        assert_eq!(text(&tree), "abcghij");
    }

    #[test]
    fn delete_middle_of_inserted_piece() {
        let mut tree = PieceTree::new("abcdefghij");

        tree.insert("12345", 5);

        assert_eq!(text(&tree), "abcde12345fghij");

        tree.delete(7, 2);

        assert_eq!(text(&tree), "abcde125fghij");
    }

    #[test]
    fn delete_entire_inserted_piece() {
        let mut tree = PieceTree::new("abcdefghij");

        tree.insert("12345", 5);

        tree.delete(5, 5);

        assert_eq!(text(&tree), "abcdefghij");
    }

    // ============================================================
    // Delete across two pieces
    // ============================================================

    #[test]
    fn delete_from_original_into_inserted_piece() {
        let mut tree = PieceTree::new("abcdefghij");

        tree.insert("XXX", 5);

        // abcdeXXXfghij
        //
        // abc[deXXX]fghij
        tree.delete(3, 5);

        assert_eq!(text(&tree), "abcfghij");
    }

    #[test]
    fn delete_from_inserted_into_original_piece() {
        let mut tree = PieceTree::new("abcdefghij");

        tree.insert("XXX", 5);

        // abcdeXXXfghij
        //
        // abcdeXX[Xfgh]ij
        tree.delete(7, 4);

        assert_eq!(text(&tree), "abcdeXXij");
    }

    #[test]
    fn delete_exactly_across_piece_boundary() {
        let mut tree = PieceTree::new("abcdefghij");

        tree.insert("XXX", 5);

        // Remove the entire inserted piece plus two chars
        // from the following original piece.
        tree.delete(5, 5);

        assert_eq!(text(&tree), "abcdehij");
    }

    #[test]
    fn delete_starts_at_piece_boundary() {
        let mut tree = PieceTree::new("abcdefghij");

        tree.insert("XXX", 5);

        tree.delete(5, 4);

        assert_eq!(text(&tree), "abcdeghij");
    }

    #[test]
    fn delete_ends_at_piece_boundary() {
        let mut tree = PieceTree::new("abcdefghij");

        tree.insert("XXX", 5);

        tree.delete(3, 5);

        assert_eq!(text(&tree), "abcfghij");
    }

    // ============================================================
    // Delete across multiple pieces
    // ============================================================

    #[test]
    fn delete_across_three_pieces() {
        let mut tree = PieceTree::new("abcdefghij");

        tree.insert("111", 2);
        tree.insert("222", 6);

        assert_eq!(text(&tree), "ab111c222defghij");

        // ab[111c222d]efghij
        tree.delete(2, 8);

        assert_eq!(text(&tree), "abefghij");
    }

    // ============================================================
    // Delete followed by insertion
    // ============================================================

    #[test]
    fn delete_piece_then_insert_at_same_position() {
        let mut tree = PieceTree::new("abcdefghij");

        tree.insert("XXX", 5);
        assert_eq!(text(&tree), "abcdeXXXfghij");

        tree.delete(5, 3);
        assert_eq!(text(&tree), "abcdefghij");

        tree.insert("YYY", 5);

        assert_eq!(text(&tree), "abcdeYYYfghij");
    }

    #[test]
    fn delete_piece_then_insert_before_position() {
        let mut tree = PieceTree::new("abcdefghij");

        tree.insert("XXX", 5);
        tree.delete(5, 3);

        tree.insert("YYY", 4);

        assert_eq!(text(&tree), "abcdYYYefghij");
    }

    #[test]
    fn delete_piece_then_insert_after_position() {
        let mut tree = PieceTree::new("abcdefghij");

        tree.insert("XXX", 5);
        tree.delete(5, 3);

        tree.insert("YYY", 6);

        assert_eq!(text(&tree), "abcdefYYYghij");
    }

    // ============================================================
    // Adjacent inserted pieces
    // ============================================================

    #[test]
    fn adjacent_insertions_create_many_small_pieces() {
        let mut tree = PieceTree::new("abcdefghij");

        tree.insert("A", 5);
        tree.insert("B", 6);
        tree.insert("C", 7);
        tree.insert("D", 8);
        tree.insert("E", 9);

        assert_eq!(text(&tree), "abcdeABCDEfghij");
    }

    #[test]
    fn delete_one_character_from_middle_of_many_pieces() {
        let mut tree = PieceTree::new("abcdefghij");

        tree.insert("A", 5);
        tree.insert("B", 6);
        tree.insert("C", 7);
        tree.insert("D", 8);
        tree.insert("E", 9);

        assert_eq!(text(&tree), "abcdeABCDEfghij");

        tree.delete(7, 1);

        assert_eq!(text(&tree), "abcdeABDEfghij");
    }

    #[test]
    fn delete_multiple_adjacent_inserted_pieces() {
        let mut tree = PieceTree::new("abcdefghij");

        tree.insert("A", 5);
        tree.insert("B", 6);
        tree.insert("C", 7);
        tree.insert("D", 8);
        tree.insert("E", 9);

        tree.delete(6, 3);

        assert_eq!(text(&tree), "abcdeAEfghij");
    }

    // ============================================================
    // Lines + piece boundaries
    // ============================================================

    #[test]
    fn first_line_includes_newline() {
        let tree = PieceTree::new("first\nsecond\nthird");

        assert_eq!(line_text(&tree, 0), "first\n");
    }

    #[test]
    fn middle_line_includes_newline() {
        let tree = PieceTree::new("first\nsecond\nthird");

        assert_eq!(line_text(&tree, 1), "second\n");
    }

    #[test]
    fn last_line_without_newline() {
        let tree = PieceTree::new("first\nsecond\nthird");

        assert_eq!(line_text(&tree, 2), "third");
    }

    #[test]
    fn last_line_with_newline() {
        let tree = PieceTree::new("first\nsecond\nthird\n");

        assert_eq!(line_text(&tree, 2), "third\n");
    }

    #[test]
    fn empty_document_has_empty_line() {
        let tree = PieceTree::new("");

        assert_eq!(line_text(&tree, 0), "");
    }

    #[test]
    fn trailing_newline_creates_empty_last_line() {
        let tree = PieceTree::new("hello\n");

        assert_eq!(line_text(&tree, 0), "hello\n");
        assert_eq!(line_text(&tree, 1), "");
    }

    #[test]
    fn consecutive_newlines_create_empty_lines() {
        let tree = PieceTree::new("one\n\nthree");

        assert_eq!(line_text(&tree, 0), "one\n");
        assert_eq!(line_text(&tree, 1), "\n");
        assert_eq!(line_text(&tree, 2), "three");
    }

    #[test]
    fn newline_in_inserted_piece_creates_line_boundary() {
        let mut tree = PieceTree::new("hello world");

        tree.insert("\none\n", 5);

        assert_eq!(text(&tree), "hello\none\n world");

        assert_eq!(line_text(&tree, 0), "hello\n");
        assert_eq!(line_text(&tree, 1), "one\n");
        assert_eq!(line_text(&tree, 2), " world");
    }

    #[test]
    fn delete_newline_at_piece_boundary_merges_lines() {
        let mut tree = PieceTree::new("hello");

        tree.insert("\n", 5);
        tree.insert("world", 6);

        assert_eq!(text(&tree), "hello\nworld");

        tree.delete(5, 1);

        assert_eq!(text(&tree), "helloworld");
        assert_eq!(line_text(&tree, 0), "helloworld");
    }


    // ============================================================
    // CRLF / CR / LF
    // ============================================================

    #[test]
    fn handles_lf() {
        let tree = PieceTree::new("one\ntwo\nthree");

        assert_eq!(line_text(&tree, 0), "one\n");
        assert_eq!(line_text(&tree, 1), "two\n");
        assert_eq!(line_text(&tree, 2), "three");
    }

    #[test]
    fn handles_crlf() {
        let tree = PieceTree::new("one\r\ntwo\r\nthree");

        assert_eq!(line_text(&tree, 0), "one\r\n");
        assert_eq!(line_text(&tree, 1), "two\r\n");
        assert_eq!(line_text(&tree, 2), "three");
    }

    #[test]
    fn inserted_crlf_creates_correct_line() {
        let mut tree = PieceTree::new("hello world");

        tree.insert("\r\n", 5);

        assert_eq!(text(&tree), "hello\r\n world");
        assert_eq!(line_text(&tree, 0), "hello\r\n");
        assert_eq!(line_text(&tree, 1), " world");
    }

    // ============================================================
    // Undo
    // ============================================================

    #[test]
    fn undo_insert_that_split_piece() {
        let mut tree = PieceTree::new("abcdefghij");

        tree.insert("XXX", 5);

        assert_eq!(text(&tree), "abcdeXXXfghij");

        tree.undo();

        assert_eq!(text(&tree), "abcdefghij");
    }

    #[test]
    fn undo_delete_inside_piece() {
        let mut tree = PieceTree::new("abcdefghij");

        tree.delete(3, 3);

        assert_eq!(text(&tree), "abcghij");

        tree.undo();

        assert_eq!(text(&tree), "abcdefghij");
    }

    #[test]
    fn undo_delete_across_piece_boundary() {
        let mut tree = PieceTree::new("abcdefghij");

        tree.insert("XXX", 5);
        tree.delete(3, 5);

        assert_eq!(text(&tree), "abcfghij");

        tree.undo();

        assert_eq!(text(&tree), "abcdeXXXfghij");
    }

    #[test]
    fn multiple_undos() {
        let mut tree = PieceTree::new("abc");

        tree.insert("1", 3);
        tree.insert("2", 4);
        tree.insert("3", 5);

        assert_eq!(text(&tree), "abc123");

        tree.undo();
        assert_eq!(text(&tree), "abc12");

        tree.undo();
        assert_eq!(text(&tree), "abc1");

        tree.undo();
        assert_eq!(text(&tree), "abc");
    }

    // ============================================================
    // Redo
    // ============================================================

    #[test]
    fn redo_insert_that_split_piece() {
        let mut tree = PieceTree::new("abcdefghij");

        tree.insert("XXX", 5);
        tree.undo();

        assert_eq!(text(&tree), "abcdefghij");

        tree.redo();

        assert_eq!(text(&tree), "abcdeXXXfghij");
    }

    #[test]
    fn redo_delete_across_piece_boundary() {
        let mut tree = PieceTree::new("abcdefghij");

        tree.insert("XXX", 5);
        tree.delete(3, 5);

        tree.undo();

        assert_eq!(text(&tree), "abcdeXXXfghij");

        tree.redo();

        assert_eq!(text(&tree), "abcfghij");
    }

    #[test]
    fn multiple_redos() {
        let mut tree = PieceTree::new("abc");

        tree.insert("1", 3);
        tree.insert("2", 4);
        tree.insert("3", 5);

        tree.undo();
        tree.undo();
        tree.undo();

        assert_eq!(text(&tree), "abc");

        tree.redo();
        assert_eq!(text(&tree), "abc1");

        tree.redo();
        assert_eq!(text(&tree), "abc12");

        tree.redo();
        assert_eq!(text(&tree), "abc123");
    }

    #[test]
    fn new_edit_after_undo_discards_redo() {
        let mut tree = PieceTree::new("abc");

        tree.insert("1", 3);
        tree.insert("2", 4);

        tree.undo();

        assert_eq!(text(&tree), "abc1");

        tree.insert("X", 4);

        assert_eq!(text(&tree), "abc1X");

        tree.redo();

        // The old "2" insertion should no longer be redoable.
        assert_eq!(text(&tree), "abc1X");
    }


    // ============================================================
    // Repeated structural operations
    // ============================================================

    #[test]
    fn repeated_split_and_delete() {
        let mut tree = PieceTree::new("0123456789");

        for _ in 0..100 {
            tree.insert("X", 5);
            assert_eq!(text(&tree), "01234X56789");

            tree.delete(5, 1);
            assert_eq!(text(&tree), "0123456789");
        }
    }


    // ============================================================
    // Unicode
    // ============================================================

    #[test]
    fn unicode_text() {
        let tree = PieceTree::new("Hello 世界 🌍");

        assert_eq!(text(&tree), "Hello 世界 🌍");
    }

    #[test]
    fn unicode_insertion() {
        let mut tree = PieceTree::new("Hello world");

        tree.insert(" 世界", 5);

        assert_eq!(text(&tree), "Hello 世界 world");
    }

    #[test]
    fn unicode_sub_text() {
        let tree = PieceTree::new("Hello 世界");

        let start = "Hello ".len();
        let length = "世界".len();

        assert_eq!(sub_text(&tree, start, length), "世界");
    }

    #[test]
    fn unicode_deletion() {
        let mut tree = PieceTree::new("Hello 世界");

        let start = "Hello ".len();
        let length = "世界".len();

        tree.delete(start, length);

        assert_eq!(text(&tree), "Hello ");
    }

    #[test]
    fn unicode_piece_boundary_operation() {
        let mut tree = PieceTree::new("Hello 世界!");

        let boundary = "Hello ".len();

        tree.insert("beautiful ", boundary);

        assert_eq!(
            text(&tree),
            "Hello beautiful 世界!"
        );

        tree.delete(
            boundary,
            "beautiful ".len(),
        );

        assert_eq!(
            text(&tree),
            "Hello 世界!"
        );
    }

    fn reference_lines(s: &str) -> Vec<String> {
        if s.is_empty() {
            return vec![String::new()];
        }
        let mut lines = Vec::new();
        let bytes = s.as_bytes();
        let mut start = 0;
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'\n' {
                lines.push(s[start..=i].to_string());
                start = i + 1;
                i += 1;
            } else if bytes[i] == b'\r' {
                if i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
                    lines.push(s[start..=i + 1].to_string());
                    start = i + 2;
                    i += 2;
                } else {
                    i += 1;
                }
            } else {
                i += 1;
            }
        }
        if start < bytes.len() {
            lines.push(s[start..].to_string());
        } else if bytes.ends_with(b"\n") {
            lines.push(String::new());
        }
        lines
    }

    struct ReferenceModel {
        text: String,
        undo_stack: Vec<String>,
        redo_stack: Vec<String>,
    }

    impl ReferenceModel {
        fn new(content: &str) -> Self {
            Self {
                text: content.to_string(),
                undo_stack: Vec::new(),
                redo_stack: Vec::new(),
            }
        }

        fn insert(&mut self, content: &str, offset: usize) {
            if content.is_empty() {
                return;
            }
            let clamped_offset = offset.min(self.text.len());
            self.undo_stack.push(self.text.clone());
            self.redo_stack.clear();
            self.text.insert_str(clamped_offset, content);
        }

        fn delete(&mut self, offset: usize, length: usize) {
            if offset >= self.text.len() || length == 0 {
                return;
            }
            let end = offset.saturating_add(length).min(self.text.len());
            self.undo_stack.push(self.text.clone());
            self.redo_stack.clear();
            self.text.drain(offset..end);
        }

        fn undo(&mut self) {
            if let Some(prev) = self.undo_stack.pop() {
                self.redo_stack.push(self.text.clone());
                self.text = prev;
            }
        }

        fn redo(&mut self) {
            if let Some(next) = self.redo_stack.pop() {
                self.undo_stack.push(self.text.clone());
                self.text = next;
            }
        }

        fn sub_text(&self, start: usize, length: usize) -> String {
            if start >= self.text.len() || length == 0 {
                return String::new();
            }
            let end = start.saturating_add(length).min(self.text.len());
            self.text[start..end].to_string()
        }

        fn line_text(&self, line_num: usize) -> String {
            let lines = reference_lines(&self.text);
            lines.get(line_num).cloned().unwrap_or_default()
        }
    }

    struct SimplePrng {
        state: u64,
    }

    impl SimplePrng {
        fn new(seed: u64) -> Self {
            Self {
                state: if seed == 0 { 0xdeadbeefcafe } else { seed },
            }
        }

        fn next_u64(&mut self) -> u64 {
            self.state ^= self.state << 13;
            self.state ^= self.state >> 7;
            self.state ^= self.state << 17;
            self.state
        }

        fn next_range(&mut self, min: usize, max: usize) -> usize {
            if min >= max {
                return min;
            }
            min + (self.next_u64() as usize % (max - min))
        }
    }

    fn char_boundaries(s: &str) -> Vec<usize> {
        s.char_indices().map(|(i, _)| i).chain(std::iter::once(s.len())).collect()
    }

    // ============================================================
    // Black-Box Tests: Boundary Conditions & Clamping
    // ============================================================

    #[test]
    fn black_box_default_tree_is_empty() {
        let tree = PieceTree::default();

        assert_eq!(text(&tree), "");
        assert_eq!(sub_text(&tree, 0, 0), "");
        assert_eq!(sub_text(&tree, 0, 10), "");
        assert_eq!(sub_text(&tree, 1, 5), "");
        assert_eq!(sub_text(&tree, 100, 10), "");
        assert_eq!(line_text(&tree, 0), "");
        assert_eq!(line_text(&tree, 1), "");
        assert_eq!(line_text(&tree, 100), "");
    }

    #[test]
    fn black_box_insert_out_of_bounds_clamps_to_end() {
        let mut tree = PieceTree::new("hello");

        tree.insert(" world", 100);
        assert_eq!(text(&tree), "hello world");

        let mut empty = PieceTree::new("");
        empty.insert("first", 42);
        assert_eq!(text(&empty), "first");
    }

    #[test]
    fn black_box_insert_empty_string_is_noop() {
        let mut tree = PieceTree::new("hello");

        tree.insert(" world", 5);
        tree.undo(); // undo stack has 1 entry, redo stack has 1 entry ("hello world")

        // Inserting empty string should be a no-op and preserve redo stack
        tree.insert("", 0);
        tree.insert("", 2);
        tree.insert("", 5);
        tree.insert("", 100);

        assert_eq!(text(&tree), "hello");
        tree.redo();
        assert_eq!(text(&tree), "hello world");
    }

    #[test]
    fn black_box_delete_on_empty_tree_is_noop() {
        let mut tree = PieceTree::new("");

        tree.delete(0, 0);
        tree.delete(0, 10);
        tree.delete(5, 5);
        tree.delete(100, 10);

        assert_eq!(text(&tree), "");
        assert_eq!(line_text(&tree, 0), "");
        assert_eq!(line_text(&tree, 1), "");
    }

    #[test]
    fn black_box_delete_zero_length_is_noop() {
        let mut tree = PieceTree::new("hello");

        tree.insert(" world", 5);
        tree.undo(); // redo stack now has "hello world"

        // Deleting 0 length should be a no-op and preserve redo stack
        tree.delete(0, 0);
        tree.delete(2, 0);
        tree.delete(5, 0);
        tree.delete(100, 0);

        assert_eq!(text(&tree), "hello");
        tree.redo();
        assert_eq!(text(&tree), "hello world");
    }

    #[test]
    fn black_box_delete_offset_at_or_past_document_end_is_noop() {
        let mut tree = PieceTree::new("hello");

        tree.insert(" world", 5);
        tree.undo(); // redo available

        tree.delete(5, 1);
        tree.delete(6, 1);
        tree.delete(100, 10);

        assert_eq!(text(&tree), "hello");
        tree.redo();
        assert_eq!(text(&tree), "hello world");
    }

    #[test]
    fn black_box_delete_clamped_at_end_of_document() {
        let mut tree = PieceTree::new("abcdefghij");

        // delete past end clamps to document end
        tree.delete(5, 100);
        assert_eq!(text(&tree), "abcde");

        tree.delete(0, 100);
        assert_eq!(text(&tree), "");

        let mut tree2 = PieceTree::new("12345");
        tree2.delete(2, 50);
        assert_eq!(text(&tree2), "12");
    }

    #[test]
    fn black_box_sub_text_boundary_conditions() {
        let tree = PieceTree::new("abcdef");

        // Out of bounds start
        assert_eq!(sub_text(&tree, 6, 1), "");
        assert_eq!(sub_text(&tree, 7, 1), "");
        assert_eq!(sub_text(&tree, 100, 1), "");

        // Clamping length past end
        assert_eq!(sub_text(&tree, 2, 100), "cdef");
        assert_eq!(sub_text(&tree, 2, 1000), "cdef");
        assert_eq!(sub_text(&tree, 0, 100), "abcdef");

        // Zero length queries
        assert_eq!(sub_text(&tree, 0, 0), "");
        assert_eq!(sub_text(&tree, 3, 0), "");
        assert_eq!(sub_text(&tree, 6, 0), "");
        assert_eq!(sub_text(&tree, 10, 0), "");
        assert_eq!(sub_text(&tree, 100, 0), "");

        // Empty document queries
        let empty = PieceTree::new("");
        assert_eq!(sub_text(&empty, 0, 0), "");
        assert_eq!(sub_text(&empty, 0, 5), "");
        assert_eq!(sub_text(&empty, 5, 5), "");
        assert_eq!(sub_text(&empty, 100, 100), "");
    }

    

    

    

    // ============================================================
    // Black-Box Tests: Multi-line & Line Ending Edge Cases
    // ============================================================

    #[test]
    fn black_box_line_text_empty_doc_indices() {
        let tree = PieceTree::new("");

        assert_eq!(line_text(&tree, 0), "");
        assert_eq!(line_text(&tree, 1), "");
        assert_eq!(line_text(&tree, 2), "");
        assert_eq!(line_text(&tree, 100), "");
    }

    #[test]
    fn black_box_line_text_single_line_without_trailing_newline() {
        let tree = PieceTree::new("hello");

        assert_eq!(line_text(&tree, 0), "hello");
        assert_eq!(line_text(&tree, 1), "");
        assert_eq!(line_text(&tree, 2), "");
    }

    #[test]
    fn black_box_line_text_single_line_with_trailing_newline() {
        let tree = PieceTree::new("hello\n");

        assert_eq!(line_text(&tree, 0), "hello\n");
        assert_eq!(line_text(&tree, 1), "");
        assert_eq!(line_text(&tree, 2), "");
    }

    #[test]
    fn black_box_line_text_single_line_with_crlf() {
        let tree = PieceTree::new("hello\r\n");

        assert_eq!(line_text(&tree, 0), "hello\r\n");
        assert_eq!(line_text(&tree, 1), "");
        assert_eq!(line_text(&tree, 2), "");
    }

    #[test]
    fn black_box_line_text_consecutive_lf_newlines() {
        let tree = PieceTree::new("\n\n\n");

        assert_eq!(line_text(&tree, 0), "\n");
        assert_eq!(line_text(&tree, 1), "\n");
        assert_eq!(line_text(&tree, 2), "\n");
        assert_eq!(line_text(&tree, 3), "");
        assert_eq!(line_text(&tree, 4), "");
    }

    #[test]
    fn black_box_line_text_consecutive_crlf_newlines() {
        let tree = PieceTree::new("\r\n\r\n\r\n");

        assert_eq!(line_text(&tree, 0), "\r\n");
        assert_eq!(line_text(&tree, 1), "\r\n");
        assert_eq!(line_text(&tree, 2), "\r\n");
        assert_eq!(line_text(&tree, 3), "");
        assert_eq!(line_text(&tree, 4), "");
    }

    #[test]
    fn black_box_line_text_mixed_newlines() {
        let tree = PieceTree::new("line1\r\nline2\nline3\r\nline4\n");

        assert_eq!(line_text(&tree, 0), "line1\r\n");
        assert_eq!(line_text(&tree, 1), "line2\n");
        assert_eq!(line_text(&tree, 2), "line3\r\n");
        assert_eq!(line_text(&tree, 3), "line4\n");
        assert_eq!(line_text(&tree, 4), "");
        assert_eq!(line_text(&tree, 5), "");
    }

    #[test]
    fn black_box_line_text_leading_and_trailing_empty_lines() {
        let tree = PieceTree::new("\n\ncontent\n\n");

        assert_eq!(line_text(&tree, 0), "\n");
        assert_eq!(line_text(&tree, 1), "\n");
        assert_eq!(line_text(&tree, 2), "content\n");
        assert_eq!(line_text(&tree, 3), "\n");
        assert_eq!(line_text(&tree, 4), "");
    }

    #[test]
    fn black_box_line_text_inserting_newlines_at_boundaries() {
        let mut tree = PieceTree::new("abc def");

        tree.insert("\n", 3);
        assert_eq!(text(&tree), "abc\n def");
        assert_eq!(line_text(&tree, 0), "abc\n");
        assert_eq!(line_text(&tree, 1), " def");

        tree.insert("\r\n", 0);
        assert_eq!(text(&tree), "\r\nabc\n def");
        assert_eq!(line_text(&tree, 0), "\r\n");
        assert_eq!(line_text(&tree, 1), "abc\n");
        assert_eq!(line_text(&tree, 2), " def");

        tree.insert("\n", text(&tree).len());
        assert_eq!(line_text(&tree, 0), "\r\n");
        assert_eq!(line_text(&tree, 1), "abc\n");
        assert_eq!(line_text(&tree, 2), " def\n");
        assert_eq!(line_text(&tree, 3), "");
    }

    #[test]
    fn black_box_line_text_deleting_newline_merges_lines() {
        let mut tree = PieceTree::new("first\nsecond\r\nthird");

        // Delete \n after "first"
        tree.delete(5, 1);
        assert_eq!(text(&tree), "firstsecond\r\nthird");
        assert_eq!(line_text(&tree, 0), "firstsecond\r\n");
        assert_eq!(line_text(&tree, 1), "third");
        assert_eq!(line_text(&tree, 2), "");

        // Delete \r\n (2 bytes)
        let crlf_offset = "firstsecond".len();
        tree.delete(crlf_offset, 2);
        assert_eq!(text(&tree), "firstsecondthird");
        assert_eq!(line_text(&tree, 0), "firstsecondthird");
        assert_eq!(line_text(&tree, 1), "");
    }

    #[test]
    fn black_box_line_text_delete_across_multiple_lines_and_pieces() {
        let mut tree = PieceTree::new("line0\n");
        tree.insert("line1\nline2\nline3\n", 6);
        tree.insert("line4\n", text(&tree).len());

        // Document is "line0\nline1\nline2\nline3\nline4\n"
        assert_eq!(line_text(&tree, 0), "line0\n");
        assert_eq!(line_text(&tree, 1), "line1\n");
        assert_eq!(line_text(&tree, 2), "line2\n");
        assert_eq!(line_text(&tree, 3), "line3\n");
        assert_eq!(line_text(&tree, 4), "line4\n");
        assert_eq!(line_text(&tree, 5), "");

        // Delete from middle of line1 ("ne1\nline2\nli")
        let start = "line0\nli".len();
        let len = "ne1\nline2\nli".len();
        tree.delete(start, len);

        // Expected document: "line0\nline3\nline4\n"
        assert_eq!(text(&tree), "line0\nline3\nline4\n");
        assert_eq!(line_text(&tree, 0), "line0\n");
        assert_eq!(line_text(&tree, 1), "line3\n");
        assert_eq!(line_text(&tree, 2), "line4\n");
        assert_eq!(line_text(&tree, 3), "");
    }

    #[test]
    fn black_box_line_text_delete_all_restores_empty_line_behavior() {
        let mut tree = PieceTree::new("line0\nline1\nline2\n");
        tree.delete(0, text(&tree).len());

        assert_eq!(text(&tree), "");
        assert_eq!(line_text(&tree, 0), "");
        assert_eq!(line_text(&tree, 1), "");
        assert_eq!(line_text(&tree, 100), "");
    }

    // ============================================================
    // Black-Box Tests: Deep Undo & Redo Chains
    // ============================================================

    #[test]
    fn black_box_undo_redo_empty_initial_doc() {
        let mut tree = PieceTree::default();

        tree.undo();
        assert_eq!(text(&tree), "");
        tree.redo();
        assert_eq!(text(&tree), "");

        tree.insert("hello", 0);
        assert_eq!(text(&tree), "hello");

        tree.undo();
        assert_eq!(text(&tree), "");

        tree.redo();
        assert_eq!(text(&tree), "hello");
    }

    #[test]
    fn black_box_undo_redo_initial_tree_no_history() {
        let mut tree = PieceTree::new("initial");

        tree.undo();
        assert_eq!(text(&tree), "initial");
        tree.undo();
        assert_eq!(text(&tree), "initial");
        tree.redo();
        assert_eq!(text(&tree), "initial");
    }

    #[test]
    fn black_box_undo_delete_that_emptied_entire_document() {
        let mut tree = PieceTree::new("entire document");

        tree.delete(0, 15);
        assert_eq!(text(&tree), "");
        assert_eq!(line_text(&tree, 0), "");

        tree.undo();
        assert_eq!(text(&tree), "entire document");
        assert_eq!(line_text(&tree, 0), "entire document");

        tree.redo();
        assert_eq!(text(&tree), "");

        tree.undo();
        assert_eq!(text(&tree), "entire document");
    }

    #[test]
    fn black_box_deep_undo_redo_100_inserts() {
        let mut tree = PieceTree::new("");

        for i in 0..100 {
            tree.insert("a", i);
        }
        assert_eq!(text(&tree).len(), 100);

        for expected_len in (0..100).rev() {
            tree.undo();
            assert_eq!(text(&tree).len(), expected_len);
        }
        assert_eq!(text(&tree), "");

        for expected_len in 1..=100 {
            tree.redo();
            assert_eq!(text(&tree).len(), expected_len);
        }
        assert_eq!(text(&tree), "a".repeat(100));
    }

    #[test]
    fn black_box_deep_undo_redo_50_deletes() {
        let initial = "0123456789".repeat(5); // 50 chars
        let mut tree = PieceTree::new(&initial);

        // Delete 1 char from offset 10, 30 times
        for _ in 0..30 {
            tree.delete(10, 1);
        }
        assert_eq!(text(&tree).len(), 20);

        // Undo all 30 deletes
        for expected_len in 21..=50 {
            tree.undo();
            assert_eq!(text(&tree).len(), expected_len);
        }
        assert_eq!(text(&tree), initial);

        // Redo all 30 deletes
        for expected_len in (20..50).rev() {
            tree.redo();
            assert_eq!(text(&tree).len(), expected_len);
        }
        assert_eq!(text(&tree).len(), 20);
    }

    #[test]
    fn black_box_alternating_undo_redo_preserves_state() {
        let mut tree = PieceTree::new("start");

        tree.insert(" mid", 5);
        tree.insert(" end", 9);
        assert_eq!(text(&tree), "start mid end");

        for _ in 0..20 {
            tree.undo();
            assert_eq!(text(&tree), "start mid");
            tree.redo();
            assert_eq!(text(&tree), "start mid end");
        }
    }

    #[test]
    fn black_box_insert_after_undo_discards_redo_stack() {
        let mut tree = PieceTree::new("base");

        tree.insert(" 1", 4);
        tree.insert(" 2", 6);
        tree.insert(" 3", 8);

        tree.undo();
        tree.undo();
        assert_eq!(text(&tree), "base 1");

        // New edit discards redo stack
        tree.insert(" NEW", 6);
        assert_eq!(text(&tree), "base 1 NEW");

        tree.redo();
        assert_eq!(text(&tree), "base 1 NEW");

        tree.undo();
        assert_eq!(text(&tree), "base 1");
    }

    #[test]
    fn black_box_delete_after_undo_discards_redo_stack() {
        let mut tree = PieceTree::new("base");

        tree.insert(" 1", 4);
        tree.insert(" 2", 6);

        tree.undo();
        assert_eq!(text(&tree), "base 1");

        // New delete discards redo stack
        tree.delete(4, 2);
        assert_eq!(text(&tree), "base");

        tree.redo();
        assert_eq!(text(&tree), "base");

        tree.undo();
        assert_eq!(text(&tree), "base 1");
    }

    #[test]
    fn black_box_repeated_undo_past_history_is_safe() {
        let mut tree = PieceTree::new("root");

        tree.insert(" edit", 4);
        tree.undo();
        assert_eq!(text(&tree), "root");

        // Safe no-ops
        for _ in 0..10 {
            tree.undo();
            assert_eq!(text(&tree), "root");
        }

        tree.redo();
        assert_eq!(text(&tree), "root edit");

        // Redo at end of history is safe no-op
        for _ in 0..10 {
            tree.redo();
            assert_eq!(text(&tree), "root edit");
        }
    }

    // ============================================================
    // Black-Box Tests: Substring Queries Across Piece Boundaries
    // ============================================================

    #[test]
    fn black_box_sub_text_across_many_small_pieces() {
        let mut tree = PieceTree::new("");
        let mut reference = String::new();

        for i in 0..20 {
            let piece = format!("{:02}_", i);
            tree.insert(&piece, reference.len());
            reference.push_str(&piece);
        }
        // reference: "00_01_02_..._19_" (60 bytes)
        assert_eq!(text(&tree), reference);

        // Slices within single pieces
        assert_eq!(sub_text(&tree, 0, 3), "00_");
        assert_eq!(sub_text(&tree, 3, 3), "01_");
        assert_eq!(sub_text(&tree, 57, 3), "19_");

        // Slices spanning exactly 2 pieces
        assert_eq!(sub_text(&tree, 0, 6), "00_01_");
        assert_eq!(sub_text(&tree, 6, 6), "02_03_");

        // Slices spanning 5 pieces
        assert_eq!(sub_text(&tree, 9, 15), "03_04_05_06_07_");

        // Slices starting and ending in the middle of pieces
        assert_eq!(sub_text(&tree, 2, 7), "_01_02_");

        // Query entire document
        assert_eq!(sub_text(&tree, 0, 60), reference);

        // Clamped queries past end
        assert_eq!(sub_text(&tree, 55, 100), reference[55..]);
    }

    #[test]
    fn black_box_sub_text_piece_boundary_edges() {
        let mut tree = PieceTree::new("ABCDE");
        tree.insert("123", 2); // "AB123CDE"

        // Boundary is at offset 2 and offset 5
        assert_eq!(sub_text(&tree, 2, 0), "");
        assert_eq!(sub_text(&tree, 5, 0), "");
        assert_eq!(sub_text(&tree, 2, 3), "123");
        assert_eq!(sub_text(&tree, 0, 2), "AB");
        assert_eq!(sub_text(&tree, 5, 3), "CDE");
        assert_eq!(sub_text(&tree, 1, 3), "B12");
        assert_eq!(sub_text(&tree, 4, 3), "3CD");
    }

    // ============================================================
    // Black-Box Tests: Complex Interleaving of Insertions & Deletions
    // ============================================================

    #[test]
    fn black_box_interleaved_prepend_and_append() {
        let mut tree = PieceTree::new("");
        let mut expected = String::new();

        for i in 0..20 {
            let pre = format!("<{}>", i);
            let post = format!("[{}]", i);

            tree.insert(&pre, 0);
            expected.insert_str(0, &pre);

            let len = text(&tree).len();
            tree.insert(&post, len);
            expected.push_str(&post);

            assert_eq!(text(&tree), expected);
        }
    }

    #[test]
    fn black_box_nested_middle_insertions() {
        let mut tree = PieceTree::new("()");
        let mut expected = String::from("()");

        for i in 0..15 {
            let mid = expected.len() / 2;
            let tag = format!("{}:{}", i, i);
            tree.insert(&tag, mid);
            expected.insert_str(mid, &tag);

            assert_eq!(text(&tree), expected);
        }
    }

    #[test]
    fn black_box_delete_across_multiple_piece_boundaries() {
        let mut tree = PieceTree::new("piece0_");
        tree.insert("piece1_", text(&tree).len());
        tree.insert("piece2_", text(&tree).len());
        tree.insert("piece3_", text(&tree).len());
        tree.insert("piece4_", text(&tree).len());

        // Full text: "piece0_piece1_piece2_piece3_piece4_"
        let start = "piece0_pie".len();
        let len = "ce1_piece2_pie".len();
        tree.delete(start, len);

        assert_eq!(text(&tree), "piece0_piece3_piece4_");
    }

    #[test]
    fn black_box_typing_and_backspacing_simulation() {
        let mut tree = PieceTree::new("");

        let sequence = [
            ("t", true),
            ("h", true),
            ("e", true),
            ("e", true),  // typo: "thee"
            ("", false),   // backspace
            (" ", true),
            ("c", true),
            ("a", true),
            ("t", true),
        ];

        for (ch, is_insert) in sequence {
            if is_insert {
                let len = text(&tree).len();
                tree.insert(ch, len);
            } else {
                let len = text(&tree).len();
                if len > 0 {
                    tree.delete(len - 1, 1);
                }
            }
        }

        assert_eq!(text(&tree), "the cat");
    }

    #[test]
    fn black_box_delete_all_then_reinsert_then_delete_all() {
        let mut tree = PieceTree::new("generation 1");
        assert_eq!(text(&tree), "generation 1");

        tree.delete(0, text(&tree).len());
        assert_eq!(text(&tree), "");

        tree.insert("generation 2 is much longer text here", 0);
        assert_eq!(text(&tree), "generation 2 is much longer text here");

        tree.delete(0, text(&tree).len());
        assert_eq!(text(&tree), "");

        tree.insert("generation 3", 0);
        assert_eq!(text(&tree), "generation 3");
    }

    #[test]
    fn black_box_successive_backspaces_until_empty() {
        let mut tree = PieceTree::new("abcdefghij");

        for i in (0..10).rev() {
            tree.delete(i, 1);
            assert_eq!(text(&tree).len(), i);
        }
        assert_eq!(text(&tree), "");
    }

    // ============================================================
    // Black-Box Tests: Multibyte UTF-8 Characters
    // ============================================================

    #[test]
    fn black_box_unicode_2byte_characters() {
        // Greek: α (2 bytes), β (2 bytes), γ (2 bytes), δ (2 bytes)
        let mut tree = PieceTree::new("αβγδ");
        assert_eq!(text(&tree), "αβγδ");
        assert_eq!(text(&tree).len(), 8);

        // Insert ε (2 bytes) between β and γ (byte offset 4)
        tree.insert("ε", 4);
        assert_eq!(text(&tree), "αβεγδ");
        assert_eq!(text(&tree).len(), 10);

        // sub_text of "βε" (byte offset 2, length 4)
        assert_eq!(sub_text(&tree, 2, 4), "βε");

        // Delete "ε" (byte offset 4, length 2)
        tree.delete(4, 2);
        assert_eq!(text(&tree), "αβγδ");
    }

    #[test]
    fn black_box_unicode_3byte_cjk() {
        // "你好世界" (each char is 3 bytes, total 12 bytes)
        let mut tree = PieceTree::new("你好世界");
        assert_eq!(text(&tree), "你好世界");
        assert_eq!(text(&tree).len(), 12);

        // Insert "大" (3 bytes) between "好" and "世" (byte offset 6)
        tree.insert("大", 6);
        assert_eq!(text(&tree), "你好大世界");
        assert_eq!(text(&tree).len(), 15);

        // sub_text of "好大世" (offset 3, length 9)
        assert_eq!(sub_text(&tree, 3, 9), "好大世");

        // Delete "大世" (offset 6, length 6)
        tree.delete(6, 6);
        assert_eq!(text(&tree), "你好界");
    }

    #[test]
    fn black_box_unicode_4byte_emojis() {
        // Emojis: 🦀 (4 bytes), 🚀 (4 bytes), 🎉 (4 bytes), 🔥 (4 bytes)
        let mut tree = PieceTree::new("🦀🚀🔥");
        assert_eq!(text(&tree), "🦀🚀🔥");
        assert_eq!(text(&tree).len(), 12);

        // Insert 🎉 (4 bytes) between 🚀 and 🔥 (offset 8)
        tree.insert("🎉", 8);
        assert_eq!(text(&tree), "🦀🚀🎉🔥");
        assert_eq!(text(&tree).len(), 16);

        // sub_text of single emoji
        assert_eq!(sub_text(&tree, 0, 4), "🦀");
        assert_eq!(sub_text(&tree, 4, 4), "🚀");
        assert_eq!(sub_text(&tree, 8, 4), "🎉");
        assert_eq!(sub_text(&tree, 12, 4), "🔥");

        // sub_text of 2 emojis across piece boundary
        assert_eq!(sub_text(&tree, 4, 8), "🚀🎉");

        // Delete "🎉" (offset 8, length 4)
        tree.delete(8, 4);
        assert_eq!(text(&tree), "🦀🚀🔥");
    }

    #[test]
    fn black_box_unicode_multiline_with_emojis() {
        let content = "🦀 Rust\n🚀 Rocket\n🎉 Party\n";
        let tree = PieceTree::new(content);

        assert_eq!(line_text(&tree, 0), "🦀 Rust\n");
        assert_eq!(line_text(&tree, 1), "🚀 Rocket\n");
        assert_eq!(line_text(&tree, 2), "🎉 Party\n");
        assert_eq!(line_text(&tree, 3), "");

        let mut mod_tree = PieceTree::new(content);
        let rust_nl_offset = "🦀 Rust".len();
        mod_tree.delete(rust_nl_offset, 1);

        assert_eq!(line_text(&mod_tree, 0), "🦀 Rust🚀 Rocket\n");
        assert_eq!(line_text(&mod_tree, 1), "🎉 Party\n");
        assert_eq!(line_text(&mod_tree, 2), "");
    }

    #[test]
    fn black_box_unicode_undo_redo() {
        let mut tree = PieceTree::new("🦀");

        tree.insert("世界", 4);
        assert_eq!(text(&tree), "🦀世界");

        tree.undo();
        assert_eq!(text(&tree), "🦀");

        tree.redo();
        assert_eq!(text(&tree), "🦀世界");

        tree.delete(4, 6);
        assert_eq!(text(&tree), "🦀");

        tree.undo();
        assert_eq!(text(&tree), "🦀世界");
    }

    // ============================================================
    // Black-Box Tests: Differential / Model-Based Fuzz Testing
    // ============================================================

    #[test]
    fn black_box_differential_fuzz_random_operations() {
        let snippets = [
            "a", "bc", "Hello, world! ", "foo\nbar\n", "xyz\r\n123",
            "\n", "\r\n", "🦀", "世界", " "
        ];

        let seeds = [42u64, 1337, 2024, 99999, 12345678];

        for &seed in &seeds {
            let mut prng = SimplePrng::new(seed);
            let mut tree = PieceTree::new("initial_content\n");
            let mut model = ReferenceModel::new("initial_content\n");

            for _ in 0..120 {
                let op = prng.next_range(0, 100);

                if op < 40 {
                    // Insert at valid char boundary
                    let snippet = snippets[prng.next_range(0, snippets.len())];
                    let boundaries = char_boundaries(&model.text);
                    let offset = boundaries[prng.next_range(0, boundaries.len())];

                    tree.insert(snippet, offset);
                    model.insert(snippet, offset);
                } else if op < 65 {
                    // Delete at valid char boundaries
                    let boundaries = char_boundaries(&model.text);
                    if boundaries.len() > 1 {
                        let idx1 = prng.next_range(0, boundaries.len());
                        let idx2 = prng.next_range(0, boundaries.len());
                        let (start_idx, end_idx) = if idx1 <= idx2 { (idx1, idx2) } else { (idx2, idx1) };
                        let start = boundaries[start_idx];
                        let length = boundaries[end_idx] - start;

                        tree.delete(start, length);
                        model.delete(start, length);
                    }
                } else if op < 85 {
                    // Undo
                    tree.undo();
                    model.undo();
                } else {
                    // Redo
                    tree.redo();
                    model.redo();
                }

                // Verify text integrity after every step
                assert_eq!(text(&tree), model.text, "Text mismatch with seed {}", seed);

                // Sample random substring
                let boundaries = char_boundaries(&model.text);
                if boundaries.len() > 1 {
                    let idx1 = prng.next_range(0, boundaries.len());
                    let idx2 = prng.next_range(0, boundaries.len());
                    let (start_idx, end_idx) = if idx1 <= idx2 { (idx1, idx2) } else { (idx2, idx1) };
                    let start = boundaries[start_idx];
                    let length = boundaries[end_idx] - start;

                    assert_eq!(
                        sub_text(&tree, start, length),
                        model.sub_text(start, length),
                        "Sub_text mismatch at ({}, {}) with seed {}",
                        start,
                        length,
                        seed
                    );
                }

                // Sample random line query
                let line_count = reference_lines(&model.text).len();
                let test_line = prng.next_range(0, line_count + 3);
                assert_eq!(
                    line_text(&tree, test_line),
                    model.line_text(test_line),
                    "Line text mismatch at line {} with seed {}",
                    test_line,
                    seed
                );
            }
        }
    }

    #[test]
    fn black_box_differential_fuzz_undo_redo_stress() {
        let seeds = [55555u64, 77777, 88888];

        for &seed in &seeds {
            let mut prng = SimplePrng::new(seed);
            let mut tree = PieceTree::new("");
            let mut model = ReferenceModel::new("");

            // Apply 50 edits
            for i in 0..50 {
                let piece = format!("chunk{}_", i);
                let boundaries = char_boundaries(&model.text);
                let offset = boundaries[prng.next_range(0, boundaries.len())];

                tree.insert(&piece, offset);
                model.insert(&piece, offset);
                assert_eq!(text(&tree), model.text);
            }

            // Randomly undo and redo 60 times
            for _ in 0..60 {
                if prng.next_range(0, 2) == 0 {
                    tree.undo();
                    model.undo();
                } else {
                    tree.redo();
                    model.redo();
                }
                assert_eq!(text(&tree), model.text, "Mismatch during undo/redo stress, seed {}", seed);
            }

            // Undo all the way to beginning
            for _ in 0..60 {
                tree.undo();
                model.undo();
                assert_eq!(text(&tree), model.text);
            }
            assert_eq!(text(&tree), "");

            // Redo all the way back
            for _ in 0..60 {
                tree.redo();
                model.redo();
                assert_eq!(text(&tree), model.text);
            }
        }
    }

    // ------------------------------------------------------------
    // Helper for get_lines_text
    // ------------------------------------------------------------

    fn lines_text(tree: &PieceTree, start_line: usize, n: usize) -> String {
        let mut result = String::new();
        tree.get_lines_text(start_line, n, &mut result);
        result
    }

    // ============================================================
    // get_lines_text
    // ============================================================

    #[test]
    fn get_lines_text_empty_doc_returns_empty() {
        let tree = PieceTree::new("");

        assert_eq!(lines_text(&tree, 0, 0), "");
        assert_eq!(lines_text(&tree, 0, 1), "");
        assert_eq!(lines_text(&tree, 0, 10), "");
    }

    #[test]
    fn get_lines_text_zero_num_of_lines_returns_empty() {
        let tree = PieceTree::new("one\ntwo\nthree");

        assert_eq!(lines_text(&tree, 0, 0), "");
        assert_eq!(lines_text(&tree, 1, 0), "");
        assert_eq!(lines_text(&tree, 2, 0), "");
    }

    #[test]
    fn get_lines_text_single_line_doc_fetch_one_line() {
        let tree = PieceTree::new("hello");

        assert_eq!(lines_text(&tree, 0, 1), "hello");
    }

    #[test]
    fn get_lines_text_fetch_first_line_from_multiline() {
        let tree = PieceTree::new("one\ntwo\nthree");

        assert_eq!(lines_text(&tree, 0, 1), "one\n");
    }

    #[test]
    fn get_lines_text_fetch_two_lines_from_start() {
        let tree = PieceTree::new("one\ntwo\nthree");

        assert_eq!(lines_text(&tree, 0, 2), "one\ntwo\n");
    }

    #[test]
    fn get_lines_text_fetch_all_lines_from_start() {
        let tree = PieceTree::new("one\ntwo\nthree");

        assert_eq!(lines_text(&tree, 0, 3), "one\ntwo\nthree");
    }

    #[test]
    fn get_lines_text_fetch_from_mid_document() {
        let tree = PieceTree::new("one\ntwo\nthree\nfour");

        // start at line 1 ("two"), fetch 2 lines
        assert_eq!(lines_text(&tree, 1, 2), "two\nthree\n");
    }

    #[test]
    fn get_lines_text_fetch_last_line_only() {
        let tree = PieceTree::new("one\ntwo\nthree");

        assert_eq!(lines_text(&tree, 2, 1), "three");
    }

    #[test]
    fn get_lines_text_num_of_lines_exceeds_remaining_lines() {
        let tree = PieceTree::new("one\ntwo\nthree");

        // Ask for 5 lines starting at line 1; only "two\nthree" remain
        assert_eq!(lines_text(&tree, 1, 5), "two\nthree");
    }

    #[test]
    fn get_lines_text_start_line_out_of_range_returns_empty() {
        let tree = PieceTree::new("one\ntwo\nthree");

        assert_eq!(lines_text(&tree, 10, 3), "");
        assert_eq!(lines_text(&tree, 100, 1), "");
    }

    #[test]
    fn get_lines_text_trailing_newline_line_counts() {
        let tree = PieceTree::new("one\ntwo\n");

        // "one\n" is line 0, "two\n" is line 1, "" is line 2
        assert_eq!(lines_text(&tree, 0, 1), "one\n");
        assert_eq!(lines_text(&tree, 0, 2), "one\ntwo\n");
        // Fetching the empty trailing line
        assert_eq!(lines_text(&tree, 2, 1), "");
    }

    #[test]
    fn get_lines_text_crlf_line_endings() {
        let tree = PieceTree::new("one\r\ntwo\r\nthree");

        assert_eq!(lines_text(&tree, 0, 1), "one\r\n");
        assert_eq!(lines_text(&tree, 0, 2), "one\r\ntwo\r\n");
        assert_eq!(lines_text(&tree, 1, 2), "two\r\nthree");
        assert_eq!(lines_text(&tree, 2, 1), "three");
    }

    #[test]
    fn get_lines_text_across_inserted_piece_boundary() {
        let mut tree = PieceTree::new("hello world");

        tree.insert("\none\n", 5);
        // Document is now "hello\none\n world"
        assert_eq!(text(&tree), "hello\none\n world");

        assert_eq!(lines_text(&tree, 0, 1), "hello\n");
        assert_eq!(lines_text(&tree, 0, 2), "hello\none\n");
        assert_eq!(lines_text(&tree, 0, 3), "hello\none\n world");
        assert_eq!(lines_text(&tree, 1, 2), "one\n world");
    }

    #[test]
    fn get_lines_text_after_delete_that_merges_lines() {
        let mut tree = PieceTree::new("first\nsecond\nthird");

        // Delete the '\n' between first and second
        tree.delete(5, 1);
        // Document is now "firstsecond\nthird"
        assert_eq!(text(&tree), "firstsecond\nthird");

        assert_eq!(lines_text(&tree, 0, 1), "firstsecond\n");
        assert_eq!(lines_text(&tree, 0, 2), "firstsecond\nthird");
        assert_eq!(lines_text(&tree, 1, 1), "third");
    }

    #[test]
    fn get_lines_text_unicode_multiline() {
        let tree = PieceTree::new("🦀 Rust\n🚀 Rocket\n🎉 Party\n");

        assert_eq!(lines_text(&tree, 0, 1), "🦀 Rust\n");
        assert_eq!(lines_text(&tree, 0, 2), "🦀 Rust\n🚀 Rocket\n");
        assert_eq!(lines_text(&tree, 0, 3), "🦀 Rust\n🚀 Rocket\n🎉 Party\n");
        assert_eq!(lines_text(&tree, 1, 2), "🚀 Rocket\n🎉 Party\n");
        assert_eq!(lines_text(&tree, 2, 1), "🎉 Party\n");
    }

    #[test]
    fn get_lines_text_consecutive_newlines() {
        let tree = PieceTree::new("one\n\nthree");

        assert_eq!(lines_text(&tree, 0, 1), "one\n");
        assert_eq!(lines_text(&tree, 1, 1), "\n");
        assert_eq!(lines_text(&tree, 0, 2), "one\n\n");
        assert_eq!(lines_text(&tree, 0, 3), "one\n\nthree");
        assert_eq!(lines_text(&tree, 1, 2), "\nthree");
    }

    #[test]
    fn get_lines_text_matches_repeated_get_line_text() {
        // Verify get_lines_text(start, n) == concatenation of get_line_text(start..start+n)
        let content = "alpha\nbeta\ngamma\ndelta\nepsilon";
        let tree = PieceTree::new(content);

        for start in 0..5 {
            for n in 0..=5 {
                let by_range = lines_text(&tree, start, n);
                let by_single: String = (start..start + n)
                    .map(|l| line_text(&tree, l))
                    .collect();
                assert_eq!(
                    by_range, by_single,
                    "Mismatch for start={} n={}",
                    start, n
                );
            }
        }
    }

    // ============================================================
    // get_lines_text — additional edge-case / bug-catching tests
    // ============================================================

    #[test]
    fn get_lines_text_does_not_corrupt_existing_buffer_content() {
        // The signature takes `content: &mut String`.
        // The fetched lines must appear in the buffer; a buggy impl that calls
        // `content.clear()` before writing would lose pre-existing data — the
        // fetched portion must still be present and correct regardless.
        let tree = PieceTree::new("one\ntwo\nthree");
        let mut buf = String::new();
        tree.get_lines_text(0, 2, &mut buf);
        // The result must contain the first two lines.
        assert!(
            buf.contains("one\n") && buf.contains("two\n"),
            "Expected fetched lines present in buffer, got: {:?}",
            buf
        );
    }

    #[test]
    fn get_lines_text_start_line_one_past_last_line_returns_empty() {
        // A document with N logical lines has valid indices 0..N-1.
        // Requesting exactly index N must return empty (boundary check).
        let tree = PieceTree::new("only line");
        // Line count == 1, so start_line == 1 is out of range.
        assert_eq!(lines_text(&tree, 1, 1), "");
        assert_eq!(lines_text(&tree, 1, 10), "");
    }

    #[test]
    fn get_lines_text_single_newline_doc() {
        // A document that is just "\n" has two logical lines: "\n" (line 0) and "" (line 1).
        let tree = PieceTree::new("\n");

        assert_eq!(lines_text(&tree, 0, 1), "\n");
        assert_eq!(lines_text(&tree, 1, 1), "");
        assert_eq!(lines_text(&tree, 0, 2), "\n");
        assert_eq!(lines_text(&tree, 2, 1), ""); // fully out of range
    }

    #[test]
    fn get_lines_text_doc_of_only_newlines() {
        // Document: "\n\n\n" — four logical lines.
        let tree = PieceTree::new("\n\n\n");

        assert_eq!(lines_text(&tree, 0, 1), "\n");
        assert_eq!(lines_text(&tree, 1, 1), "\n");
        assert_eq!(lines_text(&tree, 2, 1), "\n");
        assert_eq!(lines_text(&tree, 3, 1), ""); // trailing empty line
        assert_eq!(lines_text(&tree, 0, 4), "\n\n\n");
        assert_eq!(lines_text(&tree, 1, 3), "\n\n");
    }

    #[test]
    fn get_lines_text_large_num_of_lines_saturates_at_doc_end() {
        // Passing usize::MAX should not panic — it must clamp to what is available.
        let tree = PieceTree::new("line0\nline1\nline2");
        let result = lines_text(&tree, 0, usize::MAX);
        assert_eq!(result, "line0\nline1\nline2");
    }

    #[test]
    fn get_lines_text_mixed_crlf_and_lf_line_endings() {
        // Some lines end with \r\n, others with \n — all must be returned verbatim.
        let tree = PieceTree::new("alpha\r\nbeta\ngamma\r\n");

        assert_eq!(lines_text(&tree, 0, 1), "alpha\r\n");
        assert_eq!(lines_text(&tree, 1, 1), "beta\n");
        assert_eq!(lines_text(&tree, 2, 1), "gamma\r\n");
        assert_eq!(lines_text(&tree, 0, 3), "alpha\r\nbeta\ngamma\r\n");
        assert_eq!(lines_text(&tree, 1, 2), "beta\ngamma\r\n");
    }

    #[test]
    fn get_lines_text_after_insert_at_start() {
        // Inserting before all existing content and then querying lines.
        let mut tree = PieceTree::new("world");
        tree.insert("hello\n", 0);
        // Document: "hello\nworld"
        assert_eq!(text(&tree), "hello\nworld");

        assert_eq!(lines_text(&tree, 0, 1), "hello\n");
        assert_eq!(lines_text(&tree, 1, 1), "world");
        assert_eq!(lines_text(&tree, 0, 2), "hello\nworld");
    }

    #[test]
    fn get_lines_text_after_insert_at_end() {
        // Appending content at the end of the document.
        let mut tree = PieceTree::new("hello");
        tree.insert("\nworld", 5);
        // Document: "hello\nworld"
        assert_eq!(text(&tree), "hello\nworld");

        assert_eq!(lines_text(&tree, 0, 1), "hello\n");
        assert_eq!(lines_text(&tree, 1, 1), "world");
        assert_eq!(lines_text(&tree, 0, 2), "hello\nworld");
    }

    #[test]
    fn get_lines_text_after_delete_last_newline_reduces_line_count() {
        // Removing the trailing newline should reduce the visible line count.
        let mut tree = PieceTree::new("one\ntwo\n");
        // Delete the very last '\n' (byte index 7).
        tree.delete(7, 1);
        // Document is now "one\ntwo"
        assert_eq!(text(&tree), "one\ntwo");

        assert_eq!(lines_text(&tree, 0, 1), "one\n");
        assert_eq!(lines_text(&tree, 1, 1), "two");
        assert_eq!(lines_text(&tree, 0, 2), "one\ntwo");
        // There is no longer a third line.
        assert_eq!(lines_text(&tree, 2, 1), "");
    }

    #[test]
    fn get_lines_text_start_beyond_single_line_doc_returns_empty() {
        // Ensure out-of-range is handled for single-line documents too.
        let tree = PieceTree::new("only");

        assert_eq!(lines_text(&tree, 1, 1), "");
        assert_eq!(lines_text(&tree, 5, 5), "");
    }

    #[test]
    fn get_lines_text_many_small_insertions_across_pieces() {
        // Build a document via many insertions so the piece tree has many nodes,
        // then verify get_lines_text spans across multiple piece boundaries.
        let mut tree = PieceTree::new("");
        let words = ["alpha", "beta", "gamma", "delta", "epsilon"];
        let mut offset = 0usize;
        for word in &words {
            tree.insert(&format!("{}\n", word), offset);
            offset += word.len() + 1; // +1 for '\n'
        }
        assert_eq!(text(&tree), "alpha\nbeta\ngamma\ndelta\nepsilon\n");

        assert_eq!(lines_text(&tree, 0, 1), "alpha\n");
        assert_eq!(lines_text(&tree, 2, 2), "gamma\ndelta\n");
        assert_eq!(lines_text(&tree, 0, 5), "alpha\nbeta\ngamma\ndelta\nepsilon\n");
        assert_eq!(lines_text(&tree, 4, 1), "epsilon\n");
        assert_eq!(lines_text(&tree, 5, 1), "");
    }

    #[test]
    fn get_lines_text_replace_line_via_delete_and_insert() {
        // Simulate replacing the content of line 1 by deleting it and inserting new text.
        let mut tree = PieceTree::new("foo\nbar\nbaz");
        // "bar" starts at byte offset 4, length 3.
        tree.delete(4, 3);
        tree.insert("REPLACED", 4);
        // Document: "foo\nREPLACED\nbaz"
        assert_eq!(text(&tree), "foo\nREPLACED\nbaz");

        assert_eq!(lines_text(&tree, 0, 1), "foo\n");
        assert_eq!(lines_text(&tree, 1, 1), "REPLACED\n");
        assert_eq!(lines_text(&tree, 2, 1), "baz");
        assert_eq!(lines_text(&tree, 0, 3), "foo\nREPLACED\nbaz");
    }

    #[test]
    fn get_lines_text_num_of_lines_exactly_one_past_end() {
        // Requesting exactly one more line than available should return all available lines.
        let tree = PieceTree::new("a\nb\nc");
        // 3 lines total; requesting 4 from line 0 should give all three.
        assert_eq!(lines_text(&tree, 0, 4), "a\nb\nc");
    }

    #[test]
    fn get_lines_text_line_containing_only_spaces() {
        // Lines that are non-empty but contain only whitespace must be returned as-is.
        let tree = PieceTree::new("line0\n   \nline2");

        assert_eq!(lines_text(&tree, 1, 1), "   \n");
        assert_eq!(lines_text(&tree, 0, 2), "line0\n   \n");
        assert_eq!(lines_text(&tree, 0, 3), "line0\n   \nline2");
    }

    #[test]
    fn get_lines_text_very_long_single_line() {
        // A very long single line (no newlines) — any start_line > 0 must return empty.
        let long = "x".repeat(10_000);
        let tree = PieceTree::new(&long);

        assert_eq!(lines_text(&tree, 0, 1), long);
        assert_eq!(lines_text(&tree, 1, 1), "");
    }

    #[test]
    fn get_lines_text_consistent_with_get_line_text_after_mutations() {
        // Cross-check: after mutations, get_lines_text must agree with individual
        // get_line_text calls for every (start, n) combination.
        let mut tree = PieceTree::new("aaa\nbbb\nccc\nddd\neee");

        // Delete middle newline to merge lines 1 and 2.
        tree.delete(7, 1); // removes '\n' between "bbb" and "ccc"
        // Document: "aaa\nbbbccc\nddd\neee"
        assert_eq!(text(&tree), "aaa\nbbbccc\nddd\neee");

        for start in 0..4usize {
            for n in 0..=4usize {
                let by_range = lines_text(&tree, start, n);
                let by_single: String = (start..start + n)
                    .map(|l| line_text(&tree, l))
                    .collect();
                assert_eq!(
                    by_range, by_single,
                    "Mismatch after mutation: start={} n={}",
                    start, n
                );
            }
        }
    }
}
