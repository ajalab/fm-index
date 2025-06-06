use fm_index::{FMIndexMultiPiecesWithLocate, Match, MatchWithPieceId, Search, Text};

fn main() {
    // When using FMIndexMultiPieces, the text is concatenated with an end marker \0.
    let text = concat!(
        // 0
        "Twinkle, twinkle, little star,\n",
        "How I wonder what you are!\n",
        "Up above the world so high,\n",
        "Like a diamond in the sky.\n",
        "Twinkle, twinkle, little star,\n",
        "How I wonder what you are!\n\0",
        // 1
        "When the blazing sun is gone,\n",
        "When he nothing shines upon,\n",
        "Then you show your little light,\n",
        "Twinkle, twinkle, all the night.\n",
        "Twinkle, twinkle, little star,\n",
        "How I wonder what you are!\n\0",
        // 2
        "Then the traveller in the dark,\n",
        "Thanks you for your tiny spark;\n",
        "He could not see which way to go,\n",
        "If you did not twinkle so.\n",
        "Twinkle, twinkle, little star,\n",
        "How I wonder what you are!\n\0",
    )
    .as_bytes();
    let text = Text::new(text);

    let fm_index = FMIndexMultiPiecesWithLocate::new(&text, 2).unwrap();

    // Count the number of occurrences.
    assert_eq!(4, fm_index.search("star").count());

    // List the IDs of pieces that match the given pattern.
    let mut piece_ids = fm_index
        .search("How I wonder")
        .iter_matches()
        .map(|m| m.piece_id().into())
        .collect::<Vec<usize>>();
    piece_ids.sort();
    assert_eq!(vec![0, 0, 1, 2], piece_ids);

    // Extract preceding characters from a search position.
    let preceding_chars = fm_index
        .search(" in the dark")
        .iter_matches()
        .map(|m| {
            m.iter_chars_backward()
                .take_while(|c| *c != b' ')
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(vec![b"rellevart".to_vec()], preceding_chars);

    // Extract succeeding characters from a search position.
    let succeeding_chars = fm_index
        .search("ing ")
        .iter_matches()
        .map(|m| {
            m.iter_chars_forward()
                .take_while(|c| *c != b',')
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        vec![b"ing shines upon".to_vec(), b"ing sun is gone".to_vec()],
        succeeding_chars,
    );

    // List the IDs of pieces that start with a given prefix.
    let mut piece_ids_with_prefix = fm_index
        .search_prefix("Twinkle")
        .iter_matches()
        .map(|m| m.piece_id().into())
        .collect::<Vec<usize>>();
    piece_ids_with_prefix.sort();
    assert_eq!(vec![0], piece_ids_with_prefix);

    // List the IDs of pieces that end with a given suffix.
    let mut piece_ids_with_suffix = fm_index
        .search_suffix("what you are!\n")
        .iter_matches()
        .map(|m| m.piece_id().into())
        .collect::<Vec<usize>>();
    piece_ids_with_suffix.sort();
    assert_eq!(vec![0, 1, 2], piece_ids_with_suffix);
}
