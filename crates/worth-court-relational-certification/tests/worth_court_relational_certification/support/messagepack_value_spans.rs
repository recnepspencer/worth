//! Independent DATA inspection keeps complete values, including structured map keys.
use serde::{de::IgnoredAny, Deserialize};
use std::io::Cursor;

fn value<'a>(bytes: &'a [u8], at: &mut usize) -> &'a [u8] {
    let mut decoder = rmp_serde::Deserializer::new(Cursor::new(&bytes[*at..]));
    IgnoredAny::deserialize(&mut decoder).expect("one complete MessagePack value");
    let end = *at + decoder.position() as usize;
    let value = &bytes[*at..end];
    *at = end;
    value
}
pub(crate) fn sequence(bytes: &[u8]) -> Vec<&[u8]> {
    let (count, mut at) = match bytes[0] {
        marker @ 0x90..=0x9f => ((marker & 15) as usize, 1),
        0xdc => (
            u16::from_be_bytes(bytes[1..3].try_into().unwrap()) as usize,
            3,
        ),
        0xdd => (
            u32::from_be_bytes(bytes[1..5].try_into().unwrap()) as usize,
            5,
        ),
        marker => panic!("expected sequence, got {marker:#x}"),
    };
    let values = (0..count).map(|_| value(bytes, &mut at)).collect();
    assert_eq!(at, bytes.len(), "no ignored trailing DATA");
    values
}
pub(crate) fn field<'a>(bytes: &'a [u8], name: &str) -> &'a [u8] {
    let (count, mut at) = match bytes[0] {
        marker @ 0x80..=0x8f => ((marker & 15) as usize, 1),
        0xde => (
            u16::from_be_bytes(bytes[1..3].try_into().unwrap()) as usize,
            3,
        ),
        0xdf => (
            u32::from_be_bytes(bytes[1..5].try_into().unwrap()) as usize,
            5,
        ),
        marker => panic!("expected named map, got {marker:#x}"),
    };
    let mut found = None;
    for _ in 0..count {
        let key: String = rmp_serde::from_slice(value(bytes, &mut at)).unwrap();
        let data = value(bytes, &mut at);
        if key == name {
            assert!(found.replace(data).is_none(), "unique field {name}");
        }
    }
    assert_eq!(at, bytes.len(), "no ignored trailing DATA");
    found.unwrap_or_else(|| panic!("required field {name}"))
}
