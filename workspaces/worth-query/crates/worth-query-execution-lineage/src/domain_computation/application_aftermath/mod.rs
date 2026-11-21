//! Pure aftermath data, pre-images, causal facts, and external effect models.

pub mod correction;
pub use correction::OriginalCorrectionDisclosure;
pub use worth_query_execution_obligations::facade::aftermath::correction_material;
pub mod causal_fact;
pub mod correction_coordinates;
pub use correction_coordinates::WorthQueryCorrectionSourceCoordinates;
pub use worth_query_execution_obligations::facade::aftermath::derivation_failure;
pub mod external_effect;
pub mod retained_preimage;
pub mod retained_relations;
pub use retained_relations::{
    WorthQueryRelationPreImageDenial, WorthQueryRelationPreImageFailure,
    WorthQueryRetainedPreImageRelation,
};
pub use derivation_failure::WorthQueryAftermathDerivationFailure;
pub use causal_fact::{
    WorthQueryAftermathCausalRole, WorthQueryCommittedAftermathCausality,
    WorthQueryCorrectionLineageBinding, WorthQueryRecordedCorrectionSource,
};
pub use external_effect::{
    bind_dispatch_outbox_create_intent, WorthQueryDispatchOutboxRestoredFields,
    WorthQueryPendingDispatchOutbox,
};
pub use external_effect::{
    derive_external_effect_correlation_identity, ExternalEffectCausalLink,
    ExternalEffectCorrelationBasis,
    ExternalEffectCorrelationIdentity, ExternalEffectNoEffectCause, ExternalEffectPostureIdentity,
    ExternalEffectPostureKind, ExternalEffectProvenNoEffect, ExternalRailTransportFault,
    WorthQueryDispatchOutboxLayout, WorthQueryDispatchOutboxRecord,
    WorthQueryExternalDispatchCausalRelation, WorthQueryExternalDispatchPosture,
    WorthQueryExternalDispatchPostureKind, WorthQueryExternalEffectCausalLadder,
    WorthQueryExternalEffectDispatch, WorthQueryExternalDispatchRequest,
    WorthQueryExternalEffectTransport, WorthQueryExternalTransportOutcome,
};
pub use retained_preimage::{
    demanded_field_slot, WorthQueryPreImageRetentionDenial, WorthQueryRetainedPreImage,
};
#[cfg(test)]
mod retained_preimage_tests;

pub mod recovery_handle {
    pub mod denial;
    pub use denial::{WorthQueryRecoveryHandleDenial, WorthQueryRecoveryHandleDenialKind};
}
pub use recovery_handle::{WorthQueryRecoveryHandleDenial, WorthQueryRecoveryHandleDenialKind};

pub mod dispatch_journal {
    pub use worth_query_execution_contracts::facade::QueryDispatchJournalDenial;
}
