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
}

