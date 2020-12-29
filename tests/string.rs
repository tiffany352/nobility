use nobility::bin_decode::NbtString;

#[test]
fn test_string_debug_malformed() {
    let string = NbtString::new(b"foo bar\" \0 \xC0");

    let formatted = format!("{:?}", string);
    assert_eq!(formatted, r#""foo bar\" \0 \xC0""#);
}
