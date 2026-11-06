//! `invariant_projection`'s vocabulary, moved whole from `primary_graph::invariant_projection`: its root file's items here, one child per other source file.
pub(crate) mod capacity_stop;
pub(crate) mod identity_encoding;
pub(crate) mod output_availability_candidate;
pub(crate) mod publication_recovery;
pub(crate) mod verification;
pub(crate) mod work_budget;

use crate::domain_computation::vocabulary::execution_runtime::runtime_identity::WorthQueryRuntimeAuthorityIdentity;
use std::cmp::Ordering;
use std::marker::PhantomData;
use std::sync::Arc;
use worth_foundational::facade::ChargedBytes;
use worth_query_declaration::facade::application_program::ApplicationComputationPartition;
use worth_query_installation::facade::ApplicationSchemaBindingIdentity;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::identity::KindId;

/// An entity observed by an invariant projection, typed by its entity kind.
///
/// It is valid only with readers from the same projection authority; any other
/// reader refuses it as foreign.
#[derive(Clone, Debug)]
pub struct WorthQueryInvariantEntityIdentity<Schema, Entity> {
    pub(in crate::domain_computation::primary_graph) entity_id: EntityId,
    pub(in crate::domain_computation::primary_graph) kind: KindId,
    pub(in crate::domain_computation::primary_graph) entity: Arc<str>,
    pub(in crate::domain_computation::primary_graph) authority_identity: u64,
    pub(in crate::domain_computation::primary_graph) _marker: PhantomData<fn() -> (Schema, Entity)>,
}

/// An exact projected entity admitted for the later application effect phase.
/// It is minted only by the operation-typed invariant reader after validating
/// the entity against that reader's authority.
pub struct WorthQueryInvariantMutationTarget<Schema, Entity> {
    pub(in crate::domain_computation::primary_graph) entity_id: EntityId,
    pub(in crate::domain_computation::primary_graph) entity: Arc<str>,
    pub(in crate::domain_computation::primary_graph) runtime_authority:
        WorthQueryRuntimeAuthorityIdentity,
    pub(in crate::domain_computation::primary_graph) binding_identity:
        ApplicationSchemaBindingIdentity,
    pub(in crate::domain_computation::primary_graph) admission_identity:
        crate::domain_computation::vocabulary::authorization::operation_admission::WorthQueryOperationAdmissionIdentity,
    pub(in crate::domain_computation::primary_graph) _marker: PhantomData<fn() -> (Schema, Entity)>,
}

impl<Schema, Entity> WorthQueryInvariantEntityIdentity<Schema, Entity> {
    pub const fn entity_id(&self) -> EntityId {
        self.entity_id
    }

    pub fn entity_name(&self) -> &str {
        &self.entity
    }
}

impl<Schema, Entity> PartialEq for WorthQueryInvariantEntityIdentity<Schema, Entity> {
    fn eq(&self, other: &Self) -> bool {
        self.authority_identity == other.authority_identity
            && self.entity_id == other.entity_id
            && self.kind == other.kind
            && self.entity == other.entity
    }
}

impl<Schema, Entity> Eq for WorthQueryInvariantEntityIdentity<Schema, Entity> {}

impl<Schema, Entity> PartialOrd for WorthQueryInvariantEntityIdentity<Schema, Entity> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<Schema, Entity> Ord for WorthQueryInvariantEntityIdentity<Schema, Entity> {
    fn cmp(&self, other: &Self) -> Ordering {
        (
            self.authority_identity,
            self.entity_id,
            self.kind,
            self.entity.as_ref(),
        )
            .cmp(&(
                other.authority_identity,
                other.entity_id,
                other.kind,
                other.entity.as_ref(),
            ))
    }
}

/// An entity identity encodes exactly what its equality compares: the
/// installation's projection authority, the entity, its kind and its entity
/// name. Two identities from one installation encode alike exactly when they
/// are equal; an identity issued by another installation never matches.
impl<Schema, Entity> serde::Serialize for WorthQueryInvariantEntityIdentity<Schema, Entity> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut identity = serializer.serialize_struct("WorthQueryInvariantEntityIdentity", 4)?;
        identity.serialize_field(
            "authority_identity",
            &identity_encoding::ProjectionAuthorityEncoding(self.authority_identity),
        )?;
        identity.serialize_field("entity_id", &self.entity_id)?;
        identity.serialize_field("kind", &self.kind)?;
        identity.serialize_field("entity", self.entity.as_ref())?;
        identity.end()
    }
}

impl<Schema, Entity> WorthQueryInvariantEntityIdentity<Schema, Entity> {
    #[doc(hidden)]
    pub fn authority_identity_for_test(&self) -> &u64 {
        &self.authority_identity
    }
}

impl<Schema, Entity> ChargedBytes for WorthQueryInvariantEntityIdentity<Schema, Entity> {
    /// Count this held path's complete Arc allocation. Other held paths may
    /// describe the same allocation; this makes no global deduplication claim.
    fn additional_charged_bytes(&self) -> u64 {
        std::alloc::Layout::array::<u8>(self.entity.len())
            .ok()
            .and_then(|payload| std::alloc::Layout::new::<[usize; 2]>().extend(payload).ok())
            .and_then(|(layout, _)| u64::try_from(layout.pad_to_align().size()).ok())
            .unwrap_or(u64::MAX)
    }
}

impl ApplicationComputationPartition for WorthQueryInvariantEntityIdentity<(), ()> {
    const IDENTITY: &'static str = "projection-identity-encoding-test";
}

mod court_doors_for_test;
