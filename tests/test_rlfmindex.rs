use fm_index::{converter::RangeConverter, RLFMIndex, RLFMIndexWithLocate};

#[test]
fn test_count() {
    let text = "mississippi".to_string().into_bytes();
    let ans = vec![
        ("m", 1),
        ("mi", 1),
        ("i", 4),
        ("iss", 2),
        ("ss", 2),
        ("p", 2),
        ("ppi", 1),
        ("z", 0),
        ("pps", 0),
    ];
    let rlfmi = RLFMIndex::new(text, RangeConverter::new(b'a', b'z'));
    for (pattern, expected) in ans {
        let search = rlfmi.search(pattern);
        let actual = search.count();
        assert_eq!(
            expected, actual,
            "pattern \"{}\" must occur {} times, but {}",
            pattern, expected, actual,
        );
    }
}

#[test]
fn test_locate() {
    let text = "mississippi".to_string().into_bytes();
    let ans = vec![
        ("m", vec![0]),
        ("mi", vec![0]),
        ("i", vec![1, 4, 7, 10]),
        ("iss", vec![1, 4]),
        ("ss", vec![2, 5]),
        ("p", vec![8, 9]),
        ("ppi", vec![8]),
        ("z", vec![]),
        ("pps", vec![]),
    ];

    let fm_index = RLFMIndexWithLocate::new(text, RangeConverter::new(b'a', b'z'), 2);

    for (pattern, positions) in ans {
        let search = fm_index.search(pattern);
        let expected = positions.len() as u64;
        let actual = search.count();
        assert_eq!(
            expected,
            actual,
            "pattern \"{}\" must occur {} times, but {}: {:?}",
            pattern,
            expected,
            actual,
            search.locate()
        );
        let mut res = search.locate();
        res.sort();
        assert_eq!(res, positions);
    }
}

#[test]
fn test_iter_backward() {
    let text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.".to_string().into_bytes();
    let index = RLFMIndex::new(text, RangeConverter::new(b' ', b'~'));
    let search = index.search("sit ");
    let mut prev_seq = search.iter_backward(0).take(6).collect::<Vec<_>>();
    prev_seq.reverse();
    assert_eq!(prev_seq, b"dolor ".to_owned());
}

#[test]
fn test_iter_forward() {
    let text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.".to_string().into_bytes();
    let index = RLFMIndex::new(text, RangeConverter::new(b' ', b'~'));
    let search = index.search("sit ");
    let next_seq = search.iter_forward(0).take(10).collect::<Vec<_>>();
    assert_eq!(next_seq, b"sit amet, ".to_owned());
}
