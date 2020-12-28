use crate::bin_encode::{CompoundListWriter, CompoundWriter, NbtWriter};
use crate::TagType;

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

    pub fn byte(&mut self, value: i8) {
        self.header(TagType::Byte);
        self.writer.write_i8(value);
    }

    pub fn short(&mut self, value: i16) {
        self.header(TagType::Short);
        self.writer.write_i16(value);
    }

    pub fn int(&mut self, value: i32) {
        self.header(TagType::Int);
        self.writer.write_i32(value);
    }

    pub fn long(&mut self, value: i64) {
        self.header(TagType::Long);
        self.writer.write_i64(value);
    }

    pub fn float(&mut self, value: f32) {
        self.header(TagType::Float);
        self.writer.write_f32(value);
    }

    pub fn double(&mut self, value: f64) {
        self.header(TagType::Double);
        self.writer.write_f64(value);
    }

    pub fn byte_array(&mut self, data: &[u8]) {
        self.header(TagType::ByteArray);
        self.writer.write_u32(data.len() as u32);
        self.writer.write_bytes(data);
    }

    pub fn string(&mut self, value: &str) {
        self.header(TagType::String);
        self.writer.write_string(value);
    }

    /// Allows writing a string using raw binary data, in case the
    /// string you're writing contains invalid UTF-8.
    pub fn raw_string(&mut self, data: &[u8]) {
        self.header(TagType::String);
        self.writer.write_u16(data.len() as u16);
        self.writer.write_bytes(data);
    }

    pub fn compound(&'a mut self) -> CompoundWriter<'a> {
        self.header(TagType::Compound);
        CompoundWriter::new(self.writer)
    }

    pub fn int_array(&mut self, data: &[i32]) {
        self.header(TagType::IntArray);
        self.writer.write_u32(data.len() as u32);
        for element in data {
            self.writer.write_i32(*element);
        }
    }

    pub fn long_array(&mut self, data: &[i64]) {
        self.header(TagType::IntArray);
        self.writer.write_u32(data.len() as u32);
        for element in data {
            self.writer.write_i64(*element);
        }
    }

    pub fn byte_list(&mut self, data: &[u8]) {
        self.header(TagType::List);
        self.writer.write_tag(TagType::Byte);
        self.writer.write_u32(data.len() as u32);
        self.writer.write_bytes(data);
    }

    pub fn short_list(&mut self, data: &[i16]) {
        self.header(TagType::List);
        self.writer.write_tag(TagType::Short);
        self.writer.write_u32(data.len() as u32);
        for element in data {
            self.writer.write_i16(*element);
        }
    }

    pub fn int_list(&mut self, data: &[i32]) {
        self.header(TagType::List);
        self.writer.write_tag(TagType::Int);
        self.writer.write_u32(data.len() as u32);
        for element in data {
            self.writer.write_i32(*element);
        }
    }

    pub fn long_list(&mut self, data: &[i64]) {
        self.header(TagType::List);
        self.writer.write_tag(TagType::Long);
        self.writer.write_u32(data.len() as u32);
        for element in data {
            self.writer.write_i64(*element);
        }
    }

    pub fn float_list(&mut self, data: &[f32]) {
        self.header(TagType::List);
        self.writer.write_tag(TagType::Float);
        self.writer.write_u32(data.len() as u32);
        for element in data {
            self.writer.write_f32(*element);
        }
    }

    pub fn double_list(&mut self, data: &[f64]) {
        self.header(TagType::List);
        self.writer.write_tag(TagType::Double);
        self.writer.write_u32(data.len() as u32);
        for element in data {
            self.writer.write_f64(*element);
        }
    }

    pub fn string_list(&mut self, data: &[&str]) {
        self.header(TagType::List);
        self.writer.write_tag(TagType::String);
        self.writer.write_u32(data.len() as u32);
        for element in data {
            self.writer.write_string(*element);
        }
    }

    pub fn byte_array_list(&mut self, data: &[&[u8]]) {
        self.header(TagType::List);
        self.writer.write_tag(TagType::ByteArray);
        self.writer.write_u32(data.len() as u32);
        for element in data {
            self.writer.write_u32(element.len() as u32);
            self.writer.write_bytes(element);
        }
    }

    pub fn compound_list(&'a mut self) -> CompoundListWriter<'a> {
        self.header(TagType::List);
        CompoundListWriter::new(self.writer)
    }

    // todo: list list, compound list, int array list, long array list

    pub fn is_finished(&self) -> bool {
        self.done
    }
}
