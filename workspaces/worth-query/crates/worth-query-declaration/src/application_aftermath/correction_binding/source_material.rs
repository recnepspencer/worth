//! Mechanism-specific historical material supplied after current disclosure.
use super::{ApplicationCorrectionOutputField, ApplicationCorrectionPreImage};

/// Read-only access to the finite original-output field manifest.
/// Native failure remains owned by the caller of the domain derivation callback.
pub trait ApplicationCorrectionSourceOutputs<Schema> {
    fn field(
        &self,
        field: &ApplicationCorrectionOutputField<Schema>,
    ) -> Result<&worth_foundational::facade::AspectValue, ApplicationCorrectionSourceFieldDenial>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationCorrectionSourceFieldDenial {
    Refused,
}

/// Historical material selected by the original operation's installed mechanism.
/// This read-only value supplies no current operation authority.
pub enum ApplicationCorrectionSourceMaterial<'source, Schema> {
    RecordedInverse(&'source dyn ApplicationCorrectionPreImage<Schema>),
    Compensation(&'source dyn ApplicationCorrectionSourceOutputs<Schema>),
}
