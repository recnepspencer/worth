//! Serializable physical-integrity facts shared across runtime boundaries.
//!
//! These values are descriptions only. They grant no media access, decoder
//! entry, recovery choice, quarantine mutation, or repair authority.
//!
//! A projection carries the observed scope and posture, not the owner's proof:
//! ```
//! use worth_foundational::physical_integrity_observation::{PhysicalByteRange, PhysicalIntegrityPosture};
//! let observed = (PhysicalByteRange::new(4096, 16384).unwrap(), PhysicalIntegrityPosture::Unknown);
//! assert_eq!(observed.0.end(), 20480);
//! assert!(PhysicalByteRange::new(u64::MAX, 1).is_err());
//! ```
//! Validate untrusted serialized fields and protocol bounds at their ingress;
//! serializability is not an admission mechanism.

mod adapter_evidence;
mod artifact_family;
mod artifact_identity;
mod authority_class;
mod byte_range;
mod disagreement;
mod integrity_posture;
mod physical_generation;
mod quarantine_posture;
mod recovery_option;

pub use adapter_evidence::PhysicalAdapterEvidence;
pub use artifact_family::PhysicalArtifactFamily;
pub use artifact_identity::{PhysicalArtifactIdentity, PhysicalArtifactIdentityDenial};
pub use authority_class::PhysicalAuthorityClass;
pub use byte_range::{PhysicalByteRange, PhysicalByteRangeDenial};
pub use disagreement::PhysicalIntegrityDisagreement;
pub use integrity_posture::PhysicalIntegrityPosture;
pub use physical_generation::PhysicalArtifactGeneration;
pub use quarantine_posture::PhysicalQuarantinePosture;
pub use recovery_option::PhysicalRecoveryOption;
