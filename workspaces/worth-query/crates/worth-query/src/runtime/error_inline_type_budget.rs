//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryRuntimeError>(
            "workspaces/worth-query/crates/worth-query/src/runtime/error.rs",
            "WorthQueryRuntimeError",
            128 * 1024,
        );
        owner.field::<String>("UnknownOperation.program_id");
        owner.field::<String>("UnknownOperation.operation_id");
        owner.field::<crate::memory_workspace::WorthQuerySnapshotIdentity>(
            "SharedReadStaleBasis.snapshot_identity",
        );
        owner.field::<String>("RetainedRowDecode.view_name");
        owner.field::<&'static str>("RetainedRowDecode.stage");
        owner.field::<String>("RetainedRowDecode.message");
        owner.field::<String>("ComputedDeclaration.view_name");
        owner.field::<&'static str>("ComputedDeclaration.stage");
        owner.field::<String>("ComputedDeclaration.message");
        owner.field::<String>("EffectDeclaration.effect_name");
        owner.field::<&'static str>("EffectDeclaration.stage");
        owner.field::<String>("EffectDeclaration.message");
        owner.field::<String>("LiveSubscriptionInstallation.view_name");
        owner.field::<&'static str>("LiveSubscriptionInstallation.stage");
        owner.field::<String>("LiveSubscriptionInstallation.message");
        owner.field::<WorthQueryAuthorityLane>(
            "ExistingTruthAssertionRequiresAuthorityLane.required_lane",
        );
        owner.field::<String>("IntentCommitDenied.intent_name");
        owner.field::<&'static str>("IntentCommitDenied.stage");
        owner.field::<String>("IntentCommitDenied.message");
        owner.field::<WorthQueryIntentDenialEvidence>("IntentCommitDenied.evidence");
        owner.field::<String>("IntentExecutionRoutingFailed.intent_name");
        owner.field::<&'static str>("IntentExecutionRoutingFailed.stage");
        owner.field::<String>("IntentExecutionRoutingFailed.message");
        owner.field::<WorthQueryIntentExecutionFailureEvidence>(
            "IntentExecutionRoutingFailed.evidence",
        );
        owner.field::<Box<WorthQueryRuntimeError>>("IntentExecutionRoutingFailed.source");
        owner.field::<WorthQueryPreviewPromotionDenialEvidence>(
            "PreviewPromotionWriteFailed.evidence",
        );
        owner.field::<&'static str>("InvariantRegistration.stage");
        owner.field::<String>("InvariantRegistration.message");
        owner.field::<WorthQueryAuthorityLane>("SessionLabelCollision.authority_lane");
        owner.field::<WorthQuerySessionLabel>("SessionLabelCollision.label");
        owner.field::<WorthQuerySessionLabel>("PreviewOperationEffectDenied.label");
        owner.field::<&'static str>("PreviewOperationEffectDenied.stage");
        owner.field::<String>("PreviewOperationEffectDenied.message");
    }
}
