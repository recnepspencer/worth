mod admitted;
mod builder;
mod defaults;
mod denial;

pub use admitted::AdmittedPhysicalRecordResidencyPolicy;
pub use builder::{PhysicalRecordResidencyPolicy, PhysicalRecordResidencyPolicyBuilder};
pub use denial::PhysicalRecordResidencyPolicyDenial;
pub use worth_store_buffer_pool::{
    PhysicalOperationAllocationScope, PhysicalResidencyDenial, PhysicalResidencyDimension,
    PhysicalSpeculativeWorkKind,
};

/// The proof-bearing result of physical residency policy admission.
///
/// Success carries a sealed `AdmittedPhysicalRecordResidencyPolicy`; denial
/// carries an exact `PhysicalRecordResidencyPolicyDenial`.
pub type PhysicalRecordResidencyPolicyOutcome = worth_proof::DenialTransitionOutcome<
    AdmittedPhysicalRecordResidencyPolicy,
    PhysicalRecordResidencyPolicyDenial,
>;

pub(super) use defaults::canonical_residency_policy;
