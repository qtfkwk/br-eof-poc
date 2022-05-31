use binread::{until_eof, BinRead, BinReaderExt, NullString};
use std::io::Cursor;

#[derive(BinRead)]
#[br(magic = b"DOG", assert(name.len() != 0))]
struct Dog {
    bone_pile_count: u8,

    #[br(big, count = bone_pile_count)]
    bone_piles: Vec<u16>,

    #[br(pad_before = 0x2)]
    name: NullString
    // Changed to pad_before so that it is relative to the current position not the start of the
    // dog and independent from the bone pile count (right?).
}

#[derive(BinRead)]
struct Pack {
    #[br(parse_with = until_eof)]
    dogs: Vec<Dog>,
}

#[derive(BinRead)]
#[br(magic = b"CAT", assert(name.len() != 0))]
struct Cat {
    #[br(pad_before = 0x2)]
    name: NullString
}

#[derive(BinRead)]
enum Animal {
    Dog(Dog),
    Cat(Cat),
}

#[derive(BinRead)]
struct Pets {
    #[br(parse_with = until_eof)]
    list: Vec<Animal>,
}

fn main() {
    let mut reader = Cursor::new(b"DOG\x02\x00\x01\x00\x12\0\0Rudy\0");
    let dog: Dog = reader.read_ne().unwrap();
    assert_eq!(dog.bone_piles, &[0x1, 0x12]);
    assert_eq!(dog.name.into_string(), "Rudy");

    // Use same reader/content, but read as a pack this time...
    reader.set_position(0);
    let pack: Pack = reader.read_ne().unwrap();
    assert_eq!(pack.dogs.len(), 1);
    assert_eq!(pack.dogs[0].bone_piles, &[0x1, 0x12]);
    assert_eq!(pack.dogs[0].name.clone().into_string(), "Rudy");

    // Read two dogs into a pack...
    let mut reader = Cursor::new(b"DOG\x02\x00\x01\x00\x12\0\0Rudy\0DOG\x01\x00\x45\0\0Killer\0");
    let pack: Pack = reader.read_ne().unwrap();
    assert_eq!(pack.dogs.len(), 2);
    assert_eq!(pack.dogs[0].bone_piles, &[0x1, 0x12]);
    assert_eq!(pack.dogs[0].name.clone().into_string(), "Rudy");
    assert_eq!(pack.dogs[1].bone_piles, &[0x45]);
    assert_eq!(pack.dogs[1].name.clone().into_string(), "Killer");

    // Create a cat...
    let mut reader = Cursor::new(b"CAT\0\0Fluffy\0");
    let cat: Cat = reader.read_ne().unwrap();
    assert_eq!(cat.name.into_string(), "Fluffy");

    // Try to create a group of pets...
    let mut reader = Cursor::new(b"DOG\x02\x00\x01\x00\x12\0\0Rudy\0CAT\0\0Fluffy\0DOG\x01\x00\x45\0\0Killer\0");
    let pets: Pets = reader.read_ne().unwrap();
    // Blows up with:
    // thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: EnumErrors { pos:
    // 0x2A, variant_errors: [("Dog", Io(Error { kind: UnexpectedEof, message: "failed to fill
    // whole buffer" })), ("Cat", Io(Error { kind: UnexpectedEof, message: "failed to fill whole
    // buffer" }))] }', src/main.rs:72:39
    assert_eq!(pets.list.len(), 3);
    if let Animal::Dog(dog) = &pets.list[0] {
        assert_eq!(dog.bone_piles, &[0x1, 0x12]);
        assert_eq!(dog.name.clone().into_string(), "Rudy");
    }
    if let Animal::Cat(cat) = &pets.list[1] {
        assert_eq!(cat.name.clone().into_string(), "Fluffy");
    }
    if let Animal::Dog(dog) = &pets.list[2] {
        assert_eq!(dog.bone_piles, &[0x45]);
        assert_eq!(dog.name.clone().into_string(), "Killer");
    }
}
