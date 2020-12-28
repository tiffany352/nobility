use crate::bin_encode::{CompoundWriter, NbtWriter};
use crate::TagType;
use byteorder::{BigEndian, ByteOrder};

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

    pub fn element<'b>(&'b mut self) -> CompoundWriter<'b> {
        self.length += 1;
        CompoundWriter::new(self.writer)
    }

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
