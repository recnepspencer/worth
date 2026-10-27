//! Real committed outbox → move-only recovery → resource monitor → exact return.
use super::{declaration::MonitorInput, resource::Monitor, Configuration};
use crate::document_retention_model::{operator_identity::*, retention_entry::DOCUMENT_IDENTITY};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_query_host::facade::{
    admission::authenticated_principal::*, application_entry::*, primary_graph::*,
};

struct LostResponseTransport {
    state: Arc<super::producer::ProducerState>,
    calls: AtomicUsize,
}
impl WorthQueryExternalEffectTransport for LostResponseTransport {
    fn dispatch(
        &self,
        _: WorthQueryExternalDispatchRequest<'_>,
    ) -> WorthQueryExternalTransportOutcome {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.state
            .external_response_lost
            .store(true, Ordering::SeqCst);
        WorthQueryExternalTransportOutcome::LostResponse
    }
}

pub fn run() {
    let configuration = Configuration::default();
    let application =
        crate::document_retention_model::installation::publish_monitor(configuration.clone());
    let runtime = application.runtime();
    let transport = Arc::new(LostResponseTransport {
        state: configuration.state.clone(),
        calls: AtomicUsize::new(0),
    });
    runtime
        .install_external_effect_transport(transport.clone())
        .unwrap();
    let scope = request_scope();
    let actor = authenticate_operator(runtime.installed_schema(), &scope);
    let source = super::source_binding::prepare(&application, &actor, &scope);
    let original = MonitorInput {
        identity: DOCUMENT_IDENTITY.to_owned(),
        retention_days: 9,
    };
    let outcome = runtime
        .request(&actor, &scope)
        .advanced()
        .mutate(original.clone())
        .without_source()
        .idempotency(&8001_u64)
        .execute_capability_in_program(
            application.program_runtime(),
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
        panic!("actual recoverable effect commit: {outcome:?}")
    };
    assert_eq!(transport.calls.load(Ordering::SeqCst), 1);
    let handle = runtime.mint_recovery_handle(&receipt).unwrap();
    let original_identity = handle.opaque_wire_identity();
    let authority = super::authority::inspect(runtime, &handle, &actor, &scope, &original);

    let cancelled = WorthQueryCancellationSource::new();
    let refused_scope = WorthQueryRequestScope::new(None, cancelled.token());
    cancelled.cancel();
    let stop = runtime
        .request(&actor, &refused_scope)
        .resources()
        .start_for_owned_effect::<Monitor>(1, handle, &authority)
        .err()
        .expect("actual cancelled entry");
    let (row, returned, cause) = stop.into_parts();
    assert!(row.is_none());
    assert_eq!(cause.kind(), WorthQueryResourceStopKind::Request);
    let returned = returned.expect("pre-row refusal returns exact recovery handle");
    assert_eq!(returned.opaque_wire_identity(), original_identity);

    let request = runtime.request(&actor, &scope);
    let resources = request.resources();
    let mut monitor = resources
        .start_for_owned_effect::<Monitor>(1, returned, &authority)
        .unwrap_or_else(|stop| panic!("monitor: {:?}", stop.into_parts().2));
    configuration.state.send_forged(0).unwrap();
    assert!(resources
        .advance_for_owned_effect(&mut monitor, &authority)
        .is_err());
    assert!(!resources.result_available(&monitor.receipt()).unwrap());
    configuration
        .state
        .cleanup_pending
        .store(true, Ordering::SeqCst);
    let failed = resources
        .close_for_effect_recovery(&mut monitor, &authority)
        .err()
        .expect("real provider cleanup pending");
    let (none, cause) = failed.into_parts();
    assert!(
        none.is_none(),
        "unresolved row still owns original recovery handle"
    );
    assert_eq!(cause.kind(), WorthQueryResourceStopKind::Cleanup);
    assert_eq!(
        runtime.recovery_claim_status(&receipt).unwrap(),
        WorthQueryRecoveryClaimStatus::Live
    );
    configuration
        .state
        .cleanup_pending
        .store(false, Ordering::SeqCst);
    let (returned, _) = resources
        .close_for_effect_recovery(&mut monitor, &authority)
        .unwrap_or_else(|stop| panic!("real cleanup: {:?}", stop.into_parts().1));
    assert_eq!(returned.opaque_wire_identity(), original_identity);

    let mut monitor = resources
        .start_for_owned_effect::<Monitor>(1, returned, &authority)
        .unwrap_or_else(|stop| panic!("monitor original effect: {:?}", stop.into_parts().2));
    configuration.state.send_indeterminate(1).unwrap();
    resources
        .advance_for_owned_effect(&mut monitor, &authority)
        .unwrap();
    assert_eq!(
        resources.state(&monitor.receipt()).unwrap(),
        WorthQueryResourceState::EffectRecoveryPending
    );
    assert!(
        !resources.result_available(&monitor.receipt()).unwrap(),
        "genuine uncertainty admission is not ordinary commit"
    );
    assert!(
        resources.close(&mut monitor).is_err(),
        "resource Close cannot discard the recovery owner"
    );
    let (returned, _) = resources
        .close_for_effect_recovery(&mut monitor, &authority)
        .unwrap_or_else(|stop| panic!("exact recovery handoff: {:?}", stop.into_parts().1));
    assert_eq!(returned.opaque_wire_identity(), original_identity);
    assert_eq!(
        runtime.recovery_claim_status(&receipt).unwrap(),
        WorthQueryRecoveryClaimStatus::Live
    );
    assert_eq!(
        transport.calls.load(Ordering::SeqCst),
        1,
        "monitoring/timeout/cleanup never redispatches"
    );
    assert_eq!(configuration.state.counts(), (2, 2, 3));
    // Resource cleanup returned the unresolved effect owner. Dropping the
    // process-local fixture makes no external rollback/reconciliation claim.
    drop(returned);
    super::source_binding::run(&application, &configuration, &actor, &scope, source);
}
