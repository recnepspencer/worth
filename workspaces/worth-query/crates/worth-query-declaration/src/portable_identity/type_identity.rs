use std::borrow::Cow;

/// Stable semantic identity for a Rust type axis retained by a portable package.
///
/// The value is declaration metadata, not runtime authority. Declarations must
/// still pass schema validation before the identity can enter package meaning.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryPortableTypeIdentity(Cow<'static, str>);

impl WorthQueryPortableTypeIdentity {
    #[doc(hidden)]
    pub const fn declared(value: &'static str) -> Self {
        Self(Cow::Borrowed(value))
    }

    /// Reconstruct descriptive identity text received from an untrusted boundary.
    ///
    /// This value carries no declaration membership or package authority.
    pub fn from_untrusted(value: String) -> Self {
        Self(Cow::Owned(value))
    }

    pub const fn as_str(&self) -> &str {
        match &self.0 {
            Cow::Borrowed(value) => value,
            Cow::Owned(value) => value.as_str(),
        }
    }

    #[cfg(test)]
    pub(crate) const fn declared_name(&self) -> &'static str {
        match &self.0 {
            Cow::Borrowed(value) => value,
            Cow::Owned(_) => panic!("portable type declarations require static identity text"),
        }
    }

    /// Heap storage retained beyond the identity's inline representation.
    pub fn retained_heap_bytes(&self) -> usize {
        match &self.0 {
            Cow::Borrowed(_) => 0,
            Cow::Owned(value) => value.capacity(),
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.as_str().is_empty()
            && self.as_str().trim() == self.as_str()
            && !self
                .as_str()
                .chars()
                .any(|character| character.is_whitespace() || character.is_control())
    }
}

#[cfg(test)]
mod storage_tests {
    use super::*;
    #[test]
    fn retained_identity_bytes_count_capacity_only_for_owned_text() {
        assert_eq!(
            WorthQueryPortableTypeIdentity::declared("declared").retained_heap_bytes(),
            0
        );
        let mut text = String::with_capacity(128);
        text.push_str("owned");
        let capacity = text.capacity();
        assert_eq!(
            WorthQueryPortableTypeIdentity::from_untrusted(text).retained_heap_bytes(),
            capacity
        );
    }
}
