use binread::{BinRead, BinReaderExt, NullString};
use std::io::Cursor;

#[derive(BinRead)]
#[br(magic = b"DOG", assert(name.len() != 0))]
struct Dog {
    bone_pile_count: u8,

    #[br(big, count = bone_pile_count)]
    bone_piles: Vec<u16>,

    #[br(align_before = 0xA)]
    name: NullString
}

fn main() {
    let mut reader = Cursor::new(b"DOG\x02\x00\x01\x00\x12\0\0Rudy\0");
    let dog: Dog = reader.read_ne().unwrap();
    assert_eq!(dog.bone_piles, &[0x1, 0x12]);
    assert_eq!(dog.name.into_string(), "Rudy")
}
