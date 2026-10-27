//! Definition admission refuses before changing the index inventory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DerivedIndexRegistrationDenial {
    IdentityExhausted,
}
