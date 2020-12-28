# Nobility

Nobility is a Rust crate for encoding and decoding NBT, which is a
format used by Minecraft: Java Edition.

Features:

- Decoder which creates few memory allocations
- Encoder that uses builders instead of heap allocated objects
- Supports TAG_Long_Array, added in Minecraft 1.12 (all tags as of
  2020).
- Can encode and decode test files correctly (e.g. bigtest.nbt).
- Supports the Java variant of CESU-8 used for encoding text.
- Zero usage of `unsafe`.

This library is based on the spec at
<https://wiki.vg/NBT#Specification>.

Missing features:

- Serde support. Ran into lifetime issues.
- CJSON support. Not yet implemented.
- Bedrock edition support. The format used there is different.
- Roundtrip encode/decode, as the encoder and decoder use different
  types.

## Decoding

```rust
let mut file = File::open("hello_world.nbt").unwrap();
let mut data = vec![];
file.read_to_end(&mut data).unwrap();
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
```

## Encoding

```rust
let mut writer = NbtWriter::new();

let mut root = writer.root("hello world");
root.field("name").string("Bananrama");
// finish() call is required.
root.finish();

let result: Vec<u8> = writer.finish();
```
