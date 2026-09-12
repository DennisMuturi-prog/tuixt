pub mod piece_tree;

#[cfg(test)]
mod tests {
    use super::piece_tree;

    fn get_text(pt: &piece_tree::PieceTree) -> String {
        let mut content = String::new();
        pt.get_text(&mut content);
        content
    }

    #[test]
    fn testing_piece_tree_operations() {
        let mut my_piece_tree = piece_tree::PieceTree::new("hello");
        my_piece_tree.insert(" ", 5);
        my_piece_tree.insert("w", 6);
        my_piece_tree.insert("r", 7);
        my_piece_tree.insert("l", 8);
        my_piece_tree.insert("d", 9);
        my_piece_tree.insert("o", 7);
        assert_eq!("hello world", get_text(&my_piece_tree));
        my_piece_tree.delete(6, 3);
        assert_eq!("hello ld", get_text(&my_piece_tree));
    }

    #[test]
    fn test_empty_initialization_and_insert() {
        let mut pt = piece_tree::PieceTree::new("");
        assert_eq!("", get_text(&pt));

        pt.insert("hello", 0);
        assert_eq!("hello", get_text(&pt));

        pt.insert(" world", 5);
        assert_eq!("hello world", get_text(&pt));
    }

    #[test]
    fn test_prepend_and_append() {
        let mut pt = piece_tree::PieceTree::new("middle");
        assert_eq!("middle", get_text(&pt));

        // Prepend
        pt.insert("start-", 0);
        assert_eq!("start-middle", get_text(&pt));

        // Append
        pt.insert("-end", 12);
        assert_eq!("start-middle-end", get_text(&pt));
    }

    #[test]
    fn test_insert_in_middle_of_piece() {
        let mut pt = piece_tree::PieceTree::new("ac");
        // Insert between 'a' and 'c'
        pt.insert("b", 1);
        assert_eq!("abc", get_text(&pt));

        // Insert another string inside newly inserted piece
        pt.insert("123", 2);
        assert_eq!("ab123c", get_text(&pt));
    }

    #[test]
    fn test_delete_from_start() {
        let mut pt = piece_tree::PieceTree::new("Hello World");
        pt.delete(0, 6);
        assert_eq!("World", get_text(&pt));
    }

    #[test]
    fn test_delete_from_end() {
        let mut pt = piece_tree::PieceTree::new("Hello World");
        pt.delete(5, 6);
        assert_eq!("Hello", get_text(&pt));
    }

    #[test]
    fn test_delete_from_middle() {
        let mut pt = piece_tree::PieceTree::new("Hello Beautiful World");
        pt.delete(5, 10);
        assert_eq!("Hello World", get_text(&pt));
    }

    #[test]
    fn test_delete_entire_buffer() {
        let mut pt = piece_tree::PieceTree::new("Full deletion test");
        pt.delete(0, 20);
        assert_eq!("", get_text(&pt));

        // Should be able to insert again after deleting everything
        pt.insert("Brand new text", 0);
        assert_eq!("Brand new text", get_text(&pt));
    }

    #[test]
    fn test_consecutive_deletes() {
        let mut pt = piece_tree::PieceTree::new("abcdefghij");
        pt.delete(2, 2); // remove "cd" -> "abefghij"
        assert_eq!("abefghij", get_text(&pt));

        pt.delete(4, 2); // remove "gh" -> "abefij"
        assert_eq!("abefij", get_text(&pt));

        pt.delete(0, 1); // remove "a" -> "befij"
        assert_eq!("befij", get_text(&pt));

        pt.delete(4, 1); // remove "j" -> "befi"
        assert_eq!("befi", get_text(&pt));
    }

    #[test]
    fn test_simulated_typing_and_backspacing() {
        let mut pt = piece_tree::PieceTree::new("");

        // Typing: "The quick brown fx"
        let initial_text = "The quick brown fx";
        for (i, ch) in initial_text.chars().enumerate() {
            pt.insert(&ch.to_string(), i);
        }
        assert_eq!("The quick brown fx", get_text(&pt));

        // Backspace 'x' and 'f'
        pt.delete(17, 1); // remove 'x'
        assert_eq!("The quick brown f", get_text(&pt));
        pt.delete(16, 1); // remove 'f'
        assert_eq!("The quick brown ", get_text(&pt));

        // Type correct word: "fox"
        pt.insert("f", 16);
        pt.insert("o", 17);
        pt.insert("x", 18);
        assert_eq!("The quick brown fox", get_text(&pt));
    }

    #[test]
    fn test_multiline_editing() {
        let mut pt = piece_tree::PieceTree::new("line 1\nline 2\nline 3\n");
        assert_eq!("line 1\nline 2\nline 3\n", get_text(&pt));

        // Insert a new line between line 1 and line 2
        // "line 1\n" is 7 bytes
        pt.insert("line 1.5\n", 7);
        assert_eq!("line 1\nline 1.5\nline 2\nline 3\n", get_text(&pt));

        // Delete "line 2\n" (9 bytes starting after "line 1.5\n", offset 16)
        pt.delete(16, 7);
        assert_eq!("line 1\nline 1.5\nline 3\n", get_text(&pt));
    }

    #[test]
    fn test_interleaved_inserts_and_deletes() {
        let mut pt = piece_tree::PieceTree::new("Initial document.");
        pt.insert(" start:", 0);
        assert_eq!(" start:Initial document.", get_text(&pt));

        pt.delete(0, 1);
        assert_eq!("start:Initial document.", get_text(&pt));

        pt.insert(" draft", 23);
        assert_eq!("start:Initial document. draft", get_text(&pt));

        pt.delete(6, 8); // remove "Initial "
        assert_eq!("start:document. draft", get_text(&pt));
    }

    #[test]
    fn test_zero_length_operations() {
        let mut pt = piece_tree::PieceTree::new("Sample text");
        pt.insert("", 0);
        pt.insert("", 5);
        pt.insert("", 11);
        assert_eq!("Sample text", get_text(&pt));

        pt.delete(0, 0);
        pt.delete(5, 0);
        assert_eq!("Sample text", get_text(&pt));
    }

    #[test]
    fn test_consecutive_inserts_at_same_cursor_position() {
        let mut pt = piece_tree::PieceTree::new("base");
        // Insert repeatedly at position 0 (prepending in chunks)
        pt.insert("c", 0);
        pt.insert("b", 0);
        pt.insert("a", 0);
        assert_eq!("abcbase", get_text(&pt));

        // Insert repeatedly at end
        pt.insert("1", 7);
        pt.insert("2", 8);
        pt.insert("3", 9);
        assert_eq!("abcbase123", get_text(&pt));
    }

    #[test]
    fn test_repeated_backspaces_until_empty() {
        let text = "Text editor backspace test";
        let mut pt = piece_tree::PieceTree::new(text);
        let mut len = text.len();

        while len > 0 {
            pt.delete(len - 1, 1);
            len -= 1;
            assert_eq!(&text[..len], &get_text(&pt));
        }

        assert_eq!("", get_text(&pt));

        // Insert into the emptied tree
        pt.insert("Rebuilt", 0);
        assert_eq!("Rebuilt", get_text(&pt));
    }

    #[test]
    fn test_repeated_sequential_edits() {
        let mut pt = piece_tree::PieceTree::new("");
        for i in 0..50 {
            pt.insert(&format!("{} ", i), get_text(&pt).len());
        }

        let mut expected = String::new();
        for i in 0..50 {
            expected.push_str(&format!("{} ", i));
        }
        assert_eq!(expected, get_text(&pt));

        // Delete from the front incrementally
        for _ in 0..10 {
            let space_idx = get_text(&pt).find(' ').unwrap();
            pt.delete(0, space_idx + 1);
        }

        let mut remaining_expected = String::new();
        for i in 10..50 {
            remaining_expected.push_str(&format!("{} ", i));
        }
        assert_eq!(remaining_expected, get_text(&pt));
    }

    #[test]
    fn test_differential_model_with_reference_string() {
        // Deterministic pseudo-random operations simulating user typing, selection, deletion
        let mut pt = piece_tree::PieceTree::new("hello world");
        let mut reference = String::from("hello world");

        let mut rng: u64 = 0x123456789ABCDEF0;
        let mut next_rand = || -> u64 {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            rng
        };

        for step in 0..200 {
            let op = next_rand() % 3;
            let current_len = reference.len();

            if op == 0 || current_len == 0 {
                // Insert
                let pos = if current_len == 0 { 0 } else { (next_rand() as usize) % (current_len + 1) };
                let snippet = format!("_{}_", step);
                pt.insert(&snippet, pos);
                reference.insert_str(pos, &snippet);
            } else {
                // Delete
                let pos = (next_rand() as usize) % current_len;
                let max_len = current_len - pos;
                let del_len = ((next_rand() as usize) % max_len.min(8)) + 1;
                pt.delete(pos, del_len);
                reference.replace_range(pos..pos + del_len, "");
            }

            assert_eq!(
                reference,
                get_text(&pt),
                "Mismatch at step {}: reference len = {}, pt len = {}",
                step,
                reference.len(),
                get_text(&pt).len()
            );
        }
    }

    #[test]
    fn test_delete_on_empty_tree() {
        let mut pt = piece_tree::PieceTree::new("");
        pt.delete(0, 5);
        assert_eq!("", get_text(&pt));
        pt.delete(10, 5);
        assert_eq!("", get_text(&pt));
    }

    #[test]
    fn test_out_of_bounds_deletion() {
        let mut pt = piece_tree::PieceTree::new("hello");
        // Deleting from an index past the end
        pt.delete(10, 2);
        assert_eq!("hello", get_text(&pt));

        // Deleting length that extends past the end
        pt.delete(3, 50);
        assert_eq!("hel", get_text(&pt));
    }

    #[test]
    fn test_out_of_bounds_insertion() {
        let mut pt = piece_tree::PieceTree::new("hello");
        // Inserting at an index well past the end
        pt.insert(" world", 100);
        assert_eq!("hello world", get_text(&pt));
    }

    #[test]
    fn test_delete_spanning_multiple_pieces() {
        let mut pt = piece_tree::PieceTree::new("ABC");
        pt.insert("DEF", 3);
        pt.insert("GHI", 6);
        pt.insert("JKL", 9);
        assert_eq!("ABCDEFGHIJKL", get_text(&pt));

        // Delete from 'B' (index 1) to 'K' (index 10, length 10)
        // Should leave "AL"
        pt.delete(1, 10);
        assert_eq!("AL", get_text(&pt));
    }

    #[test]
    fn test_unicode_multibyte_characters() {
        // "🦀" is 4 bytes: [0xF0, 0x9F, 0xA6, 0x80]
        let mut pt = piece_tree::PieceTree::new("hello 🦀 world");
        assert_eq!("hello 🦀 world", get_text(&pt));

        // Insert at char boundary (byte 11, after "hello 🦀 ")
        pt.insert("beautiful ", 11);
        assert_eq!("hello 🦀 beautiful world", get_text(&pt));

        // Delete the emoji and following space (byte 6, length 5)
        pt.delete(6, 5);
        assert_eq!("hello beautiful world", get_text(&pt));
    }



    #[test]
    fn test_unicode_invalid_byte_split() {
        // Test what happens if split/delete/insert lands on a non-char boundary
        let mut pt = piece_tree::PieceTree::new("🦀");
        // "🦀" has byte length 4. Splitting at byte index 2 is invalid UTF-8.
        pt.insert("X", 2);
        let mut text = String::new();
        // Since Rust String indexing requires valid UTF-8 char boundaries,
        // slicing non-char boundary in get_text should panic or fail.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            pt.get_text(&mut text);
        }));
        // If the implementation is purely byte-based, slicing at byte 2 of a 4-byte UTF-8 char panics:
        assert!(result.is_err(), "Expected panic when slicing non-char boundary in UTF-8 string");
    }

    #[test]
    fn test_alternating_prepend_append_balance() {
        let mut pt = piece_tree::PieceTree::new("init");
        for i in 0..100 {
            pt.insert(&format!("<{}>", i), 0);
            let len = get_text(&pt).len();
            pt.insert(&format!("[{}]", i), len);
        }
        let text = get_text(&pt);
        assert!(text.starts_with("<99>"));
        assert!(text.ends_with("[99]"));
    }

    #[test]
    fn test_delete_all_from_front_one_by_one() {
        let text = "abcdefghijklmnopqrstuvwxyz";
        let mut pt = piece_tree::PieceTree::new(text);
        let mut current = text.to_string();
        while !current.is_empty() {
            pt.delete(0, 1);
            current.remove(0);
            assert_eq!(current, get_text(&pt));
        }
        assert_eq!("", get_text(&pt));
    }

    #[test]
    fn test_fuzz_large_differential_with_large_deletions() {
        let mut pt = piece_tree::PieceTree::new("abcdefghijklmnopqrstuvwxyz");
        let mut reference = String::from("abcdefghijklmnopqrstuvwxyz");

        let mut rng: u64 = 0xCAFEBABEDEADBEEF;
        let mut next_rand = || -> u64 {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            rng
        };

        for step in 0..1000 {
            let current_len = reference.len();
            let op = next_rand() % 3;

            if op == 0 || current_len == 0 {
                // Insert random snippet at random position
                let pos = if current_len == 0 { 0 } else { (next_rand() as usize) % (current_len + 1) };
                let snippet = format!("({}:{})", step, next_rand() % 100);
                pt.insert(&snippet, pos);
                reference.insert_str(pos, &snippet);
            } else {
                // Delete: allow large deletions spanning multiple pieces
                let pos = (next_rand() as usize) % current_len;
                let max_len = current_len - pos;
                // Allow deleting up to max_len (large chunks)
                let del_len = ((next_rand() as usize) % max_len) + 1;
                pt.delete(pos, del_len);
                reference.replace_range(pos..pos + del_len, "");
            }

            assert_eq!(
                reference,
                get_text(&pt),
                "Mismatch at step {}: reference len = {}, pt len = {}",
                step,
                reference.len(),
                get_text(&pt).len()
            );
        }
    }

    // ================= regression tests for the known tree bugs =================
    //
    // These drive the public API only, and they FAIL on the current revision.
    // They are written to pass once the tree invariant is restored, so they
    // double as the acceptance test for a fix.
    //
    // The invariant that is broken: in a red-black tree both children of every
    // node must have the same black height, and a red node may not have a red
    // child. Inserting *inside* an existing piece takes the middle branch of
    // `PieceTree::insert_node`: it splits the piece in two, gives both halves
    // the color of the piece being split, and hangs them where that piece's
    // black leaves used to be, while the new middle node keeps the inserted
    // node's color (red). If the split piece is black, each half is a black
    // node sitting where a black leaf was, so those paths gain a black; and
    // the replacement node drops the black that the split piece contributed.
    // Nothing red is involved in either change, so `rebalance` -- which only
    // looks for a red child of a red node -- never sees the violation. The
    // text stays correct (in-order traversal ignores colors), which is why the
    // differential tests cannot catch it. It surfaces later, inside `delete`,
    // when joining subtrees uses a black height the tree can no longer honor.
    //
    // Each sequence below was minimized by greedy removal from a 2000-op
    // randomized run against the current revision. Each one panics today.

    #[derive(Debug)]
    enum Edit {
        Ins(&'static str, usize),
        Del(usize, usize),
    }

    /// Apply an edit sequence to a fresh tree while keeping a reference model
    /// in sync, the same way the differential tests do. A panic anywhere
    /// inside the piece tree fails the test, which is the point of these
    /// tests; the final assert also guards that the document text is right.
    fn apply_edits(initial: &str, edits: &[Edit]) -> String {
        let mut pt = piece_tree::PieceTree::new(initial);
        let mut reference = String::from(initial);
        for edit in edits {
            match *edit {
                Edit::Ins(text, at) => {
                    pt.insert(text, at);
                    reference.insert_str(at, text);
                }
                Edit::Del(at, len) => {
                    pt.delete(at, len);
                    reference.replace_range(at..at + len, "");
                }
            }
        }
        assert_eq!(reference, get_text(&pt), "document text diverged");
        reference
    }

    /// Panic site 1 of 2: `insert_at_leftmost_with_target_black_height`
    /// descends past the black leaf and unwraps it (src/piece_tree.rs:354),
    /// reached from `join` <- `split` <- `delete`. 8 edits from an empty
    /// document are enough.
    #[test]
    fn test_delete_should_not_walk_past_black_leaf() {
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
    }

    /// Same panic site, reached from a non-empty initial document and through a
    /// different mix of edits (8 edits).
    #[test]
    fn test_delete_should_not_walk_past_black_leaf_nonempty_document() {
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

    /// Panic site 2 of 2: a negative-black marker survives an operation and is
    /// reddened again, which `Color::minus_black` forbids
    /// (src/piece_tree.rs:1032). 7 edits.
    #[test]
    fn test_delete_should_not_redden_a_negative_black() {
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

    /// Like `apply_edits`, but asserts the tree invariants after every single
    /// operation, so the first edit that breaks the tree is named in the
    /// failure message. This is the test-side counterpart of the tree's own
    /// invariant: colors and black heights never affect the document text, so
    /// a text comparison alone cannot see a broken tree.
    fn apply_edits_checked(initial: &str, edits: &[Edit]) -> String {
        let mut pt = piece_tree::PieceTree::new(initial);
        let mut reference = String::from(initial);
        for (i, edit) in edits.iter().enumerate() {
            match *edit {
                Edit::Ins(text, at) => {
                    pt.insert(text, at);
                    reference.insert_str(at, text);
                }
                Edit::Del(at, len) => {
                    pt.delete(at, len);
                    reference.replace_range(at..at + len, "");
                }
            }
            assert_eq!(
                reference,
                get_text(&pt),
                "document text diverged after edit #{} ({:?})",
                i,
                edit
            );
            let report = pt.invariant_report();
            assert!(
                report.is_ok(),
                "red-black invariants broken after edit #{} ({:?}):\n{}",
                i,
                edit,
                report.summary()
            );
        }
        reference
    }

    /// The bug at its source: an insertion that lands inside an existing piece
    /// splits that piece in two and hangs the halves - colored like the piece
    /// that was split - where its black leaves used to be, which changes the
    /// black height of those paths. Nothing red is involved, so the repair
    /// routine never notices, and the document text stays correct.
    ///
    /// Same 8-edit sequence as `test_delete_should_not_walk_past_black_leaf`,
    /// which shows the crash it eventually causes; here the test stops at the
    /// first edit that breaks the tree.
    #[test]
    fn test_insert_inside_a_piece_breaks_the_tree_invariants() {
        apply_edits_checked(
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
    }

    /// The same corruption occurs on the operation stream of
    /// `test_fuzz_large_differential_with_large_deletions`, using its exact
    /// LCG. That test compares document text only, so it passes while the tree
    /// is invalid; here the invariants are asserted after every step.
    #[test]
    fn test_fuzz_sequence_keeps_the_tree_invariants() {
        let mut pt = piece_tree::PieceTree::new("abcdefghijklmnopqrstuvwxyz");
        let mut reference = String::from("abcdefghijklmnopqrstuvwxyz");

        let mut rng: u64 = 0xCAFEBABEDEADBEEF;
        let mut next_rand = || -> u64 {
            rng = rng
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            rng
        };

        for step in 0..1000 {
            let current_len = reference.len();
            let op = next_rand() % 3;
            let description: String;
            if op == 0 || current_len == 0 {
                let pos = if current_len == 0 {
                    0
                } else {
                    (next_rand() as usize) % (current_len + 1)
                };
                let snippet = format!("({}:{})", step, next_rand() % 100);
                description = format!("insert({:?}, {})", snippet, pos);
                pt.insert(&snippet, pos);
                reference.insert_str(pos, &snippet);
            } else {
                let pos = (next_rand() as usize) % current_len;
                let max_len = current_len - pos;
                let del_len = ((next_rand() as usize) % max_len) + 1;
                description = format!("delete({}, {})", pos, del_len);
                pt.delete(pos, del_len);
                reference.replace_range(pos..pos + del_len, "");
            }

            assert_eq!(
                reference,
                get_text(&pt),
                "document text diverged at step {} ({})",
                step,
                description
            );
            let report = pt.invariant_report();
            assert!(
                report.is_ok(),
                "red-black invariants broken after step {} ({}):\n{}",
                step,
                description,
                report.summary()
            );
        }
    }

    // ================ regressions found while reviewing the insert fix ================
    //
    // The commit that fixed mid-piece insertion left the delete path alone: a
    // delete can still turn a valid tree into one that breaks the red-black
    // invariants while leaving the document text correct. The tests below use
    // `apply_edits_checked`, which asserts the invariants after every single
    // edit, so each failure names the operation that broke the tree.

    /// A red-red violation was left behind by the final delete. Every earlier
    /// edit leaves the tree valid, so this pins the break on `delete`, not on
    /// `insert`. Minimised from a randomised run; `get_text` stays correct even
    /// while the tree is broken.
    #[test]
    fn test_delete_can_leave_a_red_red_violation() {
        apply_edits_checked(
            "",
            &[
                Edit::Ins("dca", 0),
                Edit::Ins("abd", 1),
                Edit::Ins("da", 4),
                Edit::Ins("c", 1),
                Edit::Ins("bccc", 2),
                Edit::Del(11, 1),
                Edit::Ins("c", 10),
                Edit::Ins("baa", 9),
                Edit::Ins("ab", 1),
                Edit::Ins("caa", 7),
                Edit::Del(13, 1),
            ],
        );
    }

    /// A zero-length insertion used to be treated as real work: it split the
    /// piece it landed in and left a zero-length piece behind, and the delete
    /// that followed then left a negative-black node and inconsistent black
    /// heights. Dropping the `Ins("", 0)` from this sequence keeps the tree
    /// valid, which is what pinned the zero-length piece as the trigger;
    /// `insert` now returns early for empty input.
    #[test]
    fn test_zero_length_insert_then_delete_breaks_the_tree() {
        apply_edits_checked(
            "",
            &[
                Edit::Ins("xyyxyy", 0),
                Edit::Ins("", 0),
                Edit::Ins("yyyyxx", 2),
                Edit::Ins("xxyyyy", 1),
                Edit::Del(11, 1),
            ],
        );
    }

    struct Lcg(u64);

    impl Lcg {
        fn next(&mut self) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            self.0 >> 11
        }
        fn below(&mut self, n: usize) -> usize {
            if n == 0 {
                0
            } else {
                (self.next() as usize) % n
            }
        }
    }

    /// One randomised run over a mixed insert/delete workload. Returns the
    /// first operation after which the document text diverges or the tree
    /// invariants break.
    fn fuzz_run(seed: u64, steps: usize, inserts_only: bool) -> Result<(), String> {
        let mut rng = Lcg(seed);
        let mut pt = piece_tree::PieceTree::new("");
        let mut reference = String::new();

        for step in 0..steps {
            let len = reference.len();
            let op = rng.next() % 100;
            let description: String;
            if inserts_only || op < 45 || len == 0 {
                let n = 1 + rng.below(4);
                let snippet: String = (0..n)
                    .map(|_| char::from(b'a' + rng.below(4) as u8))
                    .collect();
                let pos = rng.below(len + 1);
                description = format!("insert({:?}, {})", snippet, pos);
                pt.insert(&snippet, pos);
                reference.insert_str(pos, &snippet);
            } else if op < 60 {
                description = format!("delete(0, {})", len);
                pt.delete(0, len);
                reference.clear();
            } else {
                let pos = rng.below(len);
                let del = 1 + rng.below(len - pos);
                description = format!("delete({}, {})", pos, del);
                pt.delete(pos, del);
                reference.replace_range(pos..pos + del, "");
            }

            if reference != get_text(&pt) {
                return Err(format!(
                    "step {step} ({description}): document text diverged\n  reference: {:?}\n  tree:      {:?}",
                    reference,
                    get_text(&pt)
                ));
            }
            let report = pt.invariant_report();
            if !report.is_ok() {
                return Err(format!("step {step} ({description}):\n{}", report.summary()));
            }
        }
        Ok(())
    }

    fn run_seeds(seeds: u64, steps: usize, inserts_only: bool) -> Vec<String> {
        let mut failures = Vec::new();
        for seed in 1..=seeds {
            let seed = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
            if let Err(reason) = fuzz_run(seed, steps, inserts_only) {
                failures.push(format!("seed {seed}: {reason}"));
            }
        }
        failures
    }

    /// The in-repo invariant fuzz pins a single LCG stream, and that stream
    /// happens to pass. These seeds do not: a delete in the middle one of them
    /// leaves a red-red violation behind.
    #[test]
    fn test_fuzz_many_seeds_keep_the_tree_invariants() {
        let failures = run_seeds(60, 200, false);
        assert!(
            failures.is_empty(),
            "{} of 60 seeds end with a broken tree; first 3:\n{}\n",
            failures.len(),
            failures
                .iter()
                .take(3)
                .cloned()
                .collect::<Vec<_>>()
                .join("\n")
        );
    }

    /// Insert-only workload: after the mid-piece insertion fix this path is
    /// sound, so this guards the rewrite rather than exposing a live bug.
    #[test]
    fn test_fuzz_insert_only_keeps_the_tree_invariants() {
        let failures = run_seeds(60, 200, true);
        assert!(
            failures.is_empty(),
            "{} of 60 insert-only seeds end with a broken tree; first 3:\n{}",
            failures.len(),
            failures
                .iter()
                .take(3)
                .cloned()
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}

