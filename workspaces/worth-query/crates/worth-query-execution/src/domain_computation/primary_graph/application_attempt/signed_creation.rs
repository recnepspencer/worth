//! Native signed-creation progression; decoded values cannot construct this state.
use std::sync::Arc;
use worth_foundational::facade::AspectFieldLocator;
use worth_query_admission::facade::authentication_event::{
    WorthQueryAuthenticationEventIntent, WorthQueryAuthenticationEventSigningOwner,
};
use worth_query_declaration::facade::application_schema::ApplicationOperationBodySigningSpec;
use worth_query_declaration::facade::{
    application_schema::ApplicationSigningProvenanceMetadata,
    authentication::WorthQueryExternalPrincipalIdentity,
};

pub(super) mod canonical;
mod dependencies;
pub(super) mod identity;
mod publication;
mod published_subject;
mod sealing;
mod subject_binding;
pub(super) use publication::publish;
pub(in crate::domain_computation::primary_graph) use published_subject::PublishedCertifiedSubject;
pub use sealing::WorthQuerySignedCreationPreparationDenial;
mod adapter;
pub use adapter::WorthQuerySignedCreationAdapter;

/// The actual candidate's declared missing initializer, not a placeholder value.
pub(super) struct DeferredSigningSlot {
    pub(super) effect: usize,
    pub(super) field: AspectFieldLocator,
    pub(super) spec: ApplicationOperationBodySigningSpec,
    pub(super) representation_capacity: usize,
}

pub(super) struct SignedCreationState<Schema> {
    pub(super) slot: DeferredSigningSlot,
    pub(super) sealed: Option<SealedSigningContext<Schema>>,
    pub(super) authentication: Option<
        worth_query_admission::facade::authentication_event::WorthQueryConsumedAuthenticationEvent<
            Schema,
        >,
    >,
}

pub(super) struct SealedSigningContext<Schema> {
    canonical_byte_capacity: usize,
    provenance_bytes: usize,
    subject_binding: subject_binding::CapturedSubject,
    _dependencies_memory:
        crate::domain_computation::primary_graph::application_contribution::QueryMemoryReservation,
    _metadata_memory:
        crate::domain_computation::primary_graph::application_contribution::QueryMemoryReservation,
    owner: WorthQueryAuthenticationEventSigningOwner<Schema>,
    principal: Arc<WorthQueryExternalPrincipalIdentity>,
    intent: WorthQueryAuthenticationEventIntent,
    metadata: ApplicationSigningProvenanceMetadata,
    logical_request: [u8; 32],
    input: [u8; 32],
    operation_meaning: [u8; 32],
    effect_identity: [u8; 32],
    subject: worth_relational::facade::identity::EntityId,
    signer: worth_relational::facade::identity::EntityId,
}
