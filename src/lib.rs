pub mod piece_tree;
mod piece_tree_test;
pub mod app;

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

    // ============================ undo / redo ============================
    //
    // The stacks store whole tree roots (the tree is persistent, so an old root
    // stays valid as `add` only grows), which means these tests can drive undo
    // and redo purely through the public API and compare document text.

    #[test]
    fn test_undo_redo_basic_insert() {
        let mut pt = piece_tree::PieceTree::new("hello");
        pt.insert("X", 0);
        assert_eq!("Xhello", get_text(&pt));

        pt.undo();
        assert_eq!("hello", get_text(&pt));

        pt.redo();
        assert_eq!("Xhello", get_text(&pt));
    }

    #[test]
    fn test_undo_redo_with_empty_history_is_noop() {
        let mut pt = piece_tree::PieceTree::new("hello");

        pt.undo();
        assert_eq!("hello", get_text(&pt));

        pt.redo();
        assert_eq!("hello", get_text(&pt));
    }

    /// The very first insert into an empty document has no root to snapshot,
    /// so it must record the empty state explicitly for undo.
    #[test]
    fn test_undo_first_insert_into_empty_tree() {
        let mut pt = piece_tree::PieceTree::new("");
        pt.insert("a", 0);
        assert_eq!("a", get_text(&pt));

        pt.undo();
        assert_eq!("", get_text(&pt));

        pt.redo();
        assert_eq!("a", get_text(&pt));
    }

    /// Undo the delete that emptied the document, then redo it. The empty
    /// state has to be recorded too, otherwise redo cannot reinstate the delete.
    #[test]
    fn test_undo_delete_all() {
        let mut pt = piece_tree::PieceTree::new("hello");
        pt.delete(0, 5);
        assert_eq!("", get_text(&pt));

        pt.undo();
        assert_eq!("hello", get_text(&pt));

        pt.redo();
        assert_eq!("", get_text(&pt));
    }

    #[test]
    fn test_new_insert_after_undo_clears_redo() {
        let mut pt = piece_tree::PieceTree::new("hello");
        pt.insert("X", 0);
        pt.undo();
        assert_eq!("hello", get_text(&pt));

        pt.insert("Y", 5);
        assert_eq!("helloY", get_text(&pt));

        pt.redo();
        assert_eq!("helloY", get_text(&pt));
    }

    #[test]
    fn test_delete_after_undo_clears_redo() {
        let mut pt = piece_tree::PieceTree::new("hello");
        pt.insert("X", 0);
        pt.undo();
        assert_eq!("hello", get_text(&pt));

        pt.delete(0, 1);
        assert_eq!("ello", get_text(&pt));

        pt.redo();
        assert_eq!("ello", get_text(&pt));
    }

    /// Deleting the whole buffer takes an early return, which used to skip the
    /// `redo_stack.clear()`; a stale redo would then resurrect the old document.
    #[test]
    fn test_delete_all_after_undo_clears_redo() {
        let mut pt = piece_tree::PieceTree::new("hello");
        pt.insert("X", 0);
        pt.undo();
        assert_eq!("hello", get_text(&pt));

        pt.delete(0, 5);
        assert_eq!("", get_text(&pt));

        pt.redo();
        assert_eq!("", get_text(&pt));
    }

    /// A redo must put the state it left onto the undo stack; when that push
    /// went to the wrong stack, a second undo did nothing.
    #[test]
    fn test_undo_after_redo_restores_previous_state() {
        let mut pt = piece_tree::PieceTree::new("hello");
        pt.insert("X", 0);
        pt.undo();
        pt.redo();
        assert_eq!("Xhello", get_text(&pt));

        pt.undo();
        assert_eq!("hello", get_text(&pt));

        pt.redo();
        assert_eq!("Xhello", get_text(&pt));
    }

    #[test]
    fn test_multiple_undo_and_redo_walk_the_whole_history() {
        let mut pt = piece_tree::PieceTree::new("mid");
        pt.insert("A", 0);
        assert_eq!("Amid", get_text(&pt));
        pt.insert("B", 4);
        assert_eq!("AmidB", get_text(&pt));
        pt.delete(1, 2);
        assert_eq!("AdB", get_text(&pt));

        // Walk the history back to the initial document.
        pt.undo();
        assert_eq!("AmidB", get_text(&pt));
        pt.undo();
        assert_eq!("Amid", get_text(&pt));
        pt.undo();
        assert_eq!("mid", get_text(&pt));
        pt.undo();
        assert_eq!("mid", get_text(&pt));

        // And forward again to the final edit.
        pt.redo();
        assert_eq!("Amid", get_text(&pt));
        pt.redo();
        assert_eq!("AmidB", get_text(&pt));
        pt.redo();
        assert_eq!("AdB", get_text(&pt));
        pt.redo();
        assert_eq!("AdB", get_text(&pt));
    }

    /// Randomised differential test: a three-stack reference model (past /
    /// current / future) drives the same inserts, deletes, undos and redos as
    /// the piece tree. Every operation must leave the document identical, and
    /// the model's branch-invalidation rule (a forward edit clears the future)
    /// has to match the tree's `redo_stack.clear()`.
    #[test]
    fn test_fuzz_undo_redo_matches_reference() {
        let mut rng = Lcg(0xDEAD_BEEF_CAFE_F00D);
        let mut pt = piece_tree::PieceTree::new("seed");
        let mut current = String::from("seed");
        let mut past: Vec<String> = Vec::new();
        let mut future: Vec<String> = Vec::new();

        for step in 0..1000 {
            let op = rng.next() % 100;
            let description: String;

            if op < 40 {
                let n = 1 + rng.below(4);
                let snippet: String = (0..n)
                    .map(|_| char::from(b'a' + rng.below(4) as u8))
                    .collect();
                let pos = rng.below(current.len() + 1);
                description = format!("insert({:?}, {})", snippet, pos);
                past.push(current.clone());
                pt.insert(&snippet, pos);
                current.insert_str(pos, &snippet);
                future.clear();
            } else if op < 65 && !current.is_empty() {
                let pos = rng.below(current.len());
                let del = 1 + rng.below(current.len() - pos);
                description = format!("delete({}, {})", pos, del);
                past.push(current.clone());
                pt.delete(pos, del);
                current.replace_range(pos..pos + del, "");
                future.clear();
            } else if op < 85 {
                description = "undo".to_string();
                if let Some(prev) = past.pop() {
                    future.push(current.clone());
                    current = prev;
                }
                pt.undo();
            } else {
                description = "redo".to_string();
                if let Some(next) = future.pop() {
                    past.push(current.clone());
                    current = next;
                }
                pt.redo();
            }

            assert_eq!(
                current,
                get_text(&pt),
                "step {} ({}): undo/redo state diverged",
                step,
                description
            );
        }
    }

    // ============================ get_sub_text ============================
    //
    // `get_sub_text` copies `length` bytes starting at `start_index` into the
    // caller's buffer, replacing whatever was there first. As with `delete`,
    // the requested range is clamped to the document: a start past the end
    // yields an empty buffer, and a length that runs past the end stops at the
    // end. Indices are byte offsets, matching the rest of the piece tree.

    fn get_sub_text(pt: &piece_tree::PieceTree, start_index: usize, length: usize) -> String {
        let mut content = String::new();
        pt.get_sub_text(&mut content, start_index, length);
        content
    }

    #[test]
    fn test_get_sub_text_basic_slice() {
        let pt = piece_tree::PieceTree::new("hello world");
        assert_eq!("hello", get_sub_text(&pt, 0, 5));
        assert_eq!("world", get_sub_text(&pt, 6, 5));
        assert_eq!("lo wo", get_sub_text(&pt, 3, 5));
    }

    #[test]
    fn test_get_sub_text_whole_document() {
        let pt = piece_tree::PieceTree::new("hello world");
        assert_eq!("hello world", get_sub_text(&pt, 0, 11));
    }

    #[test]
    fn test_get_sub_text_single_char_and_boundaries() {
        let pt = piece_tree::PieceTree::new("abcdef");
        assert_eq!("a", get_sub_text(&pt, 0, 1));
        assert_eq!("b", get_sub_text(&pt, 1, 1));
        assert_eq!("f", get_sub_text(&pt, 5, 1));
        assert_eq!("abcdef", get_sub_text(&pt, 0, 6));
    }

    #[test]
    fn test_get_sub_text_zero_length_is_empty() {
        let pt = piece_tree::PieceTree::new("hello");
        assert_eq!("", get_sub_text(&pt, 0, 0));
        assert_eq!("", get_sub_text(&pt, 2, 0));
        assert_eq!("", get_sub_text(&pt, 5, 0));
    }

    #[test]
    fn test_get_sub_text_length_past_end_clamps() {
        let pt = piece_tree::PieceTree::new("hello");
        assert_eq!("hello", get_sub_text(&pt, 0, 50));
        assert_eq!("llo", get_sub_text(&pt, 2, 50));
        assert_eq!("o", get_sub_text(&pt, 4, 50));
    }

    #[test]
    fn test_get_sub_text_start_at_or_past_end_is_empty() {
        let pt = piece_tree::PieceTree::new("hello");
        assert_eq!("", get_sub_text(&pt, 5, 3));
        assert_eq!("", get_sub_text(&pt, 100, 5));
    }

    #[test]
    fn test_get_sub_text_on_empty_document() {
        let pt = piece_tree::PieceTree::new("");
        assert_eq!("", get_sub_text(&pt, 0, 0));
        assert_eq!("", get_sub_text(&pt, 0, 10));
        assert_eq!("", get_sub_text(&pt, 10, 10));
    }

    #[test]
    fn test_get_sub_text_replaces_existing_buffer_contents() {
        let pt = piece_tree::PieceTree::new("hello world");
        let mut content = String::from("stale contents that must be replaced");
        pt.get_sub_text(&mut content, 6, 5);
        assert_eq!("world", content);
    }

    #[test]
    fn test_get_sub_text_replaces_with_empty_when_out_of_range() {
        let pt = piece_tree::PieceTree::new("hello");
        let mut content = String::from("stale");
        pt.get_sub_text(&mut content, 100, 5);
        assert_eq!("", content);
    }

    #[test]
    fn test_get_sub_text_spans_pieces_after_inserts() {
        let mut pt = piece_tree::PieceTree::new("ABC");
        pt.insert("DEF", 3);
        pt.insert("GHI", 6);
        assert_eq!("ABCDEFGHI", get_text(&pt));
        assert_eq!("CDEFGH", get_sub_text(&pt, 2, 6));
        assert_eq!("ABCDEFGHI", get_sub_text(&pt, 0, 9));
        assert_eq!("", get_sub_text(&pt, 4, 0));
    }

    #[test]
    fn test_get_sub_text_after_delete() {
        let mut pt = piece_tree::PieceTree::new("Hello Beautiful World");
        pt.delete(5, 10);
        assert_eq!("Hello World", get_text(&pt));
        assert_eq!("Hello", get_sub_text(&pt, 0, 5));
        assert_eq!("World", get_sub_text(&pt, 6, 5));
        assert_eq!(" World", get_sub_text(&pt, 5, 6));
    }

    #[test]
    fn test_get_sub_text_after_prepend_and_append() {
        let mut pt = piece_tree::PieceTree::new("middle");
        pt.insert("start-", 0);
        pt.insert("-end", 12);
        assert_eq!("start-middle-end", get_text(&pt));
        assert_eq!("start", get_sub_text(&pt, 0, 5));
        assert_eq!("middle", get_sub_text(&pt, 6, 6));
        assert_eq!("-", get_sub_text(&pt, 5, 1));
        assert_eq!("end", get_sub_text(&pt, 13, 3));
    }

    #[test]
    fn test_get_sub_text_after_undo_and_redo() {
        let mut pt = piece_tree::PieceTree::new("hello");
        pt.insert("X", 0);
        assert_eq!("Xhello", get_text(&pt));
        assert_eq!("Xhell", get_sub_text(&pt, 0, 5));

        pt.undo();
        assert_eq!("hello", get_text(&pt));
        assert_eq!("ell", get_sub_text(&pt, 1, 3));

        pt.redo();
        assert_eq!("Xhello", get_text(&pt));
        assert_eq!("ello", get_sub_text(&pt, 2, 4));
    }

    #[test]
    fn test_get_sub_text_unicode_multibyte_characters() {
        // "🦀" is 4 bytes: [0xF0, 0x9F, 0xA6, 0x80].
        let pt = piece_tree::PieceTree::new("hello 🦀 world");
        assert_eq!("hello", get_sub_text(&pt, 0, 5));
        assert_eq!("🦀", get_sub_text(&pt, 6, 4));
        assert_eq!(" 🦀 ", get_sub_text(&pt, 5, 6));
        assert_eq!("world", get_sub_text(&pt, 11, 5));
    }

    #[test]
    fn test_get_sub_text_fuzz_matches_reference_slice() {
        let mut pt = piece_tree::PieceTree::new("the quick brown fox");
        let mut reference = String::from("the quick brown fox");

        let mut rng = Lcg(0x0123_4567_89AB_CDEF);

        for step in 0..500 {
            if reference.is_empty() {
                let snippet = format!("_{}_", step);
                pt.insert(&snippet, 0);
                reference.insert_str(0, &snippet);
            }

            // Ask for arbitrary (often out-of-range) windows; the tree must
            // clamp exactly the way slicing a bounded range does.
            let start = rng.below(reference.len() + 3);
            let length = rng.below(reference.len() + 3);
            let clamped_start = start.min(reference.len());
            let clamped_end = (clamped_start + length).min(reference.len());
            let expected = reference[clamped_start..clamped_end].to_string();

            assert_eq!(
                expected,
                get_sub_text(&pt, start, length),
                "step {}: get_sub_text({}, {}) on {:?}",
                step,
                start,
                length,
                reference
            );

            // Keep the document churning so windows routinely cross pieces.
            if rng.next() % 2 == 0 {
                let pos = rng.below(reference.len() + 1);
                let snippet = format!("({})", step);
                pt.insert(&snippet, pos);
                reference.insert_str(pos, &snippet);
            } else {
                let pos = rng.below(reference.len());
                let del = 1 + rng.below(reference.len() - pos);
                pt.delete(pos, del);
                reference.replace_range(pos..pos + del, "");
            }
        }
    }

    // ============================ get_line_text ============================
    //
    // `get_line_text` copies the content of line `line_number` (0-indexed) into
    // the caller's buffer, replacing whatever was there before.
    //
    // Specifications & Invariants:
    // - Line numbers are 0-indexed (the first line is line 0).
    // - The line text includes the trailing newline (`\n` or `\r\n`) if present in the buffer.
    // - For an empty document `""`, line 0 yields `""`, and any line >= 1 is out of bounds (`""`).
    // - Out-of-bounds line numbers clear `content` to an empty string `""` without panicking.
    // - `content` is always cleared before writing, discarding any preexisting contents.

    fn get_line_text(pt: &piece_tree::PieceTree, line_number: usize) -> String {
        let mut content = String::new();
        pt.get_line_text(line_number, &mut content);
        content
    }

    fn split_lines_reference(s: &str) -> Vec<String> {
        if s.is_empty() {
            return vec![String::new()];
        }
        let mut lines = Vec::new();
        let mut start = 0;
        for (i, b) in s.bytes().enumerate() {
            if b == b'\n' {
                lines.push(s[start..=i].to_string());
                start = i + 1;
            }
        }
        if start < s.len() {
            lines.push(s[start..].to_string());
        }
        lines
    }

    #[test]
    fn test_get_line_text_empty_document() {
        let pt = piece_tree::PieceTree::new("");
        assert_eq!("", get_line_text(&pt, 0));
        assert_eq!("", get_line_text(&pt, 1));
        assert_eq!("", get_line_text(&pt, 50));
    }

    #[test]
    fn test_get_line_text_single_line_without_newline() {
        let pt = piece_tree::PieceTree::new("hello world");
        assert_eq!("hello world", get_line_text(&pt, 0));
        assert_eq!("", get_line_text(&pt, 1));
        assert_eq!("", get_line_text(&pt, 10));
    }

    #[test]
    fn test_get_line_text_single_line_with_newline() {
        let pt = piece_tree::PieceTree::new("hello world\n");
        assert_eq!("hello world\n", get_line_text(&pt, 0));
        assert_eq!("", get_line_text(&pt, 1));
    }

    #[test]
    fn test_get_line_text_multiline_with_trailing_newline() {
        let pt = piece_tree::PieceTree::new("Line 0\nLine 1\nLine 2\n");
        assert_eq!("Line 0\n", get_line_text(&pt, 0));
        assert_eq!("Line 1\n", get_line_text(&pt, 1));
        assert_eq!("Line 2\n", get_line_text(&pt, 2));
        assert_eq!("", get_line_text(&pt, 3));
    }

    #[test]
    fn test_get_line_text_multiline_without_trailing_newline() {
        let pt = piece_tree::PieceTree::new("First line\nSecond line\nThird line");
        assert_eq!("First line\n", get_line_text(&pt, 0));
        assert_eq!("Second line\n", get_line_text(&pt, 1));
        assert_eq!("Third line", get_line_text(&pt, 2));
        assert_eq!("", get_line_text(&pt, 3));
    }

    #[test]
    fn test_get_line_text_only_newlines() {
        let pt_single = piece_tree::PieceTree::new("\n");
        assert_eq!("\n", get_line_text(&pt_single, 0));
        assert_eq!("", get_line_text(&pt_single, 1));

        let pt_multi = piece_tree::PieceTree::new("\n\n\n");
        assert_eq!("\n", get_line_text(&pt_multi, 0));
        assert_eq!("\n", get_line_text(&pt_multi, 1));
        assert_eq!("\n", get_line_text(&pt_multi, 2));
        assert_eq!("", get_line_text(&pt_multi, 3));
    }

    #[test]
    fn test_get_line_text_consecutive_empty_lines() {
        let pt = piece_tree::PieceTree::new("head\n\n\ntail\n");
        assert_eq!("head\n", get_line_text(&pt, 0));
        assert_eq!("\n", get_line_text(&pt, 1));
        assert_eq!("\n", get_line_text(&pt, 2));
        assert_eq!("tail\n", get_line_text(&pt, 3));
        assert_eq!("", get_line_text(&pt, 4));
    }

    #[test]
    fn test_get_line_text_replaces_existing_buffer_content() {
        let pt = piece_tree::PieceTree::new("hello\nworld\n");
        let mut content = String::from("stale buffer content that must be replaced");
        pt.get_line_text(1, &mut content);
        assert_eq!("world\n", content);

        // Out-of-bounds must clear the buffer
        let mut out_of_bounds_content = String::from("stale content");
        pt.get_line_text(99, &mut out_of_bounds_content);
        assert_eq!("", out_of_bounds_content);
    }

    #[test]
    fn test_get_line_text_large_out_of_bounds() {
        let pt = piece_tree::PieceTree::new("single line");
        assert_eq!("", get_line_text(&pt, usize::MAX));
        assert_eq!("", get_line_text(&pt, 100_000));
    }

    #[test]
    fn test_get_line_text_single_line_spanning_multiple_pieces() {
        let mut pt = piece_tree::PieceTree::new("hello ");
        pt.insert("brave ", 6);
        pt.insert("new ", 12);
        pt.insert("world\n", 16);
        assert_eq!("hello brave new world\n", get_text(&pt));

        assert_eq!("hello brave new world\n", get_line_text(&pt, 0));
        assert_eq!("", get_line_text(&pt, 1));
    }

    #[test]
    fn test_get_line_text_piece_containing_multiple_lines() {
        let mut pt = piece_tree::PieceTree::new("");
        pt.insert("line 0\nline 1\nline 2\nline 3\nline 4\n", 0);

        assert_eq!("line 0\n", get_line_text(&pt, 0));
        assert_eq!("line 1\n", get_line_text(&pt, 1));
        assert_eq!("line 2\n", get_line_text(&pt, 2));
        assert_eq!("line 3\n", get_line_text(&pt, 3));
        assert_eq!("line 4\n", get_line_text(&pt, 4));
        assert_eq!("", get_line_text(&pt, 5));
    }

    #[test]
    fn test_get_line_text_after_inserting_newline_splits_line() {
        let mut pt = piece_tree::PieceTree::new("Hello World\n");
        assert_eq!("Hello World\n", get_line_text(&pt, 0));

        // Insert newline between "Hello" and " World" (at byte index 5)
        pt.insert("\n", 5);
        assert_eq!("Hello\n World\n", get_text(&pt));

        assert_eq!("Hello\n", get_line_text(&pt, 0));
        assert_eq!(" World\n", get_line_text(&pt, 1));
        assert_eq!("", get_line_text(&pt, 2));
    }

    #[test]
    fn test_get_line_text_after_inserting_multiline_at_various_positions() {
        let mut pt = piece_tree::PieceTree::new("middle line\n");

        // Prepend multiline chunk at start (offset 0)
        pt.insert("header 0\nheader 1\n", 0);
        assert_eq!("header 0\n", get_line_text(&pt, 0));
        assert_eq!("header 1\n", get_line_text(&pt, 1));
        assert_eq!("middle line\n", get_line_text(&pt, 2));
        assert_eq!("", get_line_text(&pt, 3));

        // Append multiline chunk at end
        let end_pos = get_text(&pt).len();
        pt.insert("footer\n", end_pos);
        assert_eq!("header 0\n", get_line_text(&pt, 0));
        assert_eq!("header 1\n", get_line_text(&pt, 1));
        assert_eq!("middle line\n", get_line_text(&pt, 2));
        assert_eq!("footer\n", get_line_text(&pt, 3));
        assert_eq!("", get_line_text(&pt, 4));
    }

    #[test]
    fn test_get_line_text_after_deleting_newline_merges_lines() {
        let mut pt = piece_tree::PieceTree::new("line 0\nline 1\n");
        assert_eq!("line 0\n", get_line_text(&pt, 0));
        assert_eq!("line 1\n", get_line_text(&pt, 1));

        // Delete the '\n' between line 0 and line 1 (index 6, length 1)
        pt.delete(6, 1);
        assert_eq!("line 0line 1\n", get_text(&pt));

        assert_eq!("line 0line 1\n", get_line_text(&pt, 0));
        assert_eq!("", get_line_text(&pt, 1));
    }

    #[test]
    fn test_get_line_text_after_deleting_across_multiple_lines() {
        let mut pt = piece_tree::PieceTree::new("alpha\nbeta\ngamma\ndelta\n");
        // "alpha\n" = 6 bytes
        // "beta\n"  = 5 bytes (6..11)
        // "gamma\n" = 6 bytes (11..17)
        // "delta\n" = 6 bytes (17..23)
        // Delete from 't' in beta (offset 8) to 'l' in delta (offset 20, len 12)
        // "alpha\nbe" + "ta\ngamma\ndel" (deleted) + "ta\n" -> "alpha\nbeta\n"
        pt.delete(8, 12);
        assert_eq!("alpha\nbeta\n", get_text(&pt));

        assert_eq!("alpha\n", get_line_text(&pt, 0));
        assert_eq!("beta\n", get_line_text(&pt, 1));
        assert_eq!("", get_line_text(&pt, 2));
    }

    #[test]
    fn test_get_line_text_delete_all_resets_to_empty() {
        let mut pt = piece_tree::PieceTree::new("alpha\nbeta\ngamma\n");
        let len = get_text(&pt).len();
        pt.delete(0, len);
        assert_eq!("", get_text(&pt));

        assert_eq!("", get_line_text(&pt, 0));
        assert_eq!("", get_line_text(&pt, 1));
    }

    #[test]
    fn test_get_line_text_undo_and_redo() {
        let mut pt = piece_tree::PieceTree::new("line 0\n");
        pt.insert("line 1\n", 7);
        assert_eq!("line 0\n", get_line_text(&pt, 0));
        assert_eq!("line 1\n", get_line_text(&pt, 1));
        assert_eq!("", get_line_text(&pt, 2));

        pt.undo();
        assert_eq!("line 0\n", get_line_text(&pt, 0));
        assert_eq!("", get_line_text(&pt, 1));

        pt.redo();
        assert_eq!("line 0\n", get_line_text(&pt, 0));
        assert_eq!("line 1\n", get_line_text(&pt, 1));
        assert_eq!("", get_line_text(&pt, 2));
    }

    #[test]
    fn test_get_line_text_unicode_multibyte_characters() {
        let pt = piece_tree::PieceTree::new("🦀 Rustacean\n🎉 Celebrate\n✨ Magic\n");
        assert_eq!("🦀 Rustacean\n", get_line_text(&pt, 0));
        assert_eq!("🎉 Celebrate\n", get_line_text(&pt, 1));
        assert_eq!("✨ Magic\n", get_line_text(&pt, 2));
        assert_eq!("", get_line_text(&pt, 3));
    }

    #[test]
    fn test_get_line_text_crlf_newlines() {
        let pt = piece_tree::PieceTree::new("first\r\nsecond\r\nthird");
        assert_eq!("first\r\n", get_line_text(&pt, 0));
        assert_eq!("second\r\n", get_line_text(&pt, 1));
        assert_eq!("third", get_line_text(&pt, 2));
        assert_eq!("", get_line_text(&pt, 3));
    }

    #[test]
    fn test_get_line_text_fuzz_matches_reference_lines() {
        let mut pt = piece_tree::PieceTree::new("the quick\nbrown fox\njumps over\n");
        let mut reference = String::from("the quick\nbrown fox\njumps over\n");

        let mut rng = Lcg(0xFEED_FACE_CAFE_BABE);

        for step in 0..500 {
            let ref_lines = split_lines_reference(&reference);

            // Verify all existing lines match the reference model
            for (line_idx, expected_line) in ref_lines.iter().enumerate() {
                assert_eq!(
                    *expected_line,
                    get_line_text(&pt, line_idx),
                    "step {}: line {} mismatch\nDocument:\n{:?}",
                    step,
                    line_idx,
                    reference
                );
            }

            // Verify out-of-bounds line queries yield empty strings
            assert_eq!("", get_line_text(&pt, ref_lines.len()));
            assert_eq!("", get_line_text(&pt, ref_lines.len() + 1));
            assert_eq!("", get_line_text(&pt, ref_lines.len() + 10));

            // Mutate document with mix of plain text and newlines
            let op = rng.next() % 100;
            if op < 50 || reference.is_empty() {
                let snippets = [
                    "hello\n",
                    "\n",
                    "world",
                    "\n\n",
                    "foo\nbar\n",
                    "baz",
                    "line\r\n",
                ];
                let snippet = snippets[rng.below(snippets.len())];
                let pos = rng.below(reference.len() + 1);
                pt.insert(snippet, pos);
                reference.insert_str(pos, snippet);
            } else {
                let pos = rng.below(reference.len());
                let del = 1 + rng.below(reference.len() - pos);
                pt.delete(pos, del);
                reference.replace_range(pos..pos + del, "");
            }
        }
    }

    #[test]
    fn test_get_line_text_exact_line_count_and_boundaries() {
        // Document with no newline (1 line: line 0)
        let pt0 = piece_tree::PieceTree::new("only line");
        assert_eq!("only line", get_line_text(&pt0, 0));
        assert_eq!("", get_line_text(&pt0, 1));
        assert_eq!("", get_line_text(&pt0, 2));
        assert_eq!("", get_line_text(&pt0, usize::MAX));
        assert_eq!("", get_line_text(&pt0, usize::MAX - 1));

        // Document with single newline (1 line: line 0)
        let pt1 = piece_tree::PieceTree::new("only line\n");
        assert_eq!("only line\n", get_line_text(&pt1, 0));
        assert_eq!("", get_line_text(&pt1, 1));
        assert_eq!("", get_line_text(&pt1, 2));
        assert_eq!("", get_line_text(&pt1, usize::MAX));

        // Document with two lines without trailing newline (2 lines: 0, 1)
        let pt2 = piece_tree::PieceTree::new("line 0\nline 1");
        assert_eq!("line 0\n", get_line_text(&pt2, 0));
        assert_eq!("line 1", get_line_text(&pt2, 1));
        assert_eq!("", get_line_text(&pt2, 2));
        assert_eq!("", get_line_text(&pt2, 3));
        assert_eq!("", get_line_text(&pt2, usize::MAX));

        // Document with two lines with trailing newline (2 lines: 0, 1)
        let pt3 = piece_tree::PieceTree::new("line 0\nline 1\n");
        assert_eq!("line 0\n", get_line_text(&pt3, 0));
        assert_eq!("line 1\n", get_line_text(&pt3, 1));
        assert_eq!("", get_line_text(&pt3, 2));
        assert_eq!("", get_line_text(&pt3, 100));
    }

    #[test]
    fn test_get_line_text_buffer_reuse_alternating_valid_and_out_of_bounds() {
        let pt = piece_tree::PieceTree::new("first\nsecond\nthird");
        let mut buf = String::from("initial dirty content that is quite lengthy");

        // Valid query line 0
        pt.get_line_text(0, &mut buf);
        assert_eq!("first\n", buf);

        // Put dirty content again
        buf.push_str("extra dirty garbage");

        // Out-of-bounds query line 3 -> must clear to empty
        pt.get_line_text(3, &mut buf);
        assert_eq!("", buf);

        // Valid query line 1
        pt.get_line_text(1, &mut buf);
        assert_eq!("second\n", buf);

        // Out-of-bounds query usize::MAX -> must clear to empty
        pt.get_line_text(usize::MAX, &mut buf);
        assert_eq!("", buf);

        // Valid query line 2 (last line without newline)
        pt.get_line_text(2, &mut buf);
        assert_eq!("third", buf);

        // Out-of-bounds query 500
        pt.get_line_text(500, &mut buf);
        assert_eq!("", buf);
    }

    #[test]
    fn test_get_line_text_leading_newlines() {
        // Single leading newline followed by text
        let pt1 = piece_tree::PieceTree::new("\nhello\n");
        assert_eq!("\n", get_line_text(&pt1, 0));
        assert_eq!("hello\n", get_line_text(&pt1, 1));
        assert_eq!("", get_line_text(&pt1, 2));

        // Two leading newlines with text without trailing newline
        let pt2 = piece_tree::PieceTree::new("\n\nworld");
        assert_eq!("\n", get_line_text(&pt2, 0));
        assert_eq!("\n", get_line_text(&pt2, 1));
        assert_eq!("world", get_line_text(&pt2, 2));
        assert_eq!("", get_line_text(&pt2, 3));

        // Leading CRLFs
        let pt3 = piece_tree::PieceTree::new("\r\n\r\nfoo\r\n");
        assert_eq!("\r\n", get_line_text(&pt3, 0));
        assert_eq!("\r\n", get_line_text(&pt3, 1));
        assert_eq!("foo\r\n", get_line_text(&pt3, 2));
        assert_eq!("", get_line_text(&pt3, 3));
    }

    #[test]
    fn test_get_line_text_trailing_consecutive_newlines() {
        let pt = piece_tree::PieceTree::new("content\n\n\n");
        assert_eq!("content\n", get_line_text(&pt, 0));
        assert_eq!("\n", get_line_text(&pt, 1));
        assert_eq!("\n", get_line_text(&pt, 2));
        assert_eq!("", get_line_text(&pt, 3));
        assert_eq!("", get_line_text(&pt, 4));
        assert_eq!("", get_line_text(&pt, 100));
    }

    #[test]
    fn test_get_line_text_crlf_split_across_pieces() {
        // Piece 1 ends with '\r', Piece 2 starts with '\n'
        let mut pt = piece_tree::PieceTree::new("line 0\r");
        pt.insert("\nline 1\r\n", 7);
        assert_eq!("line 0\r\nline 1\r\n", get_text(&pt));

        assert_eq!("line 0\r\n", get_line_text(&pt, 0));
        assert_eq!("line 1\r\n", get_line_text(&pt, 1));
        assert_eq!("", get_line_text(&pt, 2));

        // Insert between \r and \n: breaks the CRLF into separate text
        let mut pt2 = piece_tree::PieceTree::new("hello\r\nworld\n");
        // offset 6 is between '\r' (offset 5) and '\n' (offset 6)
        pt2.insert("MIDDLE", 6);
        assert_eq!("hello\rMIDDLE\nworld\n", get_text(&pt2));
        assert_eq!("hello\rMIDDLE\n", get_line_text(&pt2, 0));
        assert_eq!("world\n", get_line_text(&pt2, 1));
        assert_eq!("", get_line_text(&pt2, 2));

        // Delete \r from \r\n
        let mut pt3 = piece_tree::PieceTree::new("hello\r\nworld\n");
        pt3.delete(5, 1); // delete '\r'
        assert_eq!("hello\nworld\n", get_text(&pt3));
        assert_eq!("hello\n", get_line_text(&pt3, 0));
        assert_eq!("world\n", get_line_text(&pt3, 1));
        assert_eq!("", get_line_text(&pt3, 2));

        // Delete \n from \r\n (merges line with following line)
        let mut pt4 = piece_tree::PieceTree::new("hello\r\nworld\n");
        pt4.delete(6, 1); // delete '\n'
        assert_eq!("hello\rworld\n", get_text(&pt4));
        assert_eq!("hello\rworld\n", get_line_text(&pt4, 0));
        assert_eq!("", get_line_text(&pt4, 1));

        // Mixed line endings in one document
        let pt5 = piece_tree::PieceTree::new("line 0\r\nline 1\nline 2\r\nline 3");
        assert_eq!("line 0\r\n", get_line_text(&pt5, 0));
        assert_eq!("line 1\n", get_line_text(&pt5, 1));
        assert_eq!("line 2\r\n", get_line_text(&pt5, 2));
        assert_eq!("line 3", get_line_text(&pt5, 3));
        assert_eq!("", get_line_text(&pt5, 4));
    }

    #[test]
    fn test_get_line_text_many_pieces_single_line() {
        let mut pt = piece_tree::PieceTree::new("");
        let mut expected = String::new();

        // Insert 50 pieces without any newlines
        for i in 0..50 {
            let frag = format!("f{}_", i);
            let pos = expected.len();
            pt.insert(&frag, pos);
            expected.push_str(&frag);
            assert_eq!(expected, get_line_text(&pt, 0));
            assert_eq!("", get_line_text(&pt, 1));
        }

        // Insert newline at the end
        let end_pos = expected.len();
        pt.insert("\n", end_pos);
        expected.push('\n');
        assert_eq!(expected, get_line_text(&pt, 0));
        assert_eq!("", get_line_text(&pt, 1));

        // Insert newline in the middle (at character 50)
        pt.insert("\n", 50);
        expected.insert(50, '\n');
        let ref_lines = split_lines_reference(&expected);
        assert_eq!(ref_lines[0], get_line_text(&pt, 0));
        assert_eq!(ref_lines[1], get_line_text(&pt, 1));
        assert_eq!("", get_line_text(&pt, 2));
    }

    #[test]
    fn test_get_line_text_interleaved_pieces_across_lines() {
        let mut pt = piece_tree::PieceTree::new("aaa");
        pt.insert("bbb\n", 3);
        pt.insert("ccc", 7);
        pt.insert("ddd", 10);
        pt.insert("\neee", 13);
        pt.insert("fff\n", 17);
        assert_eq!("aaabbb\ncccddd\neeefff\n", get_text(&pt));

        assert_eq!("aaabbb\n", get_line_text(&pt, 0));
        assert_eq!("cccddd\n", get_line_text(&pt, 1));
        assert_eq!("eeefff\n", get_line_text(&pt, 2));
        assert_eq!("", get_line_text(&pt, 3));
    }

    #[test]
    fn test_get_line_text_insert_at_exact_line_boundaries() {
        let mut pt = piece_tree::PieceTree::new("line0\nline1\nline2\n");

        // 1. Insert at offset 0 (start of document / start of line 0)
        pt.insert("prefix_", 0);
        assert_eq!("prefix_line0\n", get_line_text(&pt, 0));
        assert_eq!("line1\n", get_line_text(&pt, 1));
        assert_eq!("line2\n", get_line_text(&pt, 2));
        assert_eq!("", get_line_text(&pt, 3));

        // 2. Insert right before '\n' in line 0 (offset 12)
        pt.insert("_suffix", 12);
        assert_eq!("prefix_line0_suffix\n", get_line_text(&pt, 0));
        assert_eq!("line1\n", get_line_text(&pt, 1));
        assert_eq!("line2\n", get_line_text(&pt, 2));

        // 3. Insert right after '\n' in line 0 (offset 20, start of line 1)
        pt.insert("NEW_", 20);
        assert_eq!("prefix_line0_suffix\n", get_line_text(&pt, 0));
        assert_eq!("NEW_line1\n", get_line_text(&pt, 1));
        assert_eq!("line2\n", get_line_text(&pt, 2));

        // 4. Insert at the end of the document (after line 2's '\n')
        let doc_len = get_text(&pt).len();
        pt.insert("line3_appended\n", doc_len);
        assert_eq!("prefix_line0_suffix\n", get_line_text(&pt, 0));
        assert_eq!("NEW_line1\n", get_line_text(&pt, 1));
        assert_eq!("line2\n", get_line_text(&pt, 2));
        assert_eq!("line3_appended\n", get_line_text(&pt, 3));
        assert_eq!("", get_line_text(&pt, 4));

        // 5. Insert '\n' right before an existing '\n' (creates an empty line)
        pt.insert("\n", 19);
        assert_eq!("prefix_line0_suffix\n", get_line_text(&pt, 0));
        assert_eq!("\n", get_line_text(&pt, 1));
        assert_eq!("NEW_line1\n", get_line_text(&pt, 2));
        assert_eq!("line2\n", get_line_text(&pt, 3));
        assert_eq!("line3_appended\n", get_line_text(&pt, 4));
        assert_eq!("", get_line_text(&pt, 5));
    }

    #[test]
    fn test_get_line_text_delete_at_exact_line_boundaries() {
        let mut pt = piece_tree::PieceTree::new("line 0\nline 1\nline 2\nline 3\n");

        // 1. Delete char right before '\n' of line 0 ('0' at offset 5)
        pt.delete(5, 1);
        assert_eq!("line \n", get_line_text(&pt, 0));
        assert_eq!("line 1\n", get_line_text(&pt, 1));

        // 2. Delete the '\n' between line 0 and line 1 (offset 5) -> merges line 0 and line 1
        pt.delete(5, 1);
        assert_eq!("line line 1\n", get_line_text(&pt, 0));
        assert_eq!("line 2\n", get_line_text(&pt, 1));
        assert_eq!("line 3\n", get_line_text(&pt, 2));
        assert_eq!("", get_line_text(&pt, 3));

        // 3. Delete entire line 1 ("line 2\n", len 7 at offset 12)
        pt.delete(12, 7);
        assert_eq!("line line 1\n", get_line_text(&pt, 0));
        assert_eq!("line 3\n", get_line_text(&pt, 1));
        assert_eq!("", get_line_text(&pt, 2));

        // 4. Delete trailing newline of last line (offset 18, len 1)
        pt.delete(18, 1);
        assert_eq!("line line 1\n", get_line_text(&pt, 0));
        assert_eq!("line 3", get_line_text(&pt, 1));
        assert_eq!("", get_line_text(&pt, 2));
    }

    #[test]
    fn test_get_line_text_empty_lines_in_middle_and_edges() {
        let pt = piece_tree::PieceTree::new("\n\na\n\n\nb\n\n");
        assert_eq!("\n", get_line_text(&pt, 0));
        assert_eq!("\n", get_line_text(&pt, 1));
        assert_eq!("a\n", get_line_text(&pt, 2));
        assert_eq!("\n", get_line_text(&pt, 3));
        assert_eq!("\n", get_line_text(&pt, 4));
        assert_eq!("b\n", get_line_text(&pt, 5));
        assert_eq!("\n", get_line_text(&pt, 6));
        assert_eq!("", get_line_text(&pt, 7));
        assert_eq!("", get_line_text(&pt, 8));
        assert_eq!("", get_line_text(&pt, 9));
        assert_eq!("", get_line_text(&pt, 10));
    }

    #[test]
    fn test_get_line_text_deep_tree_many_lines() {
        let mut pt = piece_tree::PieceTree::new("");
        let mut reference = String::new();

        // 1. Insert 400 lines sequentially at the end
        for i in 0..400 {
            let line = format!("line {:04}\n", i);
            let pos = reference.len();
            pt.insert(&line, pos);
            reference.push_str(&line);
        }

        // 2. Insert 100 lines at offset 0 (left-heavy rotations)
        for i in 0..100 {
            let line = format!("prepended {:03}\n", i);
            pt.insert(&line, 0);
            reference.insert_str(0, &line);
        }

        // 3. Insert 100 lines in the middle
        for i in 0..100 {
            let line = format!("middle {:03}\n", i);
            let mid_pos = reference.len() / 2;
            pt.insert(&line, mid_pos);
            reference.insert_str(mid_pos, &line);
        }

        let ref_lines = split_lines_reference(&reference);
        assert_eq!(600, ref_lines.len());

        // Verify every line in the 600-line deep tree
        for (i, expected) in ref_lines.iter().enumerate() {
            assert_eq!(
                *expected,
                get_line_text(&pt, i),
                "mismatch at line {}",
                i
            );
        }

        // Out-of-bounds queries on deep tree
        assert_eq!("", get_line_text(&pt, 600));
        assert_eq!("", get_line_text(&pt, 601));
        assert_eq!("", get_line_text(&pt, 1000));
        assert_eq!("", get_line_text(&pt, usize::MAX));
    }

    #[test]
    fn test_get_line_text_simulated_typing_and_backspacing() {
        let mut pt = piece_tree::PieceTree::new("");
        let mut text = String::new();

        let script = "hello\nworld\nfoo\nbar\n";
        for ch in script.chars() {
            let mut s = String::new();
            s.push(ch);
            let pos = text.len();
            pt.insert(&s, pos);
            text.push(ch);

            let ref_lines = split_lines_reference(&text);
            for (idx, exp) in ref_lines.iter().enumerate() {
                assert_eq!(*exp, get_line_text(&pt, idx));
            }
            assert_eq!("", get_line_text(&pt, ref_lines.len()));
        }

        // Backspace 10 characters one by one (covers deleting \n, characters, etc.)
        for _ in 0..10 {
            let pos = text.len() - 1;
            pt.delete(pos, 1);
            text.remove(pos);

            let ref_lines = split_lines_reference(&text);
            for (idx, exp) in ref_lines.iter().enumerate() {
                assert_eq!(*exp, get_line_text(&pt, idx));
            }
            assert_eq!("", get_line_text(&pt, ref_lines.len()));
        }
    }

    #[test]
    fn test_get_line_text_undo_redo_complex_line_operations() {
        let mut pt = piece_tree::PieceTree::new("line A\nline B\nline C\n");
        let mut history: Vec<Vec<String>> = Vec::new();

        history.push(split_lines_reference(&get_text(&pt)));

        // Op 1: Prepend multiline
        pt.insert("header 1\nheader 2\n", 0);
        history.push(split_lines_reference(&get_text(&pt)));

        // Op 2: Insert into line B
        let text_now = get_text(&pt);
        let b_pos = text_now.find("line B").unwrap();
        pt.insert("MODIFIED_", b_pos);
        history.push(split_lines_reference(&get_text(&pt)));

        // Op 3: Delete across lines (delete from middle of line A across newline into MODIFIED_line B)
        let text_now = get_text(&pt);
        let a_pos = text_now.find("line A").unwrap() + 4; // after "line"
        pt.delete(a_pos, 15);
        history.push(split_lines_reference(&get_text(&pt)));

        // Op 4: Append multiline
        let end = get_text(&pt).len();
        pt.insert("footer 1\nfooter 2", end);
        history.push(split_lines_reference(&get_text(&pt)));

        // Op 5: Delete everything
        let all_len = get_text(&pt).len();
        pt.delete(0, all_len);
        history.push(split_lines_reference(&get_text(&pt)));

        // Walk backwards with undo, checking all lines at every state
        for expected_lines in history.iter().rev() {
            for (line_idx, expected_line) in expected_lines.iter().enumerate() {
                assert_eq!(*expected_line, get_line_text(&pt, line_idx));
            }
            assert_eq!("", get_line_text(&pt, expected_lines.len()));
            pt.undo();
        }

        // Walk forward with redo, checking all lines at every state
        for expected_lines in history.iter().skip(1) {
            pt.redo();
            for (line_idx, expected_line) in expected_lines.iter().enumerate() {
                assert_eq!(*expected_line, get_line_text(&pt, line_idx));
            }
            assert_eq!("", get_line_text(&pt, expected_lines.len()));
        }
    }

    #[test]
    fn test_get_line_text_whitespace_and_tabs() {
        let pt = piece_tree::PieceTree::new("   \n\t\t\t\n \t \t \n\r\r\n");
        assert_eq!("   \n", get_line_text(&pt, 0));
        assert_eq!("\t\t\t\n", get_line_text(&pt, 1));
        assert_eq!(" \t \t \n", get_line_text(&pt, 2));
        assert_eq!("\r\r\n", get_line_text(&pt, 3));
        assert_eq!("", get_line_text(&pt, 4));
    }

    #[test]
    fn test_get_line_text_unicode_edits_and_line_splits() {
        let mut pt = piece_tree::PieceTree::new("日本語\n한국어\n中文\n");
        assert_eq!("日本語\n", get_line_text(&pt, 0));
        assert_eq!("한국어\n", get_line_text(&pt, 1));
        assert_eq!("中文\n", get_line_text(&pt, 2));
        assert_eq!("", get_line_text(&pt, 3));

        // Insert emoji in line 1 ("한국어\n" -> "한국✨어\n")
        pt.insert("✨", 16);
        assert_eq!("日本語\n", get_line_text(&pt, 0));
        assert_eq!("한국✨어\n", get_line_text(&pt, 1));
        assert_eq!("中文\n", get_line_text(&pt, 2));
        assert_eq!("", get_line_text(&pt, 3));

        // Split line 0 with a newline: "日\n本語\n"
        pt.insert("\n", 3);
        assert_eq!("日\n", get_line_text(&pt, 0));
        assert_eq!("本語\n", get_line_text(&pt, 1));
        assert_eq!("한국✨어\n", get_line_text(&pt, 2));
        assert_eq!("中文\n", get_line_text(&pt, 3));
        assert_eq!("", get_line_text(&pt, 4));

        // Delete "語" (3 bytes at offset 7)
        pt.delete(7, 3);
        assert_eq!("日\n", get_line_text(&pt, 0));
        assert_eq!("本\n", get_line_text(&pt, 1));
        assert_eq!("한국✨어\n", get_line_text(&pt, 2));
        assert_eq!("中文\n", get_line_text(&pt, 3));
        assert_eq!("", get_line_text(&pt, 4));
    }

    #[test]
    fn test_get_line_text_repeated_sequential_edits_on_same_line() {
        let mut pt = piece_tree::PieceTree::new("header\ntarget\nfooter\n");

        // Mutate only line 1 multiple times
        pt.insert("_1", 13);
        assert_eq!("header\n", get_line_text(&pt, 0));
        assert_eq!("target_1\n", get_line_text(&pt, 1));
        assert_eq!("footer\n", get_line_text(&pt, 2));

        pt.insert("_2", 15);
        assert_eq!("header\n", get_line_text(&pt, 0));
        assert_eq!("target_1_2\n", get_line_text(&pt, 1));
        assert_eq!("footer\n", get_line_text(&pt, 2));

        pt.delete(13, 2);
        assert_eq!("header\n", get_line_text(&pt, 0));
        assert_eq!("target_2\n", get_line_text(&pt, 1));
        assert_eq!("footer\n", get_line_text(&pt, 2));
    }

    #[test]
    fn test_get_line_text_differential_fuzz_multi_seed() {
        let seeds: [u64; 5] = [
            0x1234_5678_9ABC_DEF0,
            0xDEAD_BEEF_0123_4567,
            0xCAFE_BABE_89AB_CDEF,
            0x0FED_CBA9_8765_4321,
            0xAAAA_5555_AAAA_5555,
        ];

        for seed in seeds {
            let mut pt = piece_tree::PieceTree::new("");
            let mut reference = String::new();
            let mut rng = Lcg(seed);

            for step in 0..250 {
                let op = rng.next() % 100;
                if op < 55 || reference.is_empty() {
                    let snippets = [
                        "a",
                        "\n",
                        "line\n",
                        "\n\n",
                        "hello world",
                        "split\r\nline",
                        "🦀",
                        "foo\nbar\nbaz\n",
                    ];
                    let snippet = snippets[rng.below(snippets.len())];
                    // Pick a valid char boundary for insertion
                    let char_indices: Vec<usize> =
                        reference.char_indices().map(|(idx, _)| idx).collect();
                    let pos = if char_indices.is_empty() || rng.below(2) == 0 {
                        reference.len()
                    } else {
                        char_indices[rng.below(char_indices.len())]
                    };
                    pt.insert(snippet, pos);
                    reference.insert_str(pos, snippet);
                } else if op < 85 {
                    let char_indices: Vec<usize> =
                        reference.char_indices().map(|(idx, _)| idx).collect();
                    if !char_indices.is_empty() {
                        let start_idx = rng.below(char_indices.len());
                        let pos = char_indices[start_idx];
                        let end_idx =
                            start_idx + 1 + rng.below(char_indices.len() - start_idx);
                        let end_pos = if end_idx < char_indices.len() {
                            char_indices[end_idx]
                        } else {
                            reference.len()
                        };
                        let del = end_pos - pos;
                        pt.delete(pos, del);
                        reference.replace_range(pos..end_pos, "");
                    }
                } else if op < 93 {
                    pt.undo();
                    reference = get_text(&pt);
                } else {
                    pt.redo();
                    reference = get_text(&pt);
                }

                let ref_lines = split_lines_reference(&reference);
                for (line_idx, expected_line) in ref_lines.iter().enumerate() {
                    assert_eq!(
                        *expected_line,
                        get_line_text(&pt, line_idx),
                        "seed {:#x}, step {}: mismatch at line {}\nDocument:\n{:?}",
                        seed,
                        step,
                        line_idx,
                        reference
                    );
                }

                // Verify exact boundary and out of bounds
                assert_eq!("", get_line_text(&pt, ref_lines.len()));
                assert_eq!("", get_line_text(&pt, ref_lines.len() + 1));
                assert_eq!("", get_line_text(&pt, ref_lines.len() + 20));
            }
        }
    }
}


