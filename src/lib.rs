//! NBT decoding library.
//!
//! Currently only supports decoding into a tag tree. In the future,
//! this may support encoding and zero-allocation decoding.

pub mod parser;

/// NBT tags are a 1-byte value used to specify which type is going to
/// follow. The integer values of each enum corresponds to the actual
/// ones used, and `tag as u8` can be used to cast these to their binary
/// representation.
#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum TagType {
    End = 0,
    Byte = 1,
    Short = 2,
    Int = 3,
    Long = 4,
    Float = 5,
    Double = 6,
    ByteArray = 7,
    String = 8,
    List = 9,
    Compound = 10,
    IntArray = 11,
    LongArray = 12,
}
