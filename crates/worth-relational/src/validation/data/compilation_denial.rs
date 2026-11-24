//! Failure to compile installed invariant and Source declarations.
use super::registration_conflict::DuplicateCustomInvariantRegistration;

#[derive(Debug)]
pub(crate) enum CustomInvariantCompilationDenial {
    Source(crate::derived_relation::definition::DerivedRelationDenial),
    Duplicate(DuplicateCustomInvariantRegistration),
    Execution(crate::execution::denial_cause::RelationalExecutionDenialCause),
    CapacityOverflow,
}
impl CustomInvariantCompilationDenial {
    /// The descriptive Source binding path has no execution request or storage
    /// admission. Its shared validation can therefore refuse only its meaning.
    pub(crate) fn configuration_source(
        self,
    ) -> crate::derived_relation::definition::DerivedRelationDenial {
        match self {
            Self::Source(cause) => cause,
            Self::Execution(_) | Self::Duplicate(_) | Self::CapacityOverflow => {
                unreachable!("configuration Source validation neither reserves nor executes")
            }
        }
    }
}
