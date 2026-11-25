use worth_query_declaration::archive_decode_authority::ArchiveDecodeAuthority;
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryTypedFamilyIdentity(String);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPortableFamilyIdentityDenial {
    InvalidPortableIdentity,
}

impl WorthQueryTypedFamilyIdentity {
    pub(crate) fn declared(value: &'static str) -> Self {
        Self(value.to_string())
    }

    /// Reconstructs one authority-free family identity from untrusted owned data.
    ///
    /// This checks only the stable portable-identity grammar. It does not prove
    /// that a provider, comparator, condition family, trigger family, or unit is
    /// registered in the current host.
    pub fn from_untrusted_portable_identity(
        _: ArchiveDecodeAuthority,
        value: String,
    ) -> Result<Self, WorthQueryPortableFamilyIdentityDenial> {
        let identity = Self(value);
        if identity.is_portable() {
            Ok(identity)
        } else {
            Err(WorthQueryPortableFamilyIdentityDenial::InvalidPortableIdentity)
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn is_portable(&self) -> bool {
        !self.0.is_empty()
            && self.0.trim() == self.0
            && !self.0.chars().any(char::is_whitespace)
            && self.0.contains('.')
    }
}

pub trait WorthQueryDomainConditionFamily: 'static {
    const PORTABLE_IDENTITY: &'static str;
}

pub trait WorthQueryComparatorFamily: 'static {
    const PORTABLE_IDENTITY: &'static str;
}

pub trait WorthQueryOnDemandTriggerFamily: 'static {
    const PORTABLE_IDENTITY: &'static str;
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum WorthQueryQuantityValueFamily {
    Integer,
    Float32,
    Float64,
}

pub trait WorthQueryQuantityUnit: 'static {
    const PORTABLE_IDENTITY: &'static str;
    const VALUE_FAMILY: WorthQueryQuantityValueFamily;
}
