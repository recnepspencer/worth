use crate::declarative_live::DeclarativeLiveQueryRequest;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::WorthQueryEvidenceTag;
use crate::lower_runtime_routing::WorthQueryLowerRuntimeSubjectIdentity;
use crate::runtime::{
    SignalInvalidationRoutingReceipt, SubscriptionActivationReceipt,
    WorthQueryBackendAdmissibleMutation, WorthQueryWriteCommand,
};
use crate::subscription::SubscriptionActivationInput;

pub(super) fn live_view_subject_identity(
    view_name: &str,
    request: &DeclarativeLiveQueryRequest,
) -> Result<WorthQueryLowerRuntimeSubjectIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryLowerRuntimeSubjectIdentity::compose("live-view-route-subject")
            .field_value(WorthQueryEvidenceTag::new("view"), view_name)
            .field_value(WorthQueryEvidenceTag::new("target"), request.target())
            .field_shape(
                WorthQueryEvidenceTag::new("shape"),
                request.view_shape().as_str(),
            )
            .field_usize(
                WorthQueryEvidenceTag::new("projection_count"),
                request.query_projection().len(),
            )
            .field_usize(
                WorthQueryEvidenceTag::new("result_count"),
                request.result_fields().len(),
            )
            .seal()?,
    )
}

pub(super) fn write_command_subject_identity(
    command: &WorthQueryWriteCommand,
) -> Result<WorthQueryLowerRuntimeSubjectIdentity, IdentityDerivationDenial> {
    let mut encoder = WorthQueryLowerRuntimeSubjectIdentity::compose("write-command-route-subject")
        .field_shape(
            WorthQueryEvidenceTag::new("family"),
            command.mutation_family().as_str(),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("aspect_operations"),
            command.declared_aspect_operations().len(),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("touched_aspects"),
            command.declared_aspect_touches().len(),
        );
    if let Some(collection) = command.declared_collection_identity() {
        encoder = encoder.field_evidence_identity(
            WorthQueryEvidenceTag::new("collection"),
            collection.evidence_identity(),
        );
    }
    if let Some(identity) = command.declared_entity_identity_ref() {
        let entity_identity = identity.evidence_identity()?;
        encoder =
            encoder.field_evidence_identity(WorthQueryEvidenceTag::new("entity"), &entity_identity);
    }
    Ok(encoder.seal()?)
}

pub(super) fn backend_admissible_mutation_subject_identity(
    mutation: &WorthQueryBackendAdmissibleMutation,
) -> Result<WorthQueryLowerRuntimeSubjectIdentity, IdentityDerivationDenial> {
    let mut encoder = WorthQueryLowerRuntimeSubjectIdentity::compose("write-command-route-subject")
        .field_shape(
            WorthQueryEvidenceTag::new("family"),
            mutation.mutation_family().as_str(),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("aspect_operations"),
            mutation.declared_aspect_operations().len(),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("touched_aspects"),
            mutation.declared_aspect_touches().len(),
        );
    if let Some(collection) = mutation.declared_collection_identity() {
        encoder = encoder.field_evidence_identity(
            WorthQueryEvidenceTag::new("collection"),
            collection.evidence_identity(),
        );
    }
    if let Some(identity) = mutation.declared_entity_identity_ref() {
        let entity_identity = identity.evidence_identity()?;
        encoder =
            encoder.field_evidence_identity(WorthQueryEvidenceTag::new("entity"), &entity_identity);
    }
    Ok(encoder.seal()?)
}

pub(super) fn activation_subject_identity(
    view_name: &str,
    activation: &SubscriptionActivationInput,
    activation_receipt: &SubscriptionActivationReceipt,
) -> Result<WorthQueryLowerRuntimeSubjectIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryLowerRuntimeSubjectIdentity::compose("subscription-activation-route-subject")
            .field_value(WorthQueryEvidenceTag::new("view"), view_name)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("activation_receipt"),
                activation_receipt.receipt_identity(),
            )
            .field_usize(
                WorthQueryEvidenceTag::new("async_request_identity_width"),
                activation.future_selection().async_request_identity().len(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("future_selection_class"),
                activation.future_selection().class().as_str(),
            )
            .seal()?,
    )
}

pub(super) fn signal_invalidation_subject_identity(
    receipt: &SignalInvalidationRoutingReceipt,
) -> Result<WorthQueryLowerRuntimeSubjectIdentity, IdentityDerivationDenial> {
    let commit_identity = receipt.commit_identity().evidence_identity()?;
    let snapshot_identity = receipt.snapshot_identity().evidence_identity()?;
    Ok(
        WorthQueryLowerRuntimeSubjectIdentity::compose("signal-invalidation-route-subject")
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("routing_receipt"),
                receipt.receipt_identity(),
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("commit"), &commit_identity)
            .field_evidence_identity(WorthQueryEvidenceTag::new("snapshot"), &snapshot_identity)
            .field_usize(
                WorthQueryEvidenceTag::new("delta_count"),
                receipt.delta_count(),
            )
            .field_usize(
                WorthQueryEvidenceTag::new("routed_collection_count"),
                receipt.routed_collection_count(),
            )
            .seal()?,
    )
}
