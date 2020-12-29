use crate::bin_encode::{CompoundListWriter, CompoundWriter, NbtWriter};
use crate::TagType;
use byteorder::{BigEndian, ByteOrder};

/// A builder for creating NBT tags. This is created using [CompoundWriter::field].
///
/// # Example
///
/// ```rust
/// # use nobility::bin_encode::NbtWriter;
/// # let mut writer = NbtWriter::new();
/// # let mut player = writer.root("test");
/// player.field("Name").string("Tiffany");
/// player.field("Health").int(20);
///
/// // There is no bool type in NBT, so bytes 0 and 1 are used instead.
/// player.field("EnjoysWritingDocumentation").byte(1);
///
/// player.field("FavoriteNumbers").float_list(&[3.14159, 7.0, 2147483647.0]);
///
/// # player.finish();
/// # let _ = writer.finish();
/// ```
#[derive(Debug)]
pub struct TagWriter<'a> {
    writer: &'a mut NbtWriter,
    name: Option<&'a str>,
    done: bool,
}

impl<'a> TagWriter<'a> {
    pub(crate) fn new_field(writer: &'a mut NbtWriter, name: &'a str) -> TagWriter<'a> {
        TagWriter {
            writer,
            name: Some(name),
            done: false,
        }
    }

    fn header(&mut self, tag: TagType) {
        if self.done {
            panic!("TagWriter can only be used once");
        }

        self.writer.write_tag(tag);
        if let Some(name) = self.name {
            self.writer.write_string(name);
        }
        self.done = true;
    }

    /// Create a TAG_Byte.
    pub fn byte(&mut self, value: i8) {
        self.header(TagType::Byte);
        self.writer.write_i8(value);
    }

    /// Create a TAG_Short.
    pub fn short(&mut self, value: i16) {
        self.header(TagType::Short);
        self.writer.write_i16(value);
    }

    /// Create a TAG_Int.
    pub fn int(&mut self, value: i32) {
        self.header(TagType::Int);
        self.writer.write_i32(value);
    }

    /// Create a TAG_Long.
    pub fn long(&mut self, value: i64) {
        self.header(TagType::Long);
        self.writer.write_i64(value);
    }

    /// Create a TAG_Float.
    pub fn float(&mut self, value: f32) {
        self.header(TagType::Float);
        self.writer.write_f32(value);
    }

    /// Create a TAG_Double.
    pub fn double(&mut self, value: f64) {
        self.header(TagType::Double);
        self.writer.write_f64(value);
    }

    /// Create a TAG_Byte_Array.
    pub fn byte_array(&mut self, data: &[u8]) {
        self.header(TagType::ByteArray);
        self.writer.write_u32(data.len() as u32);
        self.writer.write_bytes(data);
    }

    /// Create a TAG_String.
    pub fn string(&mut self, value: &str) {
        self.header(TagType::String);
        self.writer.write_string(value);
    }

    /// Similar to string(), but allows writing a string using raw
    /// binary data, in case the string you're writing contains invalid
    /// UTF-8.
    pub fn raw_string(&mut self, data: &[u8]) {
        self.header(TagType::String);
        self.writer.write_u16(data.len() as u16);
        self.writer.write_bytes(data);
    }

    /// Create a TAG_Compound and returns a builder for its contents.
    pub fn compound(&'a mut self) -> CompoundWriter<'a> {
        self.header(TagType::Compound);
        CompoundWriter::new(self.writer)
    }

    /// Create a TAG_Int_Array from the given slice.
    pub fn int_array(&mut self, data: &[i32]) {
        self.header(TagType::IntArray);
        self.writer.write_u32(data.len() as u32);
        for element in data {
            self.writer.write_i32(*element);
        }
    }

    /// Create a TAG_Long_Array from the given slice.
    pub fn long_array(&mut self, data: &[i64]) {
        self.header(TagType::IntArray);
        self.writer.write_u32(data.len() as u32);
        for element in data {
            self.writer.write_i64(*element);
        }
    }

    /// Create a TAG_List of TAG_Byte.
    pub fn byte_list(&mut self, data: &[u8]) {
        self.header(TagType::List);
        self.writer.write_tag(TagType::Byte);
        self.writer.write_u32(data.len() as u32);
        self.writer.write_bytes(data);
    }

    /// Create a TAG_List of TAG_Short.
    pub fn short_list(&mut self, data: &[i16]) {
        self.header(TagType::List);
        self.writer.write_tag(TagType::Short);
        self.writer.write_u32(data.len() as u32);
        for element in data {
            self.writer.write_i16(*element);
        }
    }

    /// Create a TAG_List of TAG_Int.
    pub fn int_list(&mut self, data: &[i32]) {
        self.header(TagType::List);
        self.writer.write_tag(TagType::Int);
        self.writer.write_u32(data.len() as u32);
        for element in data {
            self.writer.write_i32(*element);
        }
    }

    /// Create a TAG_List of TAG_Long.
    pub fn long_list(&mut self, data: &[i64]) {
        self.header(TagType::List);
        self.writer.write_tag(TagType::Long);
        self.writer.write_u32(data.len() as u32);
        for element in data {
            self.writer.write_i64(*element);
        }
    }

    /// Create a TAG_List of TAG_Float.
    pub fn float_list(&mut self, data: &[f32]) {
        self.header(TagType::List);
        self.writer.write_tag(TagType::Float);
        self.writer.write_u32(data.len() as u32);
        for element in data {
            self.writer.write_f32(*element);
        }
    }

    /// Create a TAG_List of TAG_Double.
    pub fn double_list(&mut self, data: &[f64]) {
        self.header(TagType::List);
        self.writer.write_tag(TagType::Double);
        self.writer.write_u32(data.len() as u32);
        for element in data {
            self.writer.write_f64(*element);
        }
    }

    /// Create a TAG_List of TAG_String.
    pub fn string_list(&mut self, data: &[&str]) {
        self.header(TagType::List);
        self.writer.write_tag(TagType::String);
        self.writer.write_u32(data.len() as u32);
        for element in data {
            self.writer.write_string(*element);
        }
    }

    /// Create a TAG_List of TAG_Byte_Array.
    pub fn byte_array_list(&mut self, data: &[&[u8]]) {
        self.header(TagType::List);
        self.writer.write_tag(TagType::ByteArray);
        self.writer.write_u32(data.len() as u32);
        for element in data {
            self.writer.write_u32(element.len() as u32);
            self.writer.write_bytes(element);
        }
    }

    /// Create a TAG_List of TAG_Compound.
    pub fn compound_list(&'a mut self) -> CompoundListWriter<'a> {
        self.header(TagType::List);
        CompoundListWriter::new(self.writer)
    }

    /// Writes the bytes of a UUID in the Minecraft 1.16+ format
    /// (TAG_Int_Array of length 4).
    pub fn uuid_bytes(&mut self, bytes: [u8; 16]) {
        self.int_array(&[
            BigEndian::read_i32(&bytes[0..4]),
            BigEndian::read_i32(&bytes[4..8]),
            BigEndian::read_i32(&bytes[8..12]),
            BigEndian::read_i32(&bytes[12..16]),
        ]);
    }

    /// Writes a [uuid::Uuid] in the Minecraft 1.16+ format
    /// (TAG_Int_Array of length 4). Requires the `uuid` feature.
    #[cfg(feature = "uuid")]
    pub fn uuid(&mut self, uuid: uuid::Uuid) {
        self.uuid_bytes(*uuid.as_bytes());
    }

    // todo: list list, compound list, int array list, long array list

    /// Returns whether or not the tag has been written into.
    pub fn is_finished(&self) -> bool {
        self.done
    }
}
