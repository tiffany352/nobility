//! Nobility is a crate for decoding and encoding NBT (Named Binary
//! Tags), the format used by Minecraft for storing data.
//!
//! The decoder is meant to perform minimal allocations - most of its
//! types are views into the original data that was passed in.
//!
//! The encoder is builder-based and does not take in any kind of
//! document structure.

#![doc(html_root_url = "https://docs.rs/nobility/0.2.0")]

/// Contains the implementation of the binary format decoder.
pub mod bin_decode;
/// Contains the implementation of the binary format encoder.
pub mod bin_encode;

/// NBT tags are a 1-byte value used to specify which type is going to
/// follow. The integer values of each enum corresponds to the actual
/// ones used, and `tag as u8` can be used to cast these to their binary
/// representation.
///
/// Tags are sequentially allocated. As of writing (2020), the most
/// recent tag is `TAG_Long_Array`, added in Minecraft 1.12.
#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum TagType {
    /// Used to mark the end of a Compound tag. `TAG_End`, ID 0.
    End = 0,
    /// Contains an `i8.` `TAG_Byte`, ID 1.
    Byte = 1,
    /// Contains an `i16`. `TAG_Short`, ID 2.
    Short = 2,
    /// Contains an `i32`. `TAG_Int`, ID 3.
    Int = 3,
    /// Contains an `i64`. `TAG_Long`, ID 4.
    Long = 4,
    /// Contains an `f32`. `TAG_Float`, ID 5.
    Float = 5,
    /// Contains an `f64`. `TAG_Double`, ID 6.
    Double = 6,
    /// Contains a `&[u8]`. `TAG_Byte_Array`, ID 7.
    ByteArray = 7,
    /// Contains a [bin_decode::NbtString]. `TAG_String`, ID 8.
    String = 8,
    /// Contains a [bin_decode::List] with the list of its elements as a second tag. `TAG_List`, ID 9.
    List = 9,
    /// Contains a [bin_decode::Compound]. This is a key-value map, but ordered. `TAG_Compound`, ID 10.
    Compound = 10,
    /// Contains a [bin_decode::IntArray]. `TAG_Int_Array`, ID 11.
    IntArray = 11,
    /// Contains a [bin_decode::LongArray]. `TAG_Long_Array`, ID 12.
    LongArray = 12,
}
