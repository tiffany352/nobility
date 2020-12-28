use crate::bin_encode::{CompoundListWriter, NbtWriter, TagWriter};
use crate::TagType;

/// A builder for a TAG_Compound, allowing fields to be added
/// sequentially.
///
/// # Panics
///
/// This object will panic on drop if finish() is not called. Otherwise,
/// an invalid NBT document would be generated.
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

    /// Create a new field and return a builder for filling in its value.
    pub fn field<'b>(&'b mut self, name: &'b str) -> TagWriter<'b> {
        TagWriter::new_field(self.writer, name)
    }

    /// Creates a compound field. The reason to use this is that
    /// `.field(name).compound()` has too short of a lifetime for the
    /// intermediate TagWriter, and would have to be spread across
    /// multiple variables, causing verbose code.
    pub fn compound_field<'b>(&'b mut self, name: &'b str) -> CompoundWriter<'b> {
        self.writer.write_tag(TagType::Compound);
        self.writer.write_string(name);
        CompoundWriter::new(self.writer)
    }

    /// Creates a list of compounds. The reason to use this is that
    /// `.field(name).compound_list()` has too short of a lifetime for
    /// the intermediate TagWriter, and would have to be spread across
    /// multiple variables, causing verbose code.
    pub fn compound_list_field<'b>(&'b mut self, name: &'b str) -> CompoundListWriter<'b> {
        self.writer.write_tag(TagType::List);
        self.writer.write_string(name);
        CompoundListWriter::new(self.writer)
    }

    /// Finishes the compound tag. This must be called after you're done
    /// appending elements, or a panic will occur on drop.
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
