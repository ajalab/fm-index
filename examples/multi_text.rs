use fm_index::converter::IdConverter;
use fm_index::{Match, MatchWithTextId, MultiTextFMIndexWithLocate, Search};

fn main() {
    // When using MultiTextFMIndex, the text is concatenated with an end marker \0.
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
    );

    // Converter converts each character into packed representation.
    // IdConverter represents an identity converter, which preserves the given characters.
    let converter = IdConverter::new::<u8>();

    let fm_index = MultiTextFMIndexWithLocate::new(text.as_bytes().to_vec(), converter, 2);

    // Count the number of occurrences.
    assert_eq!(4, fm_index.search("star").count());

    // List the text IDs of all occurrences.
    let mut text_ids = fm_index
        .search("How I wonder")
        .iter_matches()
        .map(|m| m.text_id().into())
        .collect::<Vec<u64>>();
    text_ids.sort();
    assert_eq!(vec![0, 0, 1, 2], text_ids);

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

    // List the IDs of texts that have the suffix.
    let mut text_ids_with_suffix = fm_index
        .search_suffix("what you are!\n")
        .iter_matches()
        .map(|m| m.text_id().into())
        .collect::<Vec<u64>>();
    text_ids_with_suffix.sort();
    assert_eq!(vec![0, 1, 2], text_ids_with_suffix);
}
