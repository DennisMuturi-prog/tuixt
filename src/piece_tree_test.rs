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

    fn lines_text(tree: &PieceTree, start_line: usize, n: usize) -> String {
        let mut result = String::new();
        tree.get_lines_text(start_line, n, &mut result);
        result
    }

    fn line_start_offset(tree: &PieceTree, line_number: usize) -> usize {
        tree.get_start_offset_of_a_line(line_number)
    }

    // ------------------------------------------------------------
    // Reference model + PRNG (for differential / fuzz tests)
    // ------------------------------------------------------------

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

    /// Byte offset where `line_num` starts in `s`, computed from the document
    /// text alone (independent of the piece tree). Out-of-range line numbers
    /// map to the end of the document.
    fn reference_line_start_offset(s: &str, line_num: usize) -> usize {
        let lines = reference_lines(s);
        if line_num >= lines.len() {
            return s.len();
        }
        lines[..line_num].iter().map(|l| l.len()).sum()
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
        s.char_indices()
            .map(|(i, _)| i)
            .chain(std::iter::once(s.len()))
            .collect()
    }

    // ------------------------------------------------------------
    // Regression edit helper
    //
    // These sequences were minimised from randomized runs against a revision
    // of the piece tree whose red-black invariants could be broken by a
    // mid-piece insert followed by a delete. The break surfaces as a panic
    // (an `.unwrap()` on a black leaf, or `Color::minus_black` rejecting a
    // negative-black marker), which is exactly what `apply_edits` catches.
    // ------------------------------------------------------------

    #[derive(Debug)]
    enum Edit {
        Ins(&'static str, usize),
        Del(usize, usize),
    }

    fn apply_edits(initial: &str, edits: &[Edit]) -> String {
        let mut pt = PieceTree::new(initial);
        let mut reference = String::from(initial);
        for edit in edits {
            match *edit {
                Edit::Ins(content, at) => {
                    pt.insert(content, at);
                    reference.insert_str(at, content);
                }
                Edit::Del(at, len) => {
                    pt.delete(at, len);
                    reference.replace_range(at..at + len, "");
                }
            }
        }
        assert_eq!(reference, text(&pt), "document text diverged");
        reference
    }

    // ============================================================
    // Construction
    // ============================================================

    #[test]
    fn new_tree_preserves_content() {
        assert_eq!(text(&PieceTree::new("")), "");
        assert_eq!(text(&PieceTree::new("Hello, world!")), "Hello, world!");
        assert_eq!(text(&PieceTree::new("one\ntwo\nthree")), "one\ntwo\nthree");
        assert_eq!(text(&PieceTree::new("one\ntwo\n")), "one\ntwo\n");
        assert_eq!(text(&PieceTree::default()), "");
    }

    // ============================================================
    // Insertion & piece boundaries
    // ============================================================

    #[test]
    fn insert_into_empty_tree() {
        let mut tree = PieceTree::new("");
        tree.insert("hello", 0);
        assert_eq!(text(&tree), "hello");

        // An out-of-bounds offset clamps to the end of an empty tree.
        let mut empty = PieceTree::new("");
        empty.insert("first", 42);
        assert_eq!(text(&empty), "first");
    }

    #[test]
    fn insert_at_beginning_middle_and_end() {
        let mut at_start = PieceTree::new("world");
        at_start.insert("hello ", 0);
        assert_eq!(text(&at_start), "hello world");

        let mut in_middle = PieceTree::new("helo");
        in_middle.insert("l", 3);
        assert_eq!(text(&in_middle), "hello");

        let mut at_end = PieceTree::new("hello");
        at_end.insert(" world", 5);
        assert_eq!(text(&at_end), "hello world");

        // An out-of-bounds offset clamps to the end.
        let mut clamped = PieceTree::new("hello");
        clamped.insert(" world", 100);
        assert_eq!(text(&clamped), "hello world");
    }

    #[test]
    fn insert_empty_string_is_noop() {
        let mut tree = PieceTree::new("hello");
        tree.insert(" world", 5);
        tree.undo();
        assert_eq!(text(&tree), "hello");

        // Inserting empty strings is a no-op and must preserve the redo stack.
        tree.insert("", 0);
        tree.insert("", 2);
        tree.insert("", 5);
        tree.insert("", 100);

        assert_eq!(text(&tree), "hello");
        tree.redo();
        assert_eq!(text(&tree), "hello world");
    }

    #[test]
    fn insert_in_middle_creates_piece_boundary() {
        let mut tree = PieceTree::new("abcdefghij");

        // Inserting inside a piece splits it and creates adjacent pieces.
        tree.insert("XXX", 5);
        assert_eq!(text(&tree), "abcdeXXXfghij");

        // Inserting again at the same boundary stacks the new piece first.
        tree.insert("YYY", 5);
        assert_eq!(text(&tree), "abcdeYYYXXXfghij");
    }

    #[test]
    fn repeated_insertions_create_many_pieces() {
        let mut tree = PieceTree::new("0123456789");
        tree.insert("A", 5);
        tree.insert("B", 5);
        tree.insert("C", 5);
        tree.insert("D", 5);
        tree.insert("E", 5);
        assert_eq!(text(&tree), "01234EDCBA56789");

        let mut adjacent = PieceTree::new("abcdefghij");
        adjacent.insert("A", 5);
        adjacent.insert("B", 6);
        adjacent.insert("C", 7);
        adjacent.insert("D", 8);
        adjacent.insert("E", 9);
        assert_eq!(text(&adjacent), "abcdeABCDEfghij");
    }

    #[test]
    fn insert_newline_splits_line() {
        let mut tree = PieceTree::new("hello world");
        tree.insert("\n", 5);
        assert_eq!(text(&tree), "hello\n world");
        assert_eq!(line_text(&tree, 0), "hello\n");
        assert_eq!(line_text(&tree, 1), " world");
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
    // get_sub_text
    // ============================================================

    #[test]
    fn sub_text_basic() {
        let tree = PieceTree::new("abcdefghij");
        assert_eq!(sub_text(&tree, 2, 4), "cdef");
        assert_eq!(sub_text(&tree, 5, 0), "");
        assert_eq!(sub_text(&tree, 0, 10), "abcdefghij");
    }

    #[test]
    fn sub_text_across_piece_boundaries() {
        let mut tree = PieceTree::new("abcdefghij");
        tree.insert("XXX", 5);
        // abcdeXXXfghij
        assert_eq!(sub_text(&tree, 3, 6), "deXXXf");
        assert_eq!(sub_text(&tree, 6, 5), "XXfgh");
        assert_eq!(sub_text(&tree, 5, 3), "XXX");
        assert_eq!(sub_text(&tree, 8, 3), "fgh");
    }

    #[test]
    fn sub_text_out_of_bounds_clamps() {
        let tree = PieceTree::new("abcdef");
        assert_eq!(sub_text(&tree, 6, 1), "");
        assert_eq!(sub_text(&tree, 100, 1), "");
        assert_eq!(sub_text(&tree, 2, 100), "cdef");
        assert_eq!(sub_text(&tree, 0, 100), "abcdef");
        assert_eq!(sub_text(&tree, 3, 0), "");
        assert_eq!(sub_text(&tree, 100, 0), "");

        let empty = PieceTree::new("");
        assert_eq!(sub_text(&empty, 0, 5), "");
        assert_eq!(sub_text(&empty, 100, 100), "");
    }

    #[test]
    fn sub_text_huge_length_does_not_overflow() {
        let tree = PieceTree::new("abcdef");
        assert_eq!(sub_text(&tree, 0, usize::MAX), "abcdef");
        assert_eq!(sub_text(&tree, 2, usize::MAX), "cdef");
        assert_eq!(sub_text(&tree, 6, usize::MAX), "");
    }

    #[test]
    fn sub_text_huge_length_across_pieces_does_not_overflow() {
        let mut tree = PieceTree::new("abcdefghij");
        tree.insert("XXX", 5);
        // abcdeXXXfghij
        assert_eq!(sub_text(&tree, 0, usize::MAX), "abcdeXXXfghij");
        assert_eq!(sub_text(&tree, 3, usize::MAX), "deXXXfghij");
        assert_eq!(sub_text(&tree, 5, usize::MAX), "XXXfghij");
    }

    // ============================================================
    // Deletion
    // ============================================================

    #[test]
    fn delete_entire_text() {
        let mut tree = PieceTree::new("hello");
        tree.delete(0, 5);
        assert_eq!(text(&tree), "");
        tree.insert("rebuilt", 0);
        assert_eq!(text(&tree), "rebuilt");
    }

    #[test]
    fn delete_from_beginning_middle_and_end() {
        let mut from_start = PieceTree::new("hello world");
        from_start.delete(0, 6);
        assert_eq!(text(&from_start), "world");

        let mut from_end = PieceTree::new("hello world");
        from_end.delete(5, 6);
        assert_eq!(text(&from_end), "hello");

        let mut from_middle = PieceTree::new("hello world");
        from_middle.delete(5, 1);
        assert_eq!(text(&from_middle), "helloworld");
    }

    #[test]
    fn delete_zero_length_is_noop() {
        let mut tree = PieceTree::new("hello");
        tree.insert(" world", 5);
        tree.undo();
        assert_eq!(text(&tree), "hello");

        // Deleting 0 length is a no-op and must preserve the redo stack.
        tree.delete(0, 0);
        tree.delete(2, 0);
        tree.delete(5, 0);
        tree.delete(100, 0);

        assert_eq!(text(&tree), "hello");
        tree.redo();
        assert_eq!(text(&tree), "hello world");
    }

    #[test]
    fn delete_out_of_bounds_clamps_and_noop() {
        // A length past the end clamps to the document end.
        let mut tree = PieceTree::new("abcdefghij");
        tree.delete(5, 100);
        assert_eq!(text(&tree), "abcde");
        tree.delete(0, 100);
        assert_eq!(text(&tree), "");

        // An offset at or past the end is a no-op and preserves redo.
        let mut tree2 = PieceTree::new("hello");
        tree2.insert(" world", 5);
        tree2.undo();
        tree2.delete(5, 1);
        tree2.delete(6, 1);
        tree2.delete(100, 10);
        assert_eq!(text(&tree2), "hello");
        tree2.redo();
        assert_eq!(text(&tree2), "hello world");

        // Deleting from an empty tree is a no-op.
        let mut empty = PieceTree::new("");
        empty.delete(0, 0);
        empty.delete(0, 10);
        empty.delete(5, 5);
        empty.delete(100, 10);
        assert_eq!(text(&empty), "");
        assert_eq!(line_text(&empty, 0), "");
    }

    #[test]
    fn delete_middle_of_original_and_inserted_piece() {
        let mut tree = PieceTree::new("abcdefghij");
        tree.delete(3, 3);
        assert_eq!(text(&tree), "abcghij");

        let mut inserted = PieceTree::new("abcdefghij");
        inserted.insert("12345", 5);
        assert_eq!(text(&inserted), "abcde12345fghij");
        inserted.delete(7, 2);
        assert_eq!(text(&inserted), "abcde125fghij");

        let mut whole = PieceTree::new("abcdefghij");
        whole.insert("12345", 5);
        whole.delete(5, 5);
        assert_eq!(text(&whole), "abcdefghij");
    }

    #[test]
    fn delete_across_piece_boundaries() {
        // From the original piece into the inserted piece.
        let mut a = PieceTree::new("abcdefghij");
        a.insert("XXX", 5);
        a.delete(3, 5);
        assert_eq!(text(&a), "abcfghij");

        // From the inserted piece into the original piece.
        let mut b = PieceTree::new("abcdefghij");
        b.insert("XXX", 5);
        b.delete(7, 4);
        assert_eq!(text(&b), "abcdeXXij");

        // Exactly across a piece boundary (entire inserted piece + original).
        let mut c = PieceTree::new("abcdefghij");
        c.insert("XXX", 5);
        c.delete(5, 5);
        assert_eq!(text(&c), "abcdehij");
    }

    #[test]
    fn delete_across_three_pieces() {
        let mut tree = PieceTree::new("abcdefghij");
        tree.insert("111", 2);
        tree.insert("222", 6);
        assert_eq!(text(&tree), "ab111c222defghij");
        tree.delete(2, 8);
        assert_eq!(text(&tree), "abefghij");
    }

    // ============================================================
    // Line text & line endings
    // ============================================================

    #[test]
    fn line_text_basic() {
        let tree = PieceTree::new("first\nsecond\nthird");
        assert_eq!(line_text(&tree, 0), "first\n");
        assert_eq!(line_text(&tree, 1), "second\n");
        assert_eq!(line_text(&tree, 2), "third");
    }

    #[test]
    fn line_text_empty_document() {
        let tree = PieceTree::new("");
        assert_eq!(line_text(&tree, 0), "");
        assert_eq!(line_text(&tree, 1), "");
        assert_eq!(line_text(&tree, 100), "");
    }

    #[test]
    fn line_text_trailing_and_consecutive_newlines() {
        let trailing = PieceTree::new("hello\n");
        assert_eq!(line_text(&trailing, 0), "hello\n");
        assert_eq!(line_text(&trailing, 1), "");

        let consecutive = PieceTree::new("one\n\nthree");
        assert_eq!(line_text(&consecutive, 0), "one\n");
        assert_eq!(line_text(&consecutive, 1), "\n");
        assert_eq!(line_text(&consecutive, 2), "three");

        let only_newlines = PieceTree::new("\n\n\n");
        assert_eq!(line_text(&only_newlines, 0), "\n");
        assert_eq!(line_text(&only_newlines, 1), "\n");
        assert_eq!(line_text(&only_newlines, 2), "\n");
        assert_eq!(line_text(&only_newlines, 3), "");
    }

    #[test]
    fn line_text_crlf_and_mixed_newlines() {
        let crlf = PieceTree::new("one\r\ntwo\r\nthree");
        assert_eq!(line_text(&crlf, 0), "one\r\n");
        assert_eq!(line_text(&crlf, 1), "two\r\n");
        assert_eq!(line_text(&crlf, 2), "three");

        let mixed = PieceTree::new("line1\r\nline2\nline3\r\nline4\n");
        assert_eq!(line_text(&mixed, 0), "line1\r\n");
        assert_eq!(line_text(&mixed, 1), "line2\n");
        assert_eq!(line_text(&mixed, 2), "line3\r\n");
        assert_eq!(line_text(&mixed, 3), "line4\n");
        assert_eq!(line_text(&mixed, 4), "");
    }

    #[test]
    fn line_text_newline_in_inserted_piece() {
        let mut tree = PieceTree::new("hello world");
        tree.insert("\none\n", 5);
        assert_eq!(text(&tree), "hello\none\n world");
        assert_eq!(line_text(&tree, 0), "hello\n");
        assert_eq!(line_text(&tree, 1), "one\n");
        assert_eq!(line_text(&tree, 2), " world");
    }

    #[test]
    fn line_text_delete_newline_merges_lines() {
        let mut tree = PieceTree::new("hello");
        tree.insert("\n", 5);
        tree.insert("world", 6);
        assert_eq!(text(&tree), "hello\nworld");
        tree.delete(5, 1);
        assert_eq!(text(&tree), "helloworld");
        assert_eq!(line_text(&tree, 0), "helloworld");
    }

    // ============================================================
    // get_lines_text
    // ============================================================

    #[test]
    fn get_lines_text_basic() {
        let tree = PieceTree::new("one\ntwo\nthree");
        assert_eq!(lines_text(&tree, 0, 0), "");
        assert_eq!(lines_text(&tree, 0, 1), "one\n");
        assert_eq!(lines_text(&tree, 0, 2), "one\ntwo\n");
        assert_eq!(lines_text(&tree, 0, 3), "one\ntwo\nthree");
        assert_eq!(lines_text(&tree, 1, 2), "two\nthree");
        assert_eq!(lines_text(&tree, 2, 1), "three");
        // A count past the remaining lines clamps to what is available.
        assert_eq!(lines_text(&tree, 1, 5), "two\nthree");
        // A start line out of range yields empty.
        assert_eq!(lines_text(&tree, 10, 3), "");
    }

    #[test]
    fn get_lines_text_clamps_and_out_of_range() {
        let tree = PieceTree::new("line0\nline1\nline2");
        assert_eq!(lines_text(&tree, 0, usize::MAX), "line0\nline1\nline2");
        assert_eq!(lines_text(&tree, 3, 1), "");
        assert_eq!(lines_text(&tree, 5, 5), "");

        let empty = PieceTree::new("");
        assert_eq!(lines_text(&empty, 0, 0), "");
        assert_eq!(lines_text(&empty, 0, 1), "");
        assert_eq!(lines_text(&empty, 0, 10), "");

        let single_newline = PieceTree::new("\n");
        assert_eq!(lines_text(&single_newline, 0, 1), "\n");
        assert_eq!(lines_text(&single_newline, 1, 1), "");
        assert_eq!(lines_text(&single_newline, 0, 2), "\n");
    }

    #[test]
    fn get_lines_text_huge_count_from_nonzero_start() {
        // Regression: a huge count from a non-zero start line used to overflow
        // `start_line_num + num_of_lines` and panic.
        let tree = PieceTree::new("line0\nline1\nline2");
        assert_eq!(lines_text(&tree, 1, usize::MAX), "line1\nline2");
        assert_eq!(lines_text(&tree, 2, usize::MAX), "line2");
        assert_eq!(lines_text(&tree, 0, usize::MAX), "line0\nline1\nline2");

        // Trailing / unterminated last lines clamp the same way.
        let trailing = PieceTree::new("a\nb\n");
        assert_eq!(lines_text(&trailing, 1, usize::MAX), "b\n");
        assert_eq!(lines_text(&trailing, 2, usize::MAX), "");

        let unterminated = PieceTree::new("a\nb");
        assert_eq!(lines_text(&unterminated, 1, usize::MAX), "b");
    }

    #[test]
    fn get_lines_text_consistent_with_get_line_text() {
        let mut tree = PieceTree::new("aaa\nbbb\nccc\nddd\neee");
        tree.delete(7, 1);
        assert_eq!(text(&tree), "aaa\nbbbccc\nddd\neee");

        for start in 0..4usize {
            for n in 0..=4usize {
                let by_range = lines_text(&tree, start, n);
                let by_single: String = (start..start + n).map(|l| line_text(&tree, l)).collect();
                assert_eq!(by_range, by_single, "start={} n={}", start, n);
            }
        }
    }

    // ============================================================
    // get_lines_text — stitched across left / node / right
    //
    // A single logical line can be assembled from pieces living in the root's
    // left subtree, the root node's own piece, and the root's right subtree.
    // `get_lines_text` must stitch all three back together in order. This test
    // grows a tree large and mixed enough that at least one line genuinely
    // spans all three regions, then checks every window of lines against a
    // reference model.
    // ============================================================

    fn assert_all_windows_match(tree: &PieceTree, doc: &str) {
        let lines = reference_lines(doc);
        for start in 0..lines.len() {
            for n in 0..=(lines.len() - start) {
                let expected: String = lines.iter().skip(start).take(n).cloned().collect();
                assert_eq!(
                    lines_text(tree, start, n),
                    expected,
                    "get_lines_text({}, {}) mismatch",
                    start,
                    n
                );
            }
        }
        assert_eq!(lines_text(tree, lines.len(), 1), "");
        assert_eq!(lines_text(tree, lines.len() + 7, 3), "");
    }

    #[test]
    fn get_lines_text_line_spans_left_node_and_right_regions() {
        let mut tree = PieceTree::new("seed line 0\nseed line 1\nseed line 2\n");
        let mut model = ReferenceModel::new("seed line 0\nseed line 1\nseed line 2\n");

        for i in 0..250 {
            let boundaries = char_boundaries(&model.text);
            let snippet = if i % 4 == 0 {
                format!("chunk{:03}\n", i)
            } else {
                format!("<{}>", i)
            };
            let at = boundaries[SimplePrng::new(i as u64 + 1).next_range(0, boundaries.len())];
            tree.insert(&snippet, at);
            model.insert(&snippet, at);

            if i % 7 == 3 && !model.text.is_empty() {
                let boundaries = char_boundaries(&model.text);
                let n = boundaries.len();
                if n > 1 {
                    let a = SimplePrng::new(i as u64 + 99).next_range(0, n - 1);
                    let max_span = (n - 1 - a).min(4);
                    let b = a + 1 + SimplePrng::new(i as u64 + 123).next_range(0, max_span);
                    let start = boundaries[a];
                    let len = boundaries[b] - start;
                    tree.delete(start, len);
                    model.delete(start, len);
                }
            }
        }

        assert_eq!(
            text(&tree),
            model.text,
            "tree and model diverged while building"
        );

        let spans = tree.line_spans();
        assert!(
            spans.iter().any(|s| s.spans_all_regions()),
            "expected at least one line to span left/node/right; spans: {:?}",
            spans
        );
        assert!(
            spans.iter().any(|s| s.pieces >= 3),
            "expected at least one line made of 3+ pieces; spans: {:?}",
            spans
        );

        assert_all_windows_match(&tree, &model.text);
    }

    // ============================================================
    // get_start_offset_of_a_line
    // ============================================================

    #[test]
    fn line_start_offset_basic() {
        // "one\n" = 4 bytes, "two\n" = 4 bytes, "three" = 5 bytes.
        let tree = PieceTree::new("one\ntwo\nthree");
        assert_eq!(line_start_offset(&tree, 0), 0);
        assert_eq!(line_start_offset(&tree, 1), 4);
        assert_eq!(line_start_offset(&tree, 2), 8);
        // Out of range maps to the end of the document.
        assert_eq!(line_start_offset(&tree, 3), 13);
        assert_eq!(line_start_offset(&tree, 10), 13);
        assert_eq!(line_start_offset(&tree, usize::MAX), 13);
    }

    #[test]
    fn line_start_offset_empty_and_single_line() {
        let empty = PieceTree::new("");
        assert_eq!(line_start_offset(&empty, 0), 0);
        assert_eq!(line_start_offset(&empty, 1), 0);

        let single = PieceTree::new("hello");
        assert_eq!(line_start_offset(&single, 0), 0);
        // Only line 0 exists; anything past it is the end offset.
        assert_eq!(line_start_offset(&single, 1), 5);
        assert_eq!(line_start_offset(&single, 99), 5);
    }

    #[test]
    fn line_start_offset_trailing_and_consecutive_newlines() {
        let trailing = PieceTree::new("hello\n");
        assert_eq!(line_start_offset(&trailing, 0), 0);
        assert_eq!(line_start_offset(&trailing, 1), 6);
        assert_eq!(line_start_offset(&trailing, 2), 6);

        // "one\n"=4, "\n"=1, "three"=5 -> 10 bytes total.
        let consecutive = PieceTree::new("one\n\nthree");
        assert_eq!(line_start_offset(&consecutive, 0), 0);
        assert_eq!(line_start_offset(&consecutive, 1), 4);
        assert_eq!(line_start_offset(&consecutive, 2), 5);
        assert_eq!(line_start_offset(&consecutive, 3), 10);

        let only_newlines = PieceTree::new("\n\n\n");
        assert_eq!(line_start_offset(&only_newlines, 0), 0);
        assert_eq!(line_start_offset(&only_newlines, 1), 1);
        assert_eq!(line_start_offset(&only_newlines, 2), 2);
        assert_eq!(line_start_offset(&only_newlines, 3), 3);
        assert_eq!(line_start_offset(&only_newlines, 4), 3);
    }

    #[test]
    fn line_start_offset_crlf_and_mixed_newlines() {
        // Each CRLF line is 5 bytes ("one\r\n").
        let crlf = PieceTree::new("one\r\ntwo\r\nthree");
        assert_eq!(line_start_offset(&crlf, 0), 0);
        assert_eq!(line_start_offset(&crlf, 1), 5);
        assert_eq!(line_start_offset(&crlf, 2), 10);
        assert_eq!(line_start_offset(&crlf, 3), 15);

        let mixed = PieceTree::new("line1\r\nline2\nline3\r\nline4\n");
        // 7, 6, 7, 6, then a trailing empty line.
        assert_eq!(line_start_offset(&mixed, 0), 0);
        assert_eq!(line_start_offset(&mixed, 1), 7);
        assert_eq!(line_start_offset(&mixed, 2), 13);
        assert_eq!(line_start_offset(&mixed, 3), 20);
        assert_eq!(line_start_offset(&mixed, 4), 26);
        assert_eq!(line_start_offset(&mixed, 5), 26);
    }

    #[test]
    fn line_start_offset_after_inserting_newline() {
        let mut tree = PieceTree::new("hello world");
        tree.insert("\n", 5);
        // "hello\n world"
        assert_eq!(line_start_offset(&tree, 0), 0);
        assert_eq!(line_start_offset(&tree, 1), 6);
        assert_eq!(line_start_offset(&tree, 2), 12);
    }

    #[test]
    fn line_start_offset_after_deleting_newline() {
        let mut tree = PieceTree::new("hello\nworld");
        assert_eq!(line_start_offset(&tree, 1), 6);

        tree.delete(5, 1);
        // Merged into a single line "helloworld".
        assert_eq!(line_start_offset(&tree, 0), 0);
        assert_eq!(line_start_offset(&tree, 1), 10);
    }

    #[test]
    fn line_start_offset_unicode_byte_offsets() {
        // 🦀=4 bytes, 🚀=4 bytes, 🎉=4 bytes.
        let tree = PieceTree::new("🦀 Rust\n🚀 Rocket\n🎉 Party\n");
        assert_eq!(line_start_offset(&tree, 0), 0);
        assert_eq!(line_start_offset(&tree, 1), 10);
        assert_eq!(line_start_offset(&tree, 2), 22);
        assert_eq!(line_start_offset(&tree, 3), 33);
        assert_eq!(line_start_offset(&tree, 4), 33);
    }

    #[test]
    fn line_start_offset_consistent_with_line_text_and_sub_text() {
        // The offset of a line, combined with its length, must reproduce the
        // exact line content (including its terminator) via get_sub_text, and
        // consecutive line offsets must be contiguous.
        let mut tree = PieceTree::new("aaa\nbbb\nccc\nddd\neee");
        tree.insert("XX", 3);
        tree.delete(7, 1);

        let mut doc = String::new();
        tree.get_text(&mut doc);
        let lines = reference_lines(&doc);

        let mut expected_offset = 0;
        for (i, line) in lines.iter().enumerate() {
            let offset = line_start_offset(&tree, i);
            assert_eq!(offset, expected_offset, "start offset of line {}", i);
            assert_eq!(line_text(&tree, i), *line, "line {} content", i);
            assert_eq!(
                sub_text(&tree, offset, line.len()),
                *line,
                "sub_text at line {} start",
                i
            );
            expected_offset += line.len();
        }
        assert_eq!(line_start_offset(&tree, lines.len()), doc.len());
    }

    #[test]
    fn line_start_offset_differential_fuzz() {
        let seeds = [3u64, 42, 0xABCD, 20240924];
        let snippets = [
            "a",
            "bc",
            "hello\n",
            "line\n",
            "\n",
            "\r\n",
            "\n\n",
            "foo\nbar\nbaz\n",
            "world",
            "🦀",
            "世界",
            " \n ",
            "x\ny\nz\n",
        ];

        for &seed in &seeds {
            let mut prng = SimplePrng::new(seed);
            let mut tree = PieceTree::new("initial\ncontent\n");
            let mut model = ReferenceModel::new("initial\ncontent\n");

            for step in 0..200 {
                let op = prng.next_range(0, 100);
                if op < 50 || model.text.is_empty() {
                    let snippet = snippets[prng.next_range(0, snippets.len())];
                    let boundaries = char_boundaries(&model.text);
                    let at = boundaries[prng.next_range(0, boundaries.len())];
                    tree.insert(snippet, at);
                    model.insert(snippet, at);
                } else if op < 75 {
                    let boundaries = char_boundaries(&model.text);
                    if boundaries.len() > 1 {
                        let a = prng.next_range(0, boundaries.len());
                        let b = prng.next_range(0, boundaries.len());
                        let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
                        let start = boundaries[lo];
                        let len = boundaries[hi] - start;
                        tree.delete(start, len);
                        model.delete(start, len);
                    }
                } else if op < 88 {
                    tree.undo();
                    model.undo();
                } else {
                    tree.redo();
                    model.redo();
                }

                assert_eq!(text(&tree), model.text, "seed {} step {}", seed, step);

                let lines = reference_lines(&model.text);
                for (i, line) in lines.iter().enumerate() {
                    let expected = reference_line_start_offset(&model.text, i);
                    assert_eq!(
                        line_start_offset(&tree, i),
                        expected,
                        "seed {} step {}: line {} offset",
                        seed,
                        step,
                        i
                    );
                    assert_eq!(
                        sub_text(&tree, expected, line.len()),
                        *line,
                        "seed {} step {}: sub_text at line {}",
                        seed,
                        step,
                        i
                    );
                }

                for &extra in &[lines.len(), lines.len() + 1, lines.len() + 7] {
                    assert_eq!(
                        line_start_offset(&tree, extra),
                        model.text.len(),
                        "seed {} step {}: out-of-range line {}",
                        seed,
                        step,
                        extra
                    );
                }
            }
        }
    }

    // ============================================================
    // Undo / redo
    // ============================================================

    #[test]
    fn undo_redo_basic() {
        let mut tree = PieceTree::new("abcdefghij");
        tree.insert("XXX", 5);
        assert_eq!(text(&tree), "abcdeXXXfghij");
        tree.undo();
        assert_eq!(text(&tree), "abcdefghij");
        tree.redo();
        assert_eq!(text(&tree), "abcdeXXXfghij");
    }

    #[test]
    fn undo_redo_empty_history_is_noop() {
        let mut tree = PieceTree::new("hello");
        tree.undo();
        assert_eq!(text(&tree), "hello");
        tree.redo();
        assert_eq!(text(&tree), "hello");
    }

    #[test]
    fn new_edit_after_undo_discards_redo() {
        let mut tree = PieceTree::new("hello");
        tree.insert("X", 0);
        tree.undo();
        assert_eq!(text(&tree), "hello");
        tree.insert("Y", 5);
        assert_eq!(text(&tree), "helloY");
        tree.redo();
        assert_eq!(text(&tree), "helloY");

        let mut tree2 = PieceTree::new("hello");
        tree2.insert("X", 0);
        tree2.undo();
        tree2.delete(0, 1);
        assert_eq!(text(&tree2), "ello");
        tree2.redo();
        assert_eq!(text(&tree2), "ello");
    }

    #[test]
    fn undo_delete_entire_document() {
        let mut tree = PieceTree::new("entire document");
        tree.delete(0, 15);
        assert_eq!(text(&tree), "");
        tree.undo();
        assert_eq!(text(&tree), "entire document");
        tree.redo();
        assert_eq!(text(&tree), "");
    }

    #[test]
    fn multiple_undos_and_redos_walk_history() {
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
        tree.undo();
        assert_eq!(text(&tree), "abc");

        tree.redo();
        assert_eq!(text(&tree), "abc1");
        tree.redo();
        assert_eq!(text(&tree), "abc12");
        tree.redo();
        assert_eq!(text(&tree), "abc123");
        tree.redo();
        assert_eq!(text(&tree), "abc123");
    }

    // ============================================================
    // Unicode (byte-offset indexing)
    // ============================================================

    #[test]
    fn unicode_multibyte_characters() {
        // 2-byte Greek.
        let mut two = PieceTree::new("αβγδ");
        assert_eq!(text(&two), "αβγδ");
        two.insert("ε", 4);
        assert_eq!(text(&two), "αβεγδ");
        assert_eq!(sub_text(&two, 2, 4), "βε");
        two.delete(4, 2);
        assert_eq!(text(&two), "αβγδ");

        // 3-byte CJK.
        let mut three = PieceTree::new("你好世界");
        three.insert("大", 6);
        assert_eq!(text(&three), "你好大世界");
        assert_eq!(sub_text(&three, 3, 9), "好大世");
        three.delete(6, 6);
        assert_eq!(text(&three), "你好界");

        // 4-byte emoji.
        let mut four = PieceTree::new("🦀🚀🔥");
        four.insert("🎉", 8);
        assert_eq!(text(&four), "🦀🚀🎉🔥");
        assert_eq!(sub_text(&four, 4, 8), "🚀🎉");
        four.delete(8, 4);
        assert_eq!(text(&four), "🦀🚀🔥");
    }

    #[test]
    fn unicode_multiline_edits() {
        let content = "🦀 Rust\n🚀 Rocket\n🎉 Party\n";
        let tree = PieceTree::new(content);
        assert_eq!(line_text(&tree, 0), "🦀 Rust\n");
        assert_eq!(line_text(&tree, 1), "🚀 Rocket\n");
        assert_eq!(line_text(&tree, 2), "🎉 Party\n");
        assert_eq!(line_text(&tree, 3), "");

        let mut modified = PieceTree::new(content);
        modified.delete("🦀 Rust".len(), 1);
        assert_eq!(line_text(&modified, 0), "🦀 Rust🚀 Rocket\n");
        assert_eq!(line_text(&modified, 1), "🎉 Party\n");
    }

    // ============================================================
    // Regression: non-UTF-8-boundary byte offsets must not panic
    // ============================================================

    #[test]
    fn sub_text_mid_char_offsets_do_not_panic() {
        // 2-byte chars: α=0..2, β=2..4, γ=4..6, δ=6..8
        let two = PieceTree::new("αβγδ");
        assert_eq!(sub_text(&two, 1, 1), "α");
        assert_eq!(sub_text(&two, 1, 2), "αβ");
        assert_eq!(sub_text(&two, 3, 1), "β");

        // 3-byte chars: 你=0..3, 好=3..6
        let three = PieceTree::new("你好");
        assert_eq!(sub_text(&three, 1, 1), "你");
        assert_eq!(sub_text(&three, 4, 1), "好");

        // 4-byte chars: 🦀=0..4, 🚀=4..8
        let four = PieceTree::new("🦀🚀");
        assert_eq!(sub_text(&four, 2, 1), "🦀");
        assert_eq!(sub_text(&four, 5, 1), "🚀");
    }

    #[test]
    fn sub_text_mid_char_offsets_across_pieces_do_not_panic() {
        let mut tree = PieceTree::new("αβγδ");
        tree.insert("x", 4); // "αβxγδ"
        assert_eq!(sub_text(&tree, 1, 4), "αβx");
    }

    #[test]
    fn insert_at_mid_char_offset_keeps_valid_utf8() {
        let mut tree = PieceTree::new("αβγδ");
        tree.insert("x", 1); // byte 1 is inside 'α' -> must snap to 0
        assert_eq!(text(&tree), "xαβγδ");
        assert_eq!(line_text(&tree, 0), "xαβγδ");
    }

    #[test]
    fn delete_at_mid_char_offset_keeps_valid_utf8() {
        let mut tree = PieceTree::new("αβγδ");
        tree.delete(1, 1); // [1,2) is inside 'α' -> snap to [0,2), delete "α"
        assert_eq!(text(&tree), "βγδ");
        assert_eq!(line_text(&tree, 0), "βγδ");
    }

    // ============================================================
    // Regression: red-black tree panics
    // ============================================================

    #[test]
    fn delete_should_not_walk_past_black_leaf() {
        apply_edits(
            "",
            &[
                Edit::Ins("(263:3)", 0),
                Edit::Ins("(264:11)", 1),
                Edit::Ins("(270:77)", 8),
                Edit::Ins("(271:69)", 2),
                Edit::Ins("(272:82)", 26),
                Edit::Del(3, 20),
                Edit::Del(9, 1),
                Edit::Del(3, 1),
            ],
        );

        apply_edits(
            "hello world",
            &[
                Edit::Del(1, 1),
                Edit::Ins("(287:70)", 4),
                Edit::Del(10, 1),
                Edit::Ins("(289:68)", 8),
                Edit::Ins("(290:84)", 13),
                Edit::Del(13, 3),
                Edit::Ins("(293:61)", 10),
                Edit::Del(9, 24),
            ],
        );
    }

    #[test]
    fn delete_should_not_redden_a_negative_black() {
        apply_edits(
            "The quick brown fox jumps over the lazy dog",
            &[
                Edit::Ins("(357:28)", 1),
                Edit::Ins("(364:63)", 2),
                Edit::Del(8, 1),
                Edit::Ins("(367:17)", 3),
                Edit::Ins("(368:31)", 10),
                Edit::Del(11, 6),
                Edit::Del(8, 10),
            ],
        );
    }

    // ============================================================
    // Differential fuzz tests
    //
    // Drive the public API with a deterministic PRNG and compare the document
    // against a plain-String reference model at every step. These are the
    // tests that catch edge cases we did not anticipate: they exercise
    // arbitrary piece splits, deletions spanning many pieces, undo/redo
    // interleavings, and out-of-range clamping.
    // ============================================================

    #[test]
    fn differential_fuzz_random_operations() {
        let snippets = [
            "a",
            "bc",
            "Hello, world! ",
            "foo\nbar\n",
            "xyz\r\n123",
            "\n",
            "\r\n",
            "🦀",
            "世界",
            " ",
        ];
        let seeds = [42u64, 1337, 2024, 99999, 12345678];

        for &seed in &seeds {
            let mut prng = SimplePrng::new(seed);
            let mut tree = PieceTree::new("initial_content\n");
            let mut model = ReferenceModel::new("initial_content\n");

            for _ in 0..120 {
                let op = prng.next_range(0, 100);
                if op < 40 {
                    let snippet = snippets[prng.next_range(0, snippets.len())];
                    let boundaries = char_boundaries(&model.text);
                    let offset = boundaries[prng.next_range(0, boundaries.len())];
                    tree.insert(snippet, offset);
                    model.insert(snippet, offset);
                } else if op < 65 {
                    let boundaries = char_boundaries(&model.text);
                    if boundaries.len() > 1 {
                        let idx1 = prng.next_range(0, boundaries.len());
                        let idx2 = prng.next_range(0, boundaries.len());
                        let (start_idx, end_idx) = if idx1 <= idx2 {
                            (idx1, idx2)
                        } else {
                            (idx2, idx1)
                        };
                        let start = boundaries[start_idx];
                        let length = boundaries[end_idx] - start;
                        tree.delete(start, length);
                        model.delete(start, length);
                    }
                } else if op < 85 {
                    tree.undo();
                    model.undo();
                } else {
                    tree.redo();
                    model.redo();
                }

                assert_eq!(text(&tree), model.text, "text mismatch with seed {}", seed);

                let boundaries = char_boundaries(&model.text);
                if boundaries.len() > 1 {
                    let idx1 = prng.next_range(0, boundaries.len());
                    let idx2 = prng.next_range(0, boundaries.len());
                    let (start_idx, end_idx) = if idx1 <= idx2 {
                        (idx1, idx2)
                    } else {
                        (idx2, idx1)
                    };
                    let start = boundaries[start_idx];
                    let length = boundaries[end_idx] - start;
                    assert_eq!(
                        sub_text(&tree, start, length),
                        model.sub_text(start, length),
                        "sub_text mismatch at ({}, {}) with seed {}",
                        start,
                        length,
                        seed
                    );
                }

                let line_count = reference_lines(&model.text).len();
                let test_line = prng.next_range(0, line_count + 3);
                assert_eq!(
                    line_text(&tree, test_line),
                    model.line_text(test_line),
                    "line text mismatch at line {} with seed {}",
                    test_line,
                    seed
                );
            }
        }
    }

    #[test]
    fn differential_fuzz_undo_redo_stress() {
        let seeds = [55555u64, 77777, 88888];

        for &seed in &seeds {
            let mut prng = SimplePrng::new(seed);
            let mut tree = PieceTree::new("");
            let mut model = ReferenceModel::new("");

            for i in 0..50 {
                let piece = format!("chunk{}_", i);
                let boundaries = char_boundaries(&model.text);
                let offset = boundaries[prng.next_range(0, boundaries.len())];
                tree.insert(&piece, offset);
                model.insert(&piece, offset);
                assert_eq!(text(&tree), model.text);
            }

            for _ in 0..60 {
                if prng.next_range(0, 2) == 0 {
                    tree.undo();
                    model.undo();
                } else {
                    tree.redo();
                    model.redo();
                }
                assert_eq!(
                    text(&tree),
                    model.text,
                    "mismatch during undo/redo stress, seed {}",
                    seed
                );
            }

            for _ in 0..60 {
                tree.undo();
                model.undo();
                assert_eq!(text(&tree), model.text);
            }
            assert_eq!(text(&tree), "");

            for _ in 0..60 {
                tree.redo();
                model.redo();
                assert_eq!(text(&tree), model.text);
            }
        }
    }

    #[test]
    fn differential_fuzz_large_deletions() {
        let mut prng = SimplePrng::new(0xCAFE_BABE);
        let mut tree = PieceTree::new("abcdefghijklmnopqrstuvwxyz");
        let mut model = ReferenceModel::new("abcdefghijklmnopqrstuvwxyz");

        for step in 0..1000 {
            let len = model.text.len();
            if len == 0 || prng.next_range(0, 2) == 0 {
                let snippet = format!("({}:{})", step, prng.next_range(0, 100));
                let pos = prng.next_range(0, len + 1);
                tree.insert(&snippet, pos);
                model.insert(&snippet, pos);
            } else {
                let pos = prng.next_range(0, len);
                let del = 1 + prng.next_range(0, len - pos);
                tree.delete(pos, del);
                model.delete(pos, del);
            }
            assert_eq!(text(&tree), model.text, "step {}", step);
        }
    }

    #[test]
    fn sub_text_fuzz_matches_reference() {
        let mut prng = SimplePrng::new(0x0123_4567);
        let mut tree = PieceTree::new("the quick brown fox");
        let mut model = ReferenceModel::new("the quick brown fox");

        for step in 0..500 {
            // Query arbitrary (often out-of-range) windows; both must clamp the
            // same way.
            let start = prng.next_range(0, model.text.len() + 3);
            let length = prng.next_range(0, model.text.len() + 3);
            assert_eq!(
                sub_text(&tree, start, length),
                model.sub_text(start, length),
                "step {}: get_sub_text({}, {})",
                step,
                start,
                length
            );

            // Churn the document so windows routinely cross pieces.
            if prng.next_range(0, 2) == 0 {
                let snippet = format!("({})", step);
                let pos = prng.next_range(0, model.text.len() + 1);
                tree.insert(&snippet, pos);
                model.insert(&snippet, pos);
            } else if !model.text.is_empty() {
                let pos = prng.next_range(0, model.text.len());
                let del = 1 + prng.next_range(0, model.text.len() - pos);
                tree.delete(pos, del);
                model.delete(pos, del);
            }
        }
    }

    #[test]
    fn line_text_fuzz_matches_reference() {
        let mut prng = SimplePrng::new(0xFEED_FACE);
        let mut tree = PieceTree::new("the quick\nbrown fox\njumps over\n");
        let mut model = ReferenceModel::new("the quick\nbrown fox\njumps over\n");
        let snippets = [
            "hello\n",
            "\n",
            "world",
            "\n\n",
            "foo\nbar\n",
            "baz",
            "line\r\n",
            "🦀",
        ];

        for step in 0..500 {
            let lines = reference_lines(&model.text);
            for (i, expected) in lines.iter().enumerate() {
                assert_eq!(line_text(&tree, i), *expected, "step {}: line {}", step, i);
            }
            assert_eq!(line_text(&tree, lines.len()), "");
            assert_eq!(line_text(&tree, lines.len() + 10), "");

            if prng.next_range(0, 100) < 55 || model.text.is_empty() {
                let snippet = snippets[prng.next_range(0, snippets.len())];
                let boundaries = char_boundaries(&model.text);
                let pos = boundaries[prng.next_range(0, boundaries.len())];
                tree.insert(snippet, pos);
                model.insert(snippet, pos);
            } else {
                let boundaries = char_boundaries(&model.text);
                let n = boundaries.len();
                if n > 1 {
                    let a = prng.next_range(0, n);
                    let b = prng.next_range(0, n);
                    let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
                    let start = boundaries[lo];
                    let len = boundaries[hi] - start;
                    tree.delete(start, len);
                    model.delete(start, len);
                }
            }
        }
    }

    #[test]
    fn get_lines_text_differential_fuzz_multi_seed() {
        let seeds = [
            0x0000_0001u64,
            0x0000_002A,
            0x0000_0539,
            0x0001_869F,
            0xDEAD_BEEF,
            0x0BAD_F00D,
            0x1234_5678_9ABC_DEF0,
            0xCAFE_BABE_89AB_CDEF,
        ];

        for &seed in &seeds {
            let mut prng = SimplePrng::new(seed);
            let mut tree = PieceTree::new("initial\ncontent\nwith\nseveral\nlines\n");
            let mut model = ReferenceModel::new("initial\ncontent\nwith\nseveral\nlines\n");

            let snippets = [
                "a",
                "bc",
                "hello\n",
                "line\n",
                "\n",
                "\r\n",
                "\n\n",
                "foo\nbar\nbaz\n",
                "world",
                "🦀",
                "世界",
                " \n ",
                "x\ny\nz\n",
            ];

            for step in 0..300 {
                let op = prng.next_range(0, 100);
                if op < 45 || model.text.is_empty() {
                    let snippet = snippets[prng.next_range(0, snippets.len())];
                    let boundaries = char_boundaries(&model.text);
                    let at = boundaries[prng.next_range(0, boundaries.len())];
                    tree.insert(snippet, at);
                    model.insert(snippet, at);
                } else if op < 75 {
                    let boundaries = char_boundaries(&model.text);
                    if boundaries.len() > 1 {
                        let a = prng.next_range(0, boundaries.len());
                        let b = prng.next_range(0, boundaries.len());
                        let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
                        let start = boundaries[lo];
                        let len = boundaries[hi] - start;
                        tree.delete(start, len);
                        model.delete(start, len);
                    }
                } else if op < 88 {
                    tree.undo();
                    model.undo();
                } else {
                    tree.redo();
                    model.redo();
                }

                assert_eq!(
                    text(&tree),
                    model.text,
                    "seed {:#x} step {}: text mismatch",
                    seed,
                    step
                );

                let lines = reference_lines(&model.text);
                let total = lines.len();

                let start = prng.next_range(0, total + 3);
                let n = prng.next_range(0, total + 3);
                let expected: String = lines.iter().skip(start).take(n).cloned().collect();
                assert_eq!(
                    lines_text(&tree, start, n),
                    expected,
                    "seed {:#x} step {}: get_lines_text({}, {}) mismatch",
                    seed,
                    step,
                    start,
                    n
                );

                let by_single: String = (start..start + n).map(|l| line_text(&tree, l)).collect();
                assert_eq!(
                    lines_text(&tree, start, n),
                    by_single,
                    "seed {:#x} step {}: window vs single-line mismatch",
                    seed,
                    step
                );

                assert_eq!(lines_text(&tree, total, 1), "");
            }
        }
    }

    #[test]
    fn differential_fuzz_keeps_red_black_invariants() {
        let seeds = [1u64, 7, 99, 1234, 0xDEAD_BEEF];

        for &seed in &seeds {
            let mut prng = SimplePrng::new(seed);
            let mut tree = PieceTree::new("");
            let mut model = ReferenceModel::new("");

            for step in 0..200 {
                let op = prng.next_range(0, 100);
                if op < 50 || model.text.is_empty() {
                    let n = 1 + prng.next_range(0, 4);
                    let snippet: String = (0..n)
                        .map(|_| char::from(b'a' + prng.next_range(0, 4) as u8))
                        .collect();
                    let pos = prng.next_range(0, model.text.len() + 1);
                    tree.insert(&snippet, pos);
                    model.insert(&snippet, pos);
                } else if op < 65 {
                    let pos = prng.next_range(0, model.text.len());
                    let del = 1 + prng.next_range(0, model.text.len() - pos);
                    tree.delete(pos, del);
                    model.delete(pos, del);
                } else if op < 85 {
                    tree.undo();
                    model.undo();
                } else {
                    tree.redo();
                    model.redo();
                }

                assert_eq!(text(&tree), model.text, "seed {} step {}: text", seed, step);

                let violations = tree.check_invariants();
                assert!(
                    violations.is_empty(),
                    "seed {} step {}: red-black invariants broken: {:?}",
                    seed,
                    step,
                    violations
                );
            }
        }
    }

    // ============================================================
    // Length & line feed count lifecycle
    // ============================================================

    #[test]
    fn tree_len_and_line_feed_count_lifecycle() {
        let empty = PieceTree::new("");
        assert_eq!(empty.get_tree_len(), 0);
        assert_eq!(empty.get_line_feed_count(), 0);

        let default_tree: PieceTree = Default::default();
        assert_eq!(default_tree.get_tree_len(), 0);
        assert_eq!(default_tree.get_line_feed_count(), 0);

        let single = PieceTree::new("hello world");
        assert_eq!(single.get_tree_len(), 11);
        assert_eq!(single.get_line_feed_count(), 0);

        let multi = PieceTree::new("one\ntwo\nthree\n");
        assert_eq!(multi.get_tree_len(), 14);
        assert_eq!(multi.get_line_feed_count(), 3);

        let crlf = PieceTree::new("one\r\ntwo\r\nthree\r\n");
        assert_eq!(crlf.get_tree_len(), 17);
        assert_eq!(crlf.get_line_feed_count(), 3);

        let emoji = PieceTree::new("🦀🚀🔥");
        assert_eq!(emoji.get_tree_len(), 12); // 4 bytes each * 3 = 12
        assert_eq!(emoji.get_line_feed_count(), 0);

        // Dynamic edits update both metrics accurately
        let mut tree = PieceTree::new("hello");
        assert_eq!(tree.get_tree_len(), 5);
        assert_eq!(tree.get_line_feed_count(), 0);

        tree.insert("\nworld\n", 5);
        assert_eq!(tree.get_tree_len(), 12);
        assert_eq!(tree.get_line_feed_count(), 2);
        assert_eq!(text(&tree), "hello\nworld\n");

        tree.insert("there\n", 6);
        assert_eq!(tree.get_tree_len(), 18);
        assert_eq!(tree.get_line_feed_count(), 3);
        assert_eq!(text(&tree), "hello\nthere\nworld\n");

        // Delete newlines and content
        tree.delete(5, 1); // deletes '\n' after hello -> "hellothere\nworld\n"
        assert_eq!(tree.get_tree_len(), 17);
        assert_eq!(tree.get_line_feed_count(), 2);

        tree.delete(0, 17); // delete all
        assert_eq!(tree.get_tree_len(), 0);
        assert_eq!(tree.get_line_feed_count(), 0);
        assert_eq!(text(&tree), "");

        // Undo and redo restore len and line feed count
        tree.undo();
        assert_eq!(tree.get_tree_len(), 17);
        assert_eq!(tree.get_line_feed_count(), 2);

        tree.undo();
        assert_eq!(tree.get_tree_len(), 18);
        assert_eq!(tree.get_line_feed_count(), 3);

        tree.redo();
        assert_eq!(tree.get_tree_len(), 17);
        assert_eq!(tree.get_line_feed_count(), 2);

        tree.redo();
        assert_eq!(tree.get_tree_len(), 0);
        assert_eq!(tree.get_line_feed_count(), 0);
    }

    // ============================================================
    // Consecutive typing optimization (piece expansion) & undo/redo
    // ============================================================

    #[test]
    fn consecutive_typing_and_stepwise_undo_redo() {
        let mut tree = PieceTree::new("");
        let input = "hello\nworld";

        for (i, ch) in input.chars().enumerate() {
            let mut buf = [0u8; 4];
            let s = ch.encode_utf8(&mut buf);
            tree.insert(s, tree.get_tree_len());
            assert_eq!(tree.get_tree_len(), i + 1);
            assert_eq!(text(&tree), &input[..i + 1]);
            assert!(tree.check_invariants().is_empty());
        }

        assert_eq!(text(&tree), "hello\nworld");
        assert_eq!(tree.get_line_feed_count(), 1);

        // Step-by-step undo walks back each single character
        for i in (0..input.len()).rev() {
            tree.undo();
            assert_eq!(text(&tree), &input[..i]);
            assert_eq!(tree.get_tree_len(), i);
            assert!(tree.check_invariants().is_empty());
        }

        assert_eq!(text(&tree), "");
        assert_eq!(tree.get_tree_len(), 0);

        // Step-by-step redo restores each single character
        for i in 1..=input.len() {
            tree.redo();
            assert_eq!(text(&tree), &input[..i]);
            assert_eq!(tree.get_tree_len(), i);
            assert!(tree.check_invariants().is_empty());
        }

        assert_eq!(text(&tree), "hello\nworld");
    }

    // ============================================================
    // Edge cases for get_line_text
    // ============================================================

    #[test]
    fn line_text_extreme_line_numbers_and_boundaries() {
        let empty = PieceTree::new("");
        assert_eq!(line_text(&empty, 0), "");
        assert_eq!(line_text(&empty, 1), "");
        assert_eq!(line_text(&empty, 100), "");
        assert_eq!(line_text(&empty, usize::MAX), "");

        let single = PieceTree::new("no newlines at all");
        assert_eq!(line_text(&single, 0), "no newlines at all");
        assert_eq!(line_text(&single, 1), "");
        assert_eq!(line_text(&single, 2), "");
        assert_eq!(line_text(&single, usize::MAX), "");

        let trailing = PieceTree::new("one\n");
        assert_eq!(line_text(&trailing, 0), "one\n");
        assert_eq!(line_text(&trailing, 1), "");
        assert_eq!(line_text(&trailing, 2), "");
        assert_eq!(line_text(&trailing, usize::MAX), "");

        let only_nl = PieceTree::new("\n\n");
        assert_eq!(line_text(&only_nl, 0), "\n");
        assert_eq!(line_text(&only_nl, 1), "\n");
        assert_eq!(line_text(&only_nl, 2), "");
        assert_eq!(line_text(&only_nl, 3), "");
        assert_eq!(line_text(&only_nl, usize::MAX), "");
    }

    // ============================================================
    // Edge cases for get_lines_text
    // ============================================================

    #[test]
    fn get_lines_text_zero_count_and_extreme_indices() {
        let tree = PieceTree::new("first\nsecond\nthird");

        // Requesting 0 lines always yields empty string
        assert_eq!(lines_text(&tree, 0, 0), "");
        assert_eq!(lines_text(&tree, 1, 0), "");
        assert_eq!(lines_text(&tree, 2, 0), "");
        assert_eq!(lines_text(&tree, 3, 0), "");
        assert_eq!(lines_text(&tree, 100, 0), "");
        assert_eq!(lines_text(&tree, usize::MAX, 0), "");

        // Extreme start line numbers
        assert_eq!(lines_text(&tree, usize::MAX, 1), "");
        assert_eq!(lines_text(&tree, usize::MAX, usize::MAX), "");

        // Empty tree edge cases
        let empty = PieceTree::new("");
        assert_eq!(lines_text(&empty, 0, 0), "");
        assert_eq!(lines_text(&empty, 0, 1), "");
        assert_eq!(lines_text(&empty, 0, usize::MAX), "");
        assert_eq!(lines_text(&empty, usize::MAX, 0), "");
        assert_eq!(lines_text(&empty, usize::MAX, usize::MAX), "");
    }

    // ============================================================
    // Edge cases for get_start_offset_of_a_line
    // ============================================================

    #[test]
    fn line_start_offset_extreme_indices_and_boundaries() {
        let empty = PieceTree::new("");
        assert_eq!(line_start_offset(&empty, 0), 0);
        assert_eq!(line_start_offset(&empty, 1), 0);
        assert_eq!(line_start_offset(&empty, 100), 0);
        assert_eq!(line_start_offset(&empty, usize::MAX), 0);

        let single = PieceTree::new("hello");
        assert_eq!(line_start_offset(&single, 0), 0);
        assert_eq!(line_start_offset(&single, 1), 5);
        assert_eq!(line_start_offset(&single, 2), 5);
        assert_eq!(line_start_offset(&single, usize::MAX), 5);

        // Document without trailing newline: lines 0, 1, 2
        let tree = PieceTree::new("abc\ndef\nghi");
        assert_eq!(line_start_offset(&tree, 0), 0);
        assert_eq!(line_start_offset(&tree, 1), 4);
        assert_eq!(line_start_offset(&tree, 2), 8);
        assert_eq!(line_start_offset(&tree, 3), 11);
        assert_eq!(line_start_offset(&tree, 4), 11);
        assert_eq!(line_start_offset(&tree, usize::MAX), 11);

        // Document with trailing newline: lines 0, 1, 2, 3(empty)
        let trailing = PieceTree::new("abc\ndef\nghi\n");
        assert_eq!(line_start_offset(&trailing, 0), 0);
        assert_eq!(line_start_offset(&trailing, 1), 4);
        assert_eq!(line_start_offset(&trailing, 2), 8);
        assert_eq!(line_start_offset(&trailing, 3), 12);
        assert_eq!(line_start_offset(&trailing, 4), 12);
        assert_eq!(line_start_offset(&trailing, usize::MAX), 12);
    }

    // ============================================================
    // Edge cases for get_sub_text
    // ============================================================

    #[test]
    fn sub_text_zero_length_and_extreme_indices() {
        let tree = PieceTree::new("abcdefghij");

        // Zero length requests always return empty string
        assert_eq!(sub_text(&tree, 0, 0), "");
        assert_eq!(sub_text(&tree, 5, 0), "");
        assert_eq!(sub_text(&tree, 10, 0), "");
        assert_eq!(sub_text(&tree, 100, 0), "");
        assert_eq!(sub_text(&tree, usize::MAX, 0), "");

        // Extreme start indices
        assert_eq!(sub_text(&tree, usize::MAX, 1), "");
        assert_eq!(sub_text(&tree, usize::MAX, usize::MAX), "");
        assert_eq!(sub_text(&tree, 10, 1), "");
        assert_eq!(sub_text(&tree, 10, usize::MAX), "");

        // Slicing from valid offset with huge length clamps safely
        assert_eq!(sub_text(&tree, 0, usize::MAX), "abcdefghij");
        assert_eq!(sub_text(&tree, 4, usize::MAX), "efghij");
        assert_eq!(sub_text(&tree, 9, usize::MAX), "j");

        let empty = PieceTree::new("");
        assert_eq!(sub_text(&empty, 0, 0), "");
        assert_eq!(sub_text(&empty, 0, 10), "");
        assert_eq!(sub_text(&empty, usize::MAX, usize::MAX), "");
    }

    // ============================================================
    // Extreme deletion edge cases
    // ============================================================

    #[test]
    fn delete_extreme_ranges_and_stepwise_draining() {
        // Delete with usize::MAX length from offset 0 empties document
        let mut tree = PieceTree::new("hello world");
        tree.delete(0, usize::MAX);
        assert_eq!(text(&tree), "");
        assert_eq!(tree.get_tree_len(), 0);
        assert_eq!(tree.get_line_feed_count(), 0);
        assert!(tree.check_invariants().is_empty());

        // Delete with usize::MAX from middle
        let mut tree2 = PieceTree::new("hello world");
        tree2.delete(5, usize::MAX);
        assert_eq!(text(&tree2), "hello");
        assert_eq!(tree2.get_tree_len(), 5);
        assert!(tree2.check_invariants().is_empty());

        // Delete with extreme offset is no-op
        let mut tree3 = PieceTree::new("hello");
        tree3.delete(usize::MAX, 10);
        tree3.delete(usize::MAX, usize::MAX);
        tree3.delete(5, usize::MAX); // offset at end
        assert_eq!(text(&tree3), "hello");
        assert_eq!(tree3.get_tree_len(), 5);

        // Repeated single-byte delete from front until empty
        let mut drain_front = PieceTree::new("abcdefghij");
        for expected_len in (0..10).rev() {
            drain_front.delete(0, 1);
            assert_eq!(drain_front.get_tree_len(), expected_len);
            assert!(drain_front.check_invariants().is_empty());
        }
        assert_eq!(text(&drain_front), "");

        // Repeated single-byte delete from back until empty
        let mut drain_back = PieceTree::new("abcdefghij");
        for expected_len in (0..10).rev() {
            drain_back.delete(expected_len, 1);
            assert_eq!(drain_back.get_tree_len(), expected_len);
            assert!(drain_back.check_invariants().is_empty());
        }
        assert_eq!(text(&drain_back), "");
    }

    // ============================================================
    // Split CRLF across piece boundaries
    // ============================================================

    #[test]
    fn split_crlf_across_pieces_and_crlf_deletions() {
        // '\r' in original piece, '\n' in added piece
        let mut tree = PieceTree::new("hello\r");
        tree.insert("\nworld", 6);
        assert_eq!(text(&tree), "hello\r\nworld");
        assert_eq!(line_text(&tree, 0), "hello\r\n");
        assert_eq!(line_text(&tree, 1), "world");
        assert_eq!(tree.get_line_feed_count(), 1);
        assert_eq!(line_start_offset(&tree, 0), 0);
        assert_eq!(line_start_offset(&tree, 1), 7);
        assert!(tree.check_invariants().is_empty());

        // Deleting '\r' from CRLF leaves '\n'
        let mut del_cr = PieceTree::new("line1\r\nline2\r\n");
        del_cr.delete(5, 1); // remove '\r' of line 1
        assert_eq!(text(&del_cr), "line1\nline2\r\n");
        assert_eq!(line_text(&del_cr, 0), "line1\n");
        assert_eq!(line_text(&del_cr, 1), "line2\r\n");
        assert_eq!(del_cr.get_line_feed_count(), 2);

        // Deleting '\n' from CRLF leaves '\r', merging lines
        let mut del_lf = PieceTree::new("line1\r\nline2\r\n");
        del_lf.delete(6, 1); // remove '\n' of line 1
        assert_eq!(text(&del_lf), "line1\rline2\r\n");
        assert_eq!(line_text(&del_lf, 0), "line1\rline2\r\n");
        assert_eq!(line_text(&del_lf, 1), "");
        assert_eq!(del_lf.get_line_feed_count(), 1);
    }

    // ============================================================
    // Structural twin / identical pieces behavior
    // ============================================================

    #[test]
    fn structural_twin_identical_pieces_behavior() {
        // Multiple identical pieces placed across the tree
        let mut tree = PieceTree::new("block_");
        tree.insert("block_", 6);
        tree.insert("block_", 6);
        tree.insert("block_", 0);
        // "block_block_block_block_" (4 identical pieces)
        assert_eq!(text(&tree), "block_block_block_block_");
        assert_eq!(tree.get_tree_len(), 24);
        assert!(tree.check_invariants().is_empty());

        // Delete one twin piece from the middle
        tree.delete(6, 6);
        assert_eq!(text(&tree), "block_block_block_");
        assert_eq!(tree.get_tree_len(), 18);
        assert!(tree.check_invariants().is_empty());

        // Verify sub_text and line queries work across remaining identical pieces
        assert_eq!(sub_text(&tree, 0, 12), "block_block_");
        assert_eq!(sub_text(&tree, 6, 12), "block_block_");
    }

    // ============================================================
    // Heavy sequential prepending and appending (red-black rebalancing)
    // ============================================================

    #[test]
    fn heavy_sequential_rebalancing_left_right_alternating() {
        // Left-leaning heavy insertions (prepending)
        let mut left_tree = PieceTree::new("");
        for i in 0..100 {
            let chunk = format!("[{:03}]", i);
            left_tree.insert(&chunk, 0);
            assert!(
                left_tree.check_invariants().is_empty(),
                "left tree invariants broken at step {}",
                i
            );
        }
        assert_eq!(left_tree.get_tree_len(), 500);

        // Right-leaning heavy insertions (appending)
        let mut right_tree = PieceTree::new("");
        for i in 0..100 {
            let chunk = format!("[{:03}]", i);
            right_tree.insert(&chunk, right_tree.get_tree_len());
            assert!(
                right_tree.check_invariants().is_empty(),
                "right tree invariants broken at step {}",
                i
            );
        }
        assert_eq!(right_tree.get_tree_len(), 500);

        // Alternating prepend and append
        let mut alt_tree = PieceTree::new("center");
        for i in 0..60 {
            if i % 2 == 0 {
                alt_tree.insert("L", 0);
            } else {
                alt_tree.insert("R", alt_tree.get_tree_len());
            }
            assert!(
                alt_tree.check_invariants().is_empty(),
                "alt tree invariants broken at step {}",
                i
            );
        }
        assert_eq!(alt_tree.get_tree_len(), 66);
    }

    // ============================================================
    // Large deletion spanning dozens of pieces
    // ============================================================

    #[test]
    fn large_deletion_spanning_many_pieces() {
        let mut tree = PieceTree::new("");
        let mut model = ReferenceModel::new("");

        // Build 60 distinct pieces
        for i in 0..60 {
            let s = format!("p{:02}:", i);
            tree.insert(&s, tree.get_tree_len());
            model.insert(&s, model.text.len());
        }

        assert_eq!(text(&tree), model.text);
        assert_eq!(tree.get_tree_len(), 240);
        assert!(tree.check_invariants().is_empty());

        // Delete from piece 15 to piece 45 (spanning 30 pieces completely)
        let start = 15 * 4;
        let len = 30 * 4;
        tree.delete(start, len);
        model.delete(start, len);

        assert_eq!(text(&tree), model.text);
        assert_eq!(tree.get_tree_len(), 120);
        assert!(
            tree.check_invariants().is_empty(),
            "invariants broken after spanning deletion: {:?}",
            tree.check_invariants()
        );

        // Undo restores all 60 pieces
        tree.undo();
        model.undo();
        assert_eq!(text(&tree), model.text);
        assert_eq!(tree.get_tree_len(), 240);
        assert!(tree.check_invariants().is_empty());

        // Redo reapplies the large deletion
        tree.redo();
        model.redo();
        assert_eq!(text(&tree), model.text);
        assert_eq!(tree.get_tree_len(), 120);
        assert!(tree.check_invariants().is_empty());
    }

    // ============================================================
    // Complex Unicode & combining characters
    // ============================================================

    #[test]
    fn complex_unicode_and_combining_characters() {
        // Combining diacritical marks: 'e' + '\u{0301}' (combining acute accent)
        let mut tree = PieceTree::new("cafe\u{0301}");
        assert_eq!(text(&tree), "cafe\u{0301}");
        assert_eq!(tree.get_tree_len(), 6); // 'c','a','f','e' = 4 bytes, '\u{0301}' = 2 bytes

        // Insert another combining mark
        tree.insert("\u{0308}", 6); // combining diaeresis
        assert_eq!(text(&tree), "cafe\u{0301}\u{0308}");
        assert_eq!(tree.get_tree_len(), 8);

        // Zero-width joiner sequences: woman technologist "👩\u{200D}💻"
        let mut zwj = PieceTree::new("👩\u{200D}💻");
        assert_eq!(text(&zwj), "👩\u{200D}💻");
        zwj.insert(" works on ", 4); // right after the woman emoji (4 bytes)
        assert_eq!(text(&zwj), "👩 works on \u{200D}💻");
        assert!(zwj.check_invariants().is_empty());

        // Multibyte text with newlines and start offsets
        let multi_nl = PieceTree::new("🦀\n🚀🚀\n🎉🎉🎉\n");
        assert_eq!(multi_nl.get_line_feed_count(), 3);
        assert_eq!(line_start_offset(&multi_nl, 0), 0);
        assert_eq!(line_start_offset(&multi_nl, 1), 5); // 4 + 1
        assert_eq!(line_start_offset(&multi_nl, 2), 14); // 5 + 8 + 1
        assert_eq!(line_start_offset(&multi_nl, 3), 27); // 14 + 12 + 1
        assert_eq!(line_text(&multi_nl, 0), "🦀\n");
        assert_eq!(line_text(&multi_nl, 1), "🚀🚀\n");
        assert_eq!(line_text(&multi_nl, 2), "🎉🎉🎉\n");
        assert_eq!(line_text(&multi_nl, 3), "");
    }

    // ============================================================
    // Differential fuzzing with length and line feed verification
    // ============================================================

    #[test]
    fn differential_fuzz_length_and_line_feeds() {
        let seeds = [42u64, 1337, 0xCAFE_BABE];

        for &seed in &seeds {
            let mut prng = SimplePrng::new(seed);
            let mut tree = PieceTree::new("");
            let mut model = ReferenceModel::new("");

            for step in 0..200 {
                let op = prng.next_range(0, 100);
                if op < 50 || model.text.is_empty() {
                    let choose_nl = prng.next_range(0, 4) == 0;
                    let snippet = if choose_nl {
                        "\n".to_string()
                    } else {
                        let len = 1 + prng.next_range(0, 5);
                        (0..len).map(|_| char::from(b'a' + prng.next_range(0, 5) as u8)).collect()
                    };
                    let pos = prng.next_range(0, model.text.len() + 1);
                    tree.insert(&snippet, pos);
                    model.insert(&snippet, pos);
                } else if op < 65 {
                    let pos = prng.next_range(0, model.text.len());
                    let del = 1 + prng.next_range(0, model.text.len() - pos);
                    tree.delete(pos, del);
                    model.delete(pos, del);
                } else if op < 85 {
                    tree.undo();
                    model.undo();
                } else {
                    tree.redo();
                    model.redo();
                }

                assert_eq!(text(&tree), model.text, "seed {} step {}: text mismatch", seed, step);
                assert_eq!(
                    tree.get_tree_len(),
                    model.text.len(),
                    "seed {} step {}: tree len mismatch",
                    seed,
                    step
                );
                assert_eq!(
                    tree.get_line_feed_count(),
                    model.text.matches('\n').count(),
                    "seed {} step {}: line feed count mismatch",
                    seed,
                    step
                );
                let violations = tree.check_invariants();
                assert!(
                    violations.is_empty(),
                    "seed {} step {}: invariants broken: {:?}",
                    seed,
                    step,
                    violations
                );
            }
        }
    }
}

