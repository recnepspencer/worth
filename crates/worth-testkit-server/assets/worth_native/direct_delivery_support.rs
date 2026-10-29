use worth_proof::TransitionOutcome;
use worth_server::{
    WorthServerDirectDeliveryClass, WorthServerDirectDeliveryOutcome,
    WorthServerDirectDeliveryRequest, WorthServerDirectFreshnessMode,
    WorthServerDirectLeaseDeclarationOutcome, WorthServerQueryRequestedResume,
};

pub(crate) fn direct_request(
    requested_resume: WorthServerQueryRequestedResume,
) -> WorthServerDirectDeliveryRequest {
    WorthServerDirectDeliveryRequest::new(
        WorthServerDirectFreshnessMode::LiveStrict,
        WorthServerDirectDeliveryClass::AuthoritativeOrdered,
        requested_resume,
    )
}

pub(crate) fn direct_delivery_success(
    outcome: WorthServerDirectDeliveryOutcome,
) -> worth_server::WorthServerDirectDeliveryContract {
    match outcome {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected successful direct delivery contract, got {other:?}"),
    }
}

pub(crate) fn direct_lease_success(
    outcome: WorthServerDirectLeaseDeclarationOutcome,
) -> worth_server::WorthServerDirectLeaseDeclaration {
    match outcome {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected successful direct lease declaration, got {other:?}"),
    }
}

pub(crate) fn direct_lease_denied(
    outcome: WorthServerDirectLeaseDeclarationOutcome,
) -> worth_server::WorthServerQueryHandoffDenial {
    match outcome {
        TransitionOutcome::Denied(denial) => denial,
        other => panic!("expected denied direct lease declaration, got {other:?}"),
    }
}

pub(crate) fn direct_delivery_denied(
    outcome: WorthServerDirectDeliveryOutcome,
) -> worth_server::WorthServerQueryHandoffDenial {
    match outcome {
        TransitionOutcome::Denied(denial) => denial,
        other => panic!("expected denied direct delivery contract, got {other:?}"),
    }
}
