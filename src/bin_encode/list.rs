use crate::bin_encode::{CompoundWriter, NbtWriter};
use crate::TagType;
use byteorder::{BigEndian, ByteOrder};

/// A builder for a TAG_List of [TAG_Compounds][CompoundWriter].
///
/// # Example
///
/// ```rust
/// # use nobility::bin_encode::NbtWriter;
/// # let mut writer = NbtWriter::new();
/// # {
/// # let some_compound = writer.root("test");
/// let mut player = some_compound;
/// let mut list = player.compound_list_field("Friends");
///
/// {
///     let mut element = list.element();
///     element.field("Name").string("Alice");
///     element.field("Level").int(20);
///     element.finish();
/// }
///
/// {
///     let mut element = list.element();
///     element.field("Name").string("Steve");
///     element.field("Level").int(17);
///     element.finish();
/// }
///
/// // finish() call is required.
/// list.finish();
///
/// # player.finish();
/// # }
/// # let _ = writer.finish();
/// ```
///
/// # Panics
///
/// This object will panic on drop if finish() is not called.
#[derive(Debug)]
pub struct CompoundListWriter<'a> {
    writer: &'a mut NbtWriter,
    start_offset: usize,
    length: usize,
    done: bool,
}

impl<'a> CompoundListWriter<'a> {
    pub(crate) fn new(writer: &'a mut NbtWriter) -> CompoundListWriter<'a> {
        writer.write_tag(TagType::Compound);
        let start_offset = writer.get_vec().len();
        writer.write_u32(0);
        CompoundListWriter {
            writer,
            start_offset,
            length: 0,
            done: false,
        }
    }

    /// Start a new element in the list, returning a CompoundWriter to
    /// build it. `finish` must be called on the builder before
    /// additional elements can be added.
    pub fn element(&mut self) -> CompoundWriter {
        self.length += 1;
        CompoundWriter::new(self.writer)
    }

    /// Must be called before the builder goes out of scope, otherwise
    /// an invalid NBT document would be generated.
    pub fn finish(mut self) {
        self.done = true;
        let mut bytes = [0, 0, 0, 0];
        BigEndian::write_u32(&mut bytes, self.length as u32);
        // Somewhat of a hack, but it makes the interface nicer. Goes
        // back and overwrites the length field with the true value once
        // this builder has been finalized.
        let vec = self.writer.get_vec();
        for (i, byte) in bytes.iter().enumerate() {
            vec[self.start_offset + i] = *byte;
        }
    }
}

impl<'a> Drop for CompoundListWriter<'a> {
    fn drop(&mut self) {
        if !self.done {
            panic!("finish() must be called on CompoundListWriter before going out of scope");
        }
    }
}
