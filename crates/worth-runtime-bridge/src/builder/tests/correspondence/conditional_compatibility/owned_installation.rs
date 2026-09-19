use worth_signal::facade::{InstalledSignalConditionDecision, SignalConditionalDecisionClass};

use super::{exact_mapping, runtime, Compute};
use crate::builder::tests::correspondence::semantic_dependencies::{
    freshly_installed_dependency, temporal_contract,
};
use crate::facade::{
    BridgeAsyncCompletionRejectionKind, BridgeAsyncRequestTruthViewBasis,
    BridgeConditionalExecutionRequest, BridgeConditionalLocation,
    BridgeConditionalProviderSemantics, BridgeConditionalProviderSet,
    BridgeConditionalResolverContext, BridgeConditionalWakeProvider,
    BridgeOwnedAsyncRequestResponseDeclaration, BridgeOwnedConditionalInstallationRequest,
};

#[path = "owned_installation/retention_budget.rs"]
mod retention_budget;
use retention_budget::source_free_contract;

fn completion_envelope(
    request: &crate::facade::AdmittedBridgeAsyncRequestIdentity,
) -> worth_signal::facade::RawCompletionEnvelope {
    worth_signal::facade::RawCompletionEnvelope::new(
        request.request_handle().request_id(),
        request.request_handle().generation(),
        request.request_handle().branch_epoch(),
        request.attempt(),
        request
            .lowered()
            .resource_descriptor()
            .unwrap()
            .payload_contract_digest()
            .clone(),
        64,
    )
}

struct Wake;

impl BridgeConditionalProviderSemantics for Wake {
    type SemanticContract = &'static str;

    fn semantic_contract(&self) -> Self::SemanticContract {
        "bridge-test-temporal-wake"
    }

    fn retained_heap_bytes(
        &self,
        _semantic_contract: &Self::SemanticContract,
    ) -> Result<
        crate::facade::BridgeConditionalProviderHeapRetention,
        crate::facade::BridgeConditionalProviderRetentionOverflow,
    > {
        Ok(crate::facade::BridgeConditionalProviderHeapRetention::none())
    }
}

impl BridgeConditionalWakeProvider for Wake {
    fn resolve(
        &self,
        _context: BridgeConditionalResolverContext,
    ) -> Result<InstalledSignalConditionDecision, String> {
        Ok(InstalledSignalConditionDecision::Eligible)
    }
}

#[test]
fn bridge_allocates_conditional_signal_topology_from_semantic_dependencies() {
    let mut owner = super::owned_runtime_builder(runtime(exact_mapping(), Vec::new()))
        .expect("Bridge owns the fresh Signal graph");

    let lowering = owner
        .install_owned_conditional(BridgeOwnedConditionalInstallationRequest {
            contract: temporal_contract("query:one"),
            location: BridgeConditionalLocation::operation("query:one"),
            dependencies: vec![freshly_installed_dependency("query:one")],
            providers: BridgeConditionalProviderSet::new()
                .wake(Wake)
                .compute(Compute(7)),
        })
        .expect("semantic dependencies lower without caller-owned Signal capabilities");

    assert_eq!(lowering.location().node_identity(), "query:one");
    assert_eq!(lowering.correspondence_count(), 1);
    assert_eq!(lowering.counters().signal_targets_joined, 1);
    assert_eq!(lowering.counters().signal_contract_installations, 1);
}

#[test]
fn bridge_allocates_and_executes_source_free_initial_conditional() {
    let mut owner = super::owned_runtime_builder(runtime(exact_mapping(), Vec::new()))
        .expect("Bridge owns the fresh Signal graph");
    let lowering = owner
        .install_owned_conditional(BridgeOwnedConditionalInstallationRequest {
            contract: source_free_contract("query:source-free"),
            location: BridgeConditionalLocation::operation("query:source-free"),
            dependencies: Vec::new(),
            providers: BridgeConditionalProviderSet::new().compute(Compute(9)),
        })
        .expect("Bridge allocates a source-free node without a fake correspondence");
    let owner = owner.seal().unwrap();
    let relational = crate::truth_identity_fixtures::truth_snapshot(1, 1);
    let signal_basis = owner
        .admit_conditional_signal_basis(&lowering, owner.admitted_signal_basis())
        .unwrap();
    let mismatch = owner
        .admit_conditional_evaluation(
            crate::facade::BridgeConditionalEvaluationAdmissionRequest::source_present_at_signal_basis(
                &signal_basis,
                &relational,
            ),
        )
        .unwrap_err();
    assert_eq!(
        mismatch.kind(),
        crate::facade::BridgeConditionalDenialKind::SourcePostureMismatch
    );
    let session = owner
        .admit_conditional_evaluation(
            crate::facade::BridgeConditionalEvaluationAdmissionRequest::source_free_at_signal_basis(
                &signal_basis,
            ),
        )
        .expect("source-free admission needs no relational reader");
    let evidence = owner
        .execute_admitted_conditional(
            &session,
            BridgeConditionalExecutionRequest {
                lowering: &lowering,
                query_binding_identity: "source-free-binding",
                query_capability_identity: 1,
                snapshot_identity: "source-free-snapshot",
                truth_branch_identity: None,
                bridge_snapshot_identity: None,
                execution_identity: "source-free-execution",
                attempt: 1,
            },
            &mut (),
        )
        .unwrap();

    assert_eq!(lowering.correspondence_count(), 0);
    assert_eq!(
        evidence.signal().class(),
        SignalConditionalDecisionClass::ComputedChanged
    );
    assert_eq!(
        evidence
            .bridge_execution_counters()
            .snapshot_admission_attempts,
        0
    );
}

#[test]
fn foreign_bridge_cannot_continue_any_owned_request_stage() {
    let mut origin = super::owned_runtime_builder(runtime(exact_mapping(), Vec::new())).unwrap();
    let foreign = super::owned_runtime_builder(runtime(exact_mapping(), Vec::new()))
        .unwrap()
        .seal()
        .unwrap();
    let declaration = origin
        .install_owned_async_request_response(BridgeOwnedAsyncRequestResponseDeclaration::new(
            "foreign-continuation",
            919,
            512,
            2,
            2,
            5,
        ))
        .unwrap();
    let origin = origin.seal().unwrap();
    let request = origin
        .admit_owned_async_request_identity(
            &declaration,
            origin.admitted_signal_basis(),
            BridgeAsyncRequestTruthViewBasis::authoritative(
                crate::truth_identity_fixtures::truth_branch("foreign-continuation"),
                crate::truth_identity_fixtures::truth_commit(1),
                crate::truth_identity_fixtures::truth_snapshot(3, 1),
            ),
        )
        .unwrap();

    let identical = match origin.admit_owned_async_supersession(&request, &request) {
        Err(denial) => denial,
        Ok(_) => panic!("one request occurrence superseded itself"),
    };
    assert_eq!(
        identical.kind(),
        BridgeAsyncCompletionRejectionKind::SupersessionMismatch
    );
    let foreign_supersession = match foreign.admit_owned_async_supersession(&request, &request) {
        Err(denial) => denial,
        Ok(_) => panic!("a foreign runtime validated the request occurrences"),
    };
    assert_eq!(
        foreign_supersession.kind(),
        BridgeAsyncCompletionRejectionKind::ForeignOwnerObservationAuthority
    );

    let timeout_denial = match foreign.advance_owned_async_request_to_timeout(&request, 5) {
        Err(denial) => denial,
        Ok(_) => panic!("foreign runtime advanced the origin timeout"),
    };
    for denial in [
        foreign
            .owned_async_active_request_count(&request)
            .unwrap_err(),
        timeout_denial,
        foreign.retire_owned_async_request(&request).unwrap_err(),
        foreign
            .validate_owned_async_completion_envelope(
                &request,
                completion_envelope(request.request()),
            )
            .unwrap_err(),
    ] {
        assert_eq!(
            denial.kind(),
            BridgeAsyncCompletionRejectionKind::ForeignOwnerObservationAuthority
        );
    }

    let validated = origin
        .validate_owned_async_completion_envelope(&request, completion_envelope(request.request()))
        .unwrap();
    let completion = origin
        .admit_owned_async_completion(&request, &validated)
        .unwrap();
    let denied_ordering = foreign
        .order_owned_async_completion_report(&completion)
        .unwrap_err();
    assert_eq!(
        denied_ordering.kind(),
        BridgeAsyncCompletionRejectionKind::ForeignOwnerObservationAuthority
    );
    assert_eq!(
        origin
            .order_owned_async_completion_report(&completion)
            .unwrap()
            .ordered()
            .len(),
        1
    );
    origin.retire_owned_async_request(&request).unwrap();
    assert_eq!(
        origin.owned_async_active_request_count(&request).unwrap(),
        0
    );
}

#[test]
fn bridge_rejects_owned_dependencies_from_another_conditional_node() {
    let mut owner = super::owned_runtime_builder(runtime(exact_mapping(), Vec::new()))
        .expect("Bridge owns the fresh Signal graph");

    let result = owner.install_owned_conditional(BridgeOwnedConditionalInstallationRequest {
        contract: temporal_contract("query:first"),
        location: BridgeConditionalLocation::operation("query:first"),
        dependencies: vec![freshly_installed_dependency("query:second")],
        providers: BridgeConditionalProviderSet::new()
            .wake(Wake)
            .compute(Compute(7)),
    });
    let Err(denial) = result else {
        panic!("foreign dependency identity must fail before topology admission");
    };

    assert_eq!(
        denial.kind(),
        crate::facade::BridgeConditionalDenialKind::DeclarationCorrespondenceMismatch
    );
}

#[test]
fn effects_indeterminate_observation_cannot_cross_owned_runtimes() {
    let mut source = super::owned_runtime_builder(runtime(exact_mapping(), Vec::new()))
        .expect("source Bridge owns its Signal graph");
    let foreign = super::owned_runtime_builder(runtime(exact_mapping(), Vec::new()))
        .expect("foreign Bridge owns a distinct Signal graph");
    let lowered = source
        .install_owned_async_request_response(BridgeOwnedAsyncRequestResponseDeclaration::new(
            "owned-async-source",
            313,
            512,
            1,
            2,
            5,
        ))
        .expect("owned async source installs");
    let source = source.seal().expect("source runtime seals");
    let foreign = foreign.seal().expect("foreign runtime seals");
    let admission = source
        .admit_owned_async_request_identity(
            &lowered,
            source.admitted_signal_basis(),
            BridgeAsyncRequestTruthViewBasis::authoritative(
                crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
                crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            ),
        )
        .expect("source owner admits request");
    let issuer = admission.effects_indeterminate_issuer();

    let denial = foreign
        .admit_owned_async_effects_indeterminate(issuer.certify(64))
        .expect_err("another owned runtime cannot consume the observation");

    assert_eq!(
        denial.kind(),
        BridgeAsyncCompletionRejectionKind::ForeignOwnerObservationAuthority
    );
}
