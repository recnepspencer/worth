#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UiDslComponentReference(Box<str>);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiDslComponentReferenceDenial {
    DuplicateReference,
}

impl UiDslComponentReference {
    pub fn new(value: impl Into<Box<str>>) -> Option<Self> {
        let value = value.into();
        (!value.is_empty() && value.len() <= 128 && value.is_ascii()).then_some(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn fold_source_revision(&self, digest: &mut u64) {
        for byte in (self.0.len() as u64)
            .to_le_bytes()
            .into_iter()
            .chain(self.0.bytes())
        {
            *digest ^= u64::from(byte);
            *digest = digest.wrapping_mul(0x100_0000_01b3);
        }
    }
}
