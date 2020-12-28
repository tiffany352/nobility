use nobility::bin_decode::Document;

#[test]
pub fn decode_hello_world() {
    let data = include_bytes!("files/hello_world.nbt");
    let cursor = std::io::Cursor::new(data);

    let document = Document::load(cursor).unwrap();
    let (name, root) = document.parse().expect("Parsing to succeed");

    assert_eq!(name, "hello world");
    assert_eq!(root.len(), 1);
    assert_eq!(root[0].name, "name");
    let value_str = root[0].value.as_string().expect("Value to be a string");
    let value_str = value_str.decode().expect("Decode to succeed");
    assert_eq!(value_str, "Bananrama");
}

#[test]
pub fn decode_bigtest() {
    let data = include_bytes!("files/bigtest.nbt");
    let cursor = std::io::Cursor::new(data);

    let document = Document::load(cursor).unwrap();
    let (name, root) = document.parse().expect("Parsing to succeed");

    assert_eq!(name, "Level");
    assert_eq!(root.len(), 11);
}
