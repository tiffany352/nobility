use crate::TagType;
use byteorder::{BigEndian, ByteOrder};
use cesu8::to_java_cesu8;

mod compound;
mod list;
mod tag;

pub use compound::CompoundWriter;
pub use list::CompoundListWriter;
pub use tag::TagWriter;

pub struct NbtWriter {
    output: Vec<u8>,
    done: bool,
}

impl NbtWriter {
    pub fn new() -> NbtWriter {
        NbtWriter {
            output: vec![],
            done: false,
        }
    }

    pub fn root<'a>(&'a mut self, name: &str) -> CompoundWriter<'a> {
        self.done = true;
        self.write_tag(TagType::Compound);
        self.write_string(name);
        CompoundWriter::new(self)
    }

    pub fn finish(self) -> Vec<u8> {
        if !self.done {
            panic!();
        }
        self.output
    }

    pub(crate) fn get_vec(&mut self) -> &mut Vec<u8> {
        &mut self.output
    }

    pub(crate) fn write_i8(&mut self, value: i8) {
        self.output.push(value as u8);
    }

    pub(crate) fn write_i16(&mut self, value: i16) {
        let mut buf = [0, 0];
        BigEndian::write_i16(&mut buf, value);
        self.output.extend(&buf);
    }

    pub(crate) fn write_i32(&mut self, value: i32) {
        let mut buf = [0, 0, 0, 0];
        BigEndian::write_i32(&mut buf, value);
        self.output.extend(&buf);
    }

    pub(crate) fn write_i64(&mut self, value: i64) {
        let mut buf = [0, 0, 0, 0, 0, 0, 0, 0];
        BigEndian::write_i64(&mut buf, value);
        self.output.extend(&buf);
    }

    pub(crate) fn write_u16(&mut self, value: u16) {
        let mut buf = [0, 0];
        BigEndian::write_u16(&mut buf, value);
        self.output.extend(&buf);
    }

    pub(crate) fn write_u32(&mut self, value: u32) {
        let mut buf = [0, 0, 0, 0];
        BigEndian::write_u32(&mut buf, value);
        self.output.extend(&buf);
    }

    pub(crate) fn write_f32(&mut self, value: f32) {
        let mut buf = [0, 0, 0, 0];
        BigEndian::write_f32(&mut buf, value);
        self.output.extend(&buf);
    }

    pub(crate) fn write_f64(&mut self, value: f64) {
        let mut buf = [0, 0, 0, 0, 0, 0, 0, 0];
        BigEndian::write_f64(&mut buf, value);
        self.output.extend(&buf);
    }

    pub(crate) fn write_bytes(&mut self, data: &[u8]) {
        self.output.extend(data);
    }

    pub(crate) fn write_tag(&mut self, tag: TagType) {
        self.output.push(tag as u8);
    }

    pub(crate) fn write_string(&mut self, input: &str) {
        let data = to_java_cesu8(input);
        self.write_u16(data.len() as u16);
        self.write_bytes(&data);
    }
}
