pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use {
    worth_server::WorthServerDirectDeliveryClass, worth_server::WorthServerDirectDeliveryOutcome,
    worth_server::WorthServerDirectDeliveryRequest, worth_server::WorthServerDirectFreshnessMode,
    worth_server::WorthServerDirectLeaseDeclarationOutcome,
    worth_server::WorthServerQueryHandoffDenialCode, worth_server::WorthServerQueryHandoffInput,
    worth_server::WorthServerQueryHandoffOperation, worth_server::WorthServerQueryRequestedResume,
    worth_server::WorthServerSuccessKind, worth_server::WorthServerSurfaceFamily,
    worth_server::WorthServerTransportClass,
};

pub(crate) use {
    worth_testkit_server::shared_support_query_handoff_fixture::admit_delivery_posture,
    worth_testkit_server::shared_support_query_handoff_fixture::request_input,
    worth_testkit_server::shared_support_query_handoff_fixture::resolve_request_context,
    worth_testkit_server::shared_support_query_handoff_fixture::success,
    worth_testkit_server::shared_support_worth_native_assertions::admitted_named_read,
    worth_testkit_server::shared_support_worth_native_assertions::family_contract_digest,
    worth_testkit_server::shared_support_worth_native_assertions::operator_evidence_record,
    worth_testkit_server::shared_support_worth_native_assertions::worth_native_session,
    worth_testkit_server::shared_support_worth_native_runtime::build_server,
};

pub(crate) fn direct_request(
    requested_resume: WorthServerQueryRequestedResume,
) -> WorthServerDirectDeliveryRequest {
    worth_server::WorthServerDirectDeliveryRequest::new(
        worth_server::WorthServerDirectFreshnessMode::LiveStrict,
        worth_server::WorthServerDirectDeliveryClass::AuthoritativeOrdered,
        requested_resume,
    )
}

pub(crate) fn direct_delivery_success(
    outcome: WorthServerDirectDeliveryOutcome,
) -> worth_server::WorthServerDirectDeliveryContract {
    match outcome {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected successful direct delivery contract, got {other:?}"),
    }
}

pub(crate) fn direct_lease_success(
    outcome: WorthServerDirectLeaseDeclarationOutcome,
) -> worth_server::WorthServerDirectLeaseDeclaration {
    match outcome {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected successful direct lease declaration, got {other:?}"),
    }
}

pub(crate) fn direct_lease_denied(
    outcome: WorthServerDirectLeaseDeclarationOutcome,
) -> worth_server::WorthServerQueryHandoffDenial {
    match outcome {
        worth_proof::TransitionOutcome::Denied(denial) => denial,
        other => panic!("expected denied direct lease declaration, got {other:?}"),
    }
}

pub(crate) fn direct_delivery_denied(
    outcome: WorthServerDirectDeliveryOutcome,
) -> worth_server::WorthServerQueryHandoffDenial {
    match outcome {
        worth_proof::TransitionOutcome::Denied(denial) => denial,
        other => panic!("expected denied direct delivery contract, got {other:?}"),
    }
}
