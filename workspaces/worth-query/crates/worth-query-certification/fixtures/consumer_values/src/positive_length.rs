#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
pub struct PositiveLength(u64);

impl PositiveLength {
    pub const fn new(value: u64) -> Option<Self> {
        if value == 0 {
            None
        } else {
            Some(Self(value))
        }
    }

    pub const fn get(value: &Self) -> u64 {
        value.0
    }
}
