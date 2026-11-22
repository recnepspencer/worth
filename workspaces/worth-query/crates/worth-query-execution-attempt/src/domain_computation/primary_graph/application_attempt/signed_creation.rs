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

pub mod canonical;
pub mod dependencies;
pub mod identity;
pub mod publication;
use worth_query_execution_lineage::facade::domain_computation::primary_graph::application_attempt::signed_creation::published_subject;

pub mod subject_binding;
pub use publication::publish;
pub use published_subject::PublishedCertifiedSubject;

/// The actual candidate's declared missing initializer, not a placeholder value.
pub struct DeferredSigningSlot {
    pub effect: usize,
    pub field: AspectFieldLocator,
    pub spec: ApplicationOperationBodySigningSpec,
}

pub struct SignedCreationState<Schema> {
    pub slot: DeferredSigningSlot,
    pub sealed: Option<SealedSigningContext<Schema>>,
    pub authentication: Option<
        worth_query_admission::facade::authentication_event::WorthQueryConsumedAuthenticationEvent<
            Schema,
        >,
    >,
}

pub struct SealedSigningContext<Schema> {
    pub subject_binding: subject_binding::CapturedSubject,
    pub _dependencies_memory:
        crate::domain_computation::primary_graph::application_contribution::QueryMemoryReservation,
    pub owner: WorthQueryAuthenticationEventSigningOwner<Schema>,
    pub principal: Arc<WorthQueryExternalPrincipalIdentity>,
    pub intent: WorthQueryAuthenticationEventIntent,
    pub metadata: ApplicationSigningProvenanceMetadata,
    pub logical_request: [u8; 32],
    pub input: [u8; 32],
    pub operation_meaning: [u8; 32],
    pub effect_identity: [u8; 32],
    pub subject: worth_relational::facade::identity::EntityId,
    pub signer: worth_relational::facade::identity::EntityId,
}

pub mod sealing;
pub use sealing::{intent, sign, BodyCandidate, WorthQuerySignedCreationPreparationDenial};
