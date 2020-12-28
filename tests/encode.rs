use flate2::read::GzDecoder;
use nobility::bin_encode::NbtWriter;
use std::io::Read;

#[test]
fn test_hello() {
    // Uses NbtWriter to construct a document, then makes sure it's a
    // perfect byte match with the example document.

    let mut writer = NbtWriter::new();

    let mut root = writer.root("hello world");
    root.field("name").string("Bananrama");
    root.finish();

    let result = writer.finish();
    let expected = include_bytes!("files/hello_world.nbt");
    assert_eq!(result, expected);
}

#[test]
fn test_bigtest() {
    // Same deal, but for bigtest.nbt. Note that the order of fields in
    // the actual file is completely different from the example on
    // wiki.vg.

    let mut writer = NbtWriter::new();
    let mut root = writer.root("Level");

    root.field("longTest").long(9223372036854775807);
    root.field("shortTest").short(32767);
    root.field("stringTest")
        .raw_string(b"HELLO WORLD THIS IS A TEST STRING \xc3\x85\xc3\x84\xc3\x96!");
    root.field("floatTest").float(0.49823147058486938);
    root.field("intTest").int(2147483647);

    {
        let mut nested = root.compound_field("nested compound test");
        {
            let mut nested_ham = nested.compound_field("ham");
            nested_ham.field("name").string("Hampus");
            nested_ham.field("value").float(0.75);
            nested_ham.finish();
        }
        {
            let mut nested_egg = nested.compound_field("egg");
            nested_egg.field("name").string("Eggbert");
            nested_egg.field("value").float(0.5);
            nested_egg.finish();
        }
        nested.finish();
    }

    root.field("listTest (long)")
        .long_list(&[11, 12, 13, 14, 15]);

    {
        let mut compound_list = root.compound_list_field("listTest (compound)");
        {
            let mut elt = compound_list.element();
            elt.field("name").string("Compound tag #0");
            elt.field("created-on").long(1264099775885);
            elt.finish();
        }
        {
            let mut elt = compound_list.element();
            elt.field("name").string("Compound tag #1");
            elt.field("created-on").long(1264099775885);
            elt.finish();
        }
        compound_list.finish();
    }

    root.field("byteTest").byte(127);

    let mut byte_array_test = vec![];
    byte_array_test.reserve(1000);
    for n in 0..1000 {
        byte_array_test.push(((n * n * 255 + n * 7) % 100) as u8);
    }
    root.field("byteArrayTest (the first 1000 values of (n*n*255+n*7)%100, starting with n=0 (0, 62, 34, 16, 8, ...))").byte_array(&byte_array_test);

    root.field("doubleTest").double(0.49312871321823148);

    root.finish();
    let result = writer.finish();

    let cursor = std::io::Cursor::new(include_bytes!("files/bigtest.nbt"));
    let mut decoder = GzDecoder::new(cursor);
    let mut expected = vec![];
    decoder.read_to_end(&mut expected).unwrap();

    let mut max = 20;

    for (i, (left, right)) in result.iter().zip(expected.iter()).enumerate() {
        if max == 0 {
            println!("Skipping remaining mismatches");
            break;
        }
        if left != right {
            println!("Mismatch at offset {}: {:x} != {:x}", i, left, right);
            max -= 1;
        }
    }

    // Uncomment for debugging
    // let mut file = File::create("result.bin").unwrap();
    // file.write(&result);

    assert_eq!(result.len(), expected.len());
    if max < 20 {
        panic!("Failed");
    }
}
