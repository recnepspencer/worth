use super::super::request_identity::{
    AdmittedBridgeAsyncRequestIdentity, BridgeAsyncRequestSubscriptionInstance,
    BridgeAsyncRequestTruthViewBasis, BridgeAsyncRequestTruthViewBasisKind,
};

use super::{
    BridgeAsyncClassifiedDeniedCompletion, BridgeAsyncCompletionCounters,
    BridgeAsyncCompletionDenialClass, BridgeAsyncCompletionSupersessionClass,
    BridgeAsyncCompletionSupersessionClassificationRequest,
    BridgeAsyncCompletionSupersessionEvidence, BridgeAsyncCompletionSupersessionRejection,
    BridgeAsyncCompletionSupersessionRejectionKind, BridgeAsyncDeniedCompletion,
};

impl BridgeAsyncClassifiedDeniedCompletion {
    pub fn classify(
        request: BridgeAsyncCompletionSupersessionClassificationRequest,
    ) -> Result<Self, BridgeAsyncCompletionSupersessionRejection> {
        validate_request(&request)?;
        let supersession_class = classify_request(&request)?;
        let counters = counters_for_class(supersession_class);
        let evidence = BridgeAsyncCompletionSupersessionEvidence::new(&request, supersession_class);
        Ok(Self::new(
            request.denied_completion().clone(),
            supersession_class,
            evidence,
            counters,
        ))
    }
}

fn validate_request(
    request: &BridgeAsyncCompletionSupersessionClassificationRequest,
) -> Result<(), BridgeAsyncCompletionSupersessionRejection> {
    let denied_completion = request.denied_completion();
    if !matches!(
        denied_completion.denial_class(),
        BridgeAsyncCompletionDenialClass::Superseded
            | BridgeAsyncCompletionDenialClass::StaleDenied
    ) {
        return Err(BridgeAsyncCompletionSupersessionRejection::new(
            BridgeAsyncCompletionSupersessionRejectionKind::DeniedCompletionClassMismatch,
            "bridge async completion supersession classification requires a stale or superseded denied completion",
        ));
    }
    match (
        denied_completion.request_identity().subscription_instance(),
        request.current_subscription_instance(),
    ) {
        (Some(_), None) => {
            return Err(BridgeAsyncCompletionSupersessionRejection::new(
                BridgeAsyncCompletionSupersessionRejectionKind::MissingCurrentSubscriptionInstance,
                "subscription-backed supersession classification requires the current subscription instance",
            ));
        }
        (None, Some(_)) => {
            return Err(BridgeAsyncCompletionSupersessionRejection::new(
                BridgeAsyncCompletionSupersessionRejectionKind::UnexpectedCurrentSubscriptionInstance,
                "request-response supersession classification must not receive a subscription instance",
            ));
        }
        _ => {}
    }
    if request.preview_discarded()
        && denied_completion
            .request_identity()
            .basis_binding()
            .truth_view_basis_kind()
            != BridgeAsyncRequestTruthViewBasisKind::Preview
    {
        return Err(BridgeAsyncCompletionSupersessionRejection::new(
            BridgeAsyncCompletionSupersessionRejectionKind::PreviewDiscardRequiresPreviewBasis,
            "preview discard supersession requires a denied completion bound to preview truth basis",
        ));
    }
    if let Some(displacing) = request.displacing_request_identity() {
        validate_displacing_identity(denied_completion, displacing)?;
    }
    Ok(())
}

fn classify_request(
    request: &BridgeAsyncCompletionSupersessionClassificationRequest,
) -> Result<BridgeAsyncCompletionSupersessionClass, BridgeAsyncCompletionSupersessionRejection> {
    let denied_completion = request.denied_completion();
    let original_basis = denied_completion
        .request_identity()
        .basis_binding()
        .truth_view_basis();
    if request.preview_discarded() {
        return Ok(BridgeAsyncCompletionSupersessionClass::PreviewDiscarded);
    }
    if preview_basis_drifted(original_basis, request.current_truth_view_basis()) {
        return Ok(BridgeAsyncCompletionSupersessionClass::PreviewBasisDrifted);
    }
    if let Some(current_subscription_instance) = request.current_subscription_instance()
        && subscription_instance_drifted(
            denied_completion.request_identity().subscription_instance(),
            current_subscription_instance,
        )
    {
        return Ok(BridgeAsyncCompletionSupersessionClass::SubscriptionInstanceSuperseded);
    }
    if branch_drifted(original_basis, request.current_truth_view_basis()) {
        return Ok(BridgeAsyncCompletionSupersessionClass::BranchDrifted);
    }
    if original_basis.digest() != request.current_truth_view_basis().digest() {
        return Ok(BridgeAsyncCompletionSupersessionClass::TruthBasisSuperseded);
    }
    if request.displacing_request_identity().is_some() {
        return Ok(BridgeAsyncCompletionSupersessionClass::SignalGenerationSuperseded);
    }
    Err(BridgeAsyncCompletionSupersessionRejection::new(
        BridgeAsyncCompletionSupersessionRejectionKind::MissingDisplacingRequestIdentity,
        "supersession classification requires an explicit displacing request identity when truth basis and subscription instance did not drift",
    ))
}

fn validate_displacing_identity(
    denied_completion: &BridgeAsyncDeniedCompletion,
    displacing_request_identity: &AdmittedBridgeAsyncRequestIdentity,
) -> Result<(), BridgeAsyncCompletionSupersessionRejection> {
    let original = denied_completion.request_identity();
    if original.lowered().declaration_identity()
        != displacing_request_identity.lowered().declaration_identity()
        || original.lowered().lowering_identity()
            != displacing_request_identity.lowered().lowering_identity()
    {
        return Err(BridgeAsyncCompletionSupersessionRejection::new(
            BridgeAsyncCompletionSupersessionRejectionKind::DisplacingRequestIdentityMismatch,
            "bridge async completion supersession requires a displacing request identity admitted from the exact same lowered async source",
        ));
    }
    Ok(())
}

fn branch_drifted(
    original: &BridgeAsyncRequestTruthViewBasis,
    current: &BridgeAsyncRequestTruthViewBasis,
) -> bool {
    match (
        original.truth_branch_identity(),
        current.truth_branch_identity(),
    ) {
        (Some(left), Some(right)) => left != right,
        _ => false,
    }
}

fn preview_basis_drifted(
    original: &BridgeAsyncRequestTruthViewBasis,
    current: &BridgeAsyncRequestTruthViewBasis,
) -> bool {
    if original.kind() != BridgeAsyncRequestTruthViewBasisKind::Preview
        || current.kind() != BridgeAsyncRequestTruthViewBasisKind::Preview
    {
        return false;
    }
    original.preview_active_subscription_identity()
        != current.preview_active_subscription_identity()
        || original.preview_parent_truth_view_basis_digest()
            != current.preview_parent_truth_view_basis_digest()
}

fn subscription_instance_drifted(
    original: Option<&BridgeAsyncRequestSubscriptionInstance>,
    current: &BridgeAsyncRequestSubscriptionInstance,
) -> bool {
    original.map(BridgeAsyncRequestSubscriptionInstance::digest) != Some(current.digest())
}

fn counters_for_class(
    supersession_class: BridgeAsyncCompletionSupersessionClass,
) -> BridgeAsyncCompletionCounters {
    match supersession_class {
        BridgeAsyncCompletionSupersessionClass::TruthBasisSuperseded => {
            BridgeAsyncCompletionCounters::classified_truth_basis_supersession()
        }
        BridgeAsyncCompletionSupersessionClass::BranchDrifted => {
            BridgeAsyncCompletionCounters::classified_branch_drift_supersession()
        }
        BridgeAsyncCompletionSupersessionClass::PreviewBasisDrifted => {
            BridgeAsyncCompletionCounters::classified_preview_basis_drift_supersession()
        }
        BridgeAsyncCompletionSupersessionClass::PreviewDiscarded => {
            BridgeAsyncCompletionCounters::classified_preview_discarded_supersession()
        }
        BridgeAsyncCompletionSupersessionClass::SubscriptionInstanceSuperseded => {
            BridgeAsyncCompletionCounters::classified_subscription_instance_supersession()
        }
        BridgeAsyncCompletionSupersessionClass::SignalGenerationSuperseded => {
            BridgeAsyncCompletionCounters::classified_signal_generation_supersession()
        }
    }
}
