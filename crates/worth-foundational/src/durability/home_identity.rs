/// A home address, represented by the published Store namespace's 16 bytes.
///
/// Reading or describing this value does not prove publication, own a lease,
/// or permit opening a home. Those operations require owner admission.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct HomeIdentity([u8; 16]);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HomeIdentityDenial {
    Zero,
}

impl HomeIdentity {
    pub const fn from_observed_bytes(bytes: [u8; 16]) -> Result<Self, HomeIdentityDenial> {
        if u128::from_le_bytes(bytes) == 0 {
            Err(HomeIdentityDenial::Zero)
        } else {
            Ok(Self(bytes))
        }
    }

    pub const fn bytes(self) -> [u8; 16] {
        self.0
    }
}

impl super::identity_seal::Sealed for HomeIdentity {}
impl super::DurableIdentity for HomeIdentity {}
