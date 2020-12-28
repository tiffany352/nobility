use nobility::bin_decode::Document;
use std::fs::File;
use std::io::Read;

fn main() {
    // Load the file to parse. Document::load takes any implementation of Read.
    let mut file = File::open("tests/files/hello_world.nbt").expect("File to exist");
    let mut data = vec![];
    file.read_to_end(&mut data).expect("Read to succeed");
    let cursor = std::io::Cursor::new(data);

    // Load the document. This step either copies the data (plaintext)
    // or decompresses it (gzip).
    let doc = Document::load(cursor).unwrap();
    // Parses the document. This returns the root tag's name, and the
    // root tag (always a Compound tag). Both of these are borrowing the
    // data inside the Document.
    let (name, root) = doc.parse().unwrap();

    println!("name: {}", name.decode().unwrap());
    println!("{:#?}", root);
}
