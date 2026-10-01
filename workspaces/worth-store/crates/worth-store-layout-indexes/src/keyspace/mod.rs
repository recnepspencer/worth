mod admission;
mod admission_operation;
mod blob_catalog;
mod blob_identity;
#[cfg(test)]
mod blob_replay_tests;
mod comparator;
mod composite;
mod declaration;
mod denial;
mod encoding;
mod hash_collision;
mod prefix;
mod range;
#[cfg(test)]
mod replay_tests;
mod request_identity;
mod tenant_partition;
#[cfg(test)]
pub(crate) mod tests;
#[cfg(test)]
pub(crate) mod tests_support;
mod value;

pub(crate) use comparator::{
    canonical_bytes_for_key, compare_concrete_physical_keys, declare_comparator_law,
};
pub(crate) use composite::declare_composite_key_ordering;
pub(crate) use declaration::declare_physical_key_domain;
pub(crate) use encoding::require_canonical_key_encoding;
pub(crate) use hash_collision::{
    declare_hash_collision_law, hash_digest_for_key, require_exact_hash_identity_claim,
    verify_hash_identity,
};
pub(crate) use prefix::{prefix_bytes_for_key, prefix_successor_bytes, require_prefix_law};
pub(crate) use range::{
    range_end_bytes_for_key, range_start_bytes_for_key, require_range_bound_law,
};
pub(crate) use tenant_partition::declare_tenant_scoped_key_domain;
pub(crate) use value::admit_root_key;
#[cfg(test)]
pub(crate) use value::admit_segment_key;
pub(crate) use value::{
    admit_blob_identity_key, admit_blob_key, admit_extent_address_key, admit_page_address_key,
    admit_page_key, admit_physical_reference_key, admit_root_manifest_key,
    admit_segment_address_key, admit_wal_key, admit_wal_record_key,
};

pub use admission::{
    physical_key_domain_admission_cases, AdmittedPhysicalKeyDomain,
    PhysicalKeyDomainAdmissionCaseId, PhysicalKeyDomainAdmissionOutcome,
    PhysicalKeyDomainAdmissionView,
};
pub use admission_operation::key_domain_law;
pub use blob_catalog::{BlobCatalogPointKey, BlobCatalogPointKeyDenial};
pub use comparator::{ComparatorBehavior, ComparatorLaw};
pub use composite::{CompositeKeyField, CompositeKeyOrderingLaw};
pub use declaration::{PhysicalKeyDomain, PhysicalKeyDomainWitness};
pub use denial::PhysicalKeyDomainDenial;
pub use encoding::{CanonicalKeyBytes, CanonicalKeyEncoding, EncodingSentinelPolicy};
pub use hash_collision::{HashCollisionBehavior, HashCollisionLaw};
pub use prefix::{PrefixBoundaryBehavior, PrefixLawWitness};
pub use range::{RangeBoundBehavior, RangeBoundLawWitness};
pub use request_identity::AdmittedPhysicalAccessIdentity;
pub use tenant_partition::TenantScopedKeyDomain;
pub use value::{AdmittedConcretePhysicalKey, ConcretePhysicalKeyWitness};
