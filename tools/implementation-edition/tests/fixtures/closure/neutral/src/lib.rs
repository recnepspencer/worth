include!(concat!(env!("OUT_DIR"), "/generated.rs"));

pub fn evaluate() -> u8 {
    edition_method::evaluate() + GENERATED
}
