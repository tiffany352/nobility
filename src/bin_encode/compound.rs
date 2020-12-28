use crate::bin_encode::{CompoundListWriter, NbtWriter, TagWriter};
use crate::TagType;

pub struct CompoundWriter<'a> {
    writer: &'a mut NbtWriter,
    done: bool,
}

impl<'a> CompoundWriter<'a> {
    pub(crate) fn new(writer: &'a mut NbtWriter) -> CompoundWriter<'a> {
        CompoundWriter {
            writer,
            done: false,
        }
    }

    pub fn field<'b>(&'b mut self, name: &'b str) -> TagWriter<'b> {
        TagWriter::new_field(self.writer, name)
    }

    pub fn compound_field<'b>(&'b mut self, name: &'b str) -> CompoundWriter<'b> {
        self.writer.write_tag(TagType::Compound);
        self.writer.write_string(name);
        CompoundWriter::new(self.writer)
    }

    pub fn compound_list_field<'b>(&'b mut self, name: &'b str) -> CompoundListWriter<'b> {
        self.writer.write_tag(TagType::List);
        self.writer.write_string(name);
        CompoundListWriter::new(self.writer)
    }

    pub fn finish(mut self) {
        self.writer.write_tag(TagType::End);
        self.done = true;
    }
}

impl<'a> Drop for CompoundWriter<'a> {
    fn drop(&mut self) {
        if !self.done {
            panic!("finish() must be called on CompoundWriter before it goes out of scope.");
        }
    }
}
