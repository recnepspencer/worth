//! Actual predecessor metadata is available only after independent connected admission.
use super::super::courts::{reset, settle};
use super::*;
use worth_query_host::facade::application_entry::WorthQueryApplicationReadObservation;
mod selected;
use selected::{access, navigation_access, read};
fn install() -> Host {
    let schema = Schema::declaration().unwrap();
    in_memory_program(ApplicationProgramAuthoring::<Schema, program::Program>::begin().with_schema(&schema).validated_program().unwrap(),
        schema, (world::Configuration::default(),),
        ApplicationBuildIdentity::parse("91830000-0000-0000-0000-000000005356").unwrap(),
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        |_, graph, installed| { world::seed(graph, installed, 1, Some(10))?; policy::seed(graph) }).unwrap()
}
fn mutate(
    host: &Host,
    request: &family::Request<'_, '_, '_>,
    key: &str,
    value: u64,
) -> WorthQueryApplicationCommitReceipt {
    let outcome = request
        .advanced()
        .mutate(edit::Intent {
            key: key.into(),
            value,
        })
        .without_source()
        .idempotency(&format!("predecessor-inspection-{key}-{value}"))
        .execute_in_program::<program::Program>(
            host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    match outcome {
        WorthQueryApplicationMutationOutcome::Committed { receipt, .. } => receipt,
        other => panic!("{other:?}"),
    }
}
fn retain_output(
    host: &Host,
    request: &family::Request<'_, '_, '_>,
) -> WorthQueryApplicationReadObservation {
    let mut demand = request
        .demand(family::predecessor_family::Demand("a".into()))
        .retain_program_outputs::<program::Program, program::Tree>(host)
        .unwrap_or_else(|cause| panic!("{:?}", cause.denial()));
    for _ in 0..128 {
        demand = match demand.advance(request) {
            WorthQueryProgramOutputOperationAdvance::Pending(next) => next,
            WorthQueryProgramOutputOperationAdvance::Settled(settled) => {
                let read = settled.settlement().root_observation().retain();
                settled.deliver_current(request).unwrap();
                settled
                    .close(request)
                    .unwrap_or_else(|cause| panic!("{:?}", cause.denial()));
                return read;
            }
            WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => panic!("{denial:?}"),
            WorthQueryProgramOutputOperationAdvance::Terminal(terminal) => {
                panic!("{:?}", terminal.denial())
            }
        };
    }
    panic!("predecessor demand did not settle");
}
#[test]
fn connected_inspection_borrows_authentic_old_keys_without_current_dependency_authority() {
    let host = install();
    reset(false);
    PROBE.with(|probe| probe.borrow_mut().foreign = true);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    settle(&request, &host).unwrap();
    drop(mutate(&host, &request, "b", 20));
    let source = mutate(&host, &request, "a", 2);
    let output = retain_output(&host, &request);
    let (after, values) = read(&host, &principal, &scope, &output);
    assert_eq!(values, vec![9002]);
    PROBE.with(|probe| assert_eq!(probe.borrow().values.last(), Some(&Ok(Some(10)))));
    let report = request
        .inspect::<output::Binding>(&after)
        .why_observed()
        .caused_by::<Disclosure>(&source)
        .reference_only()
        .execute_governed(
            access(&host, &principal, &scope),
            access(&host, &principal, &scope),
            access(&host, &principal, &scope),
            access(&host, &principal, &scope),
        )
        .unwrap();
    let historical = report
        .predecessor_reads()
        .expect("real predecessor execution retained its metadata");
    assert_eq!(historical.len(), 1);
    assert_eq!(historical[0].operation(), ReadOldInputs::IDENTIFIER);
    let old_value_entity = historical[0]
        .keys()
        .find_map(|key| match key {
            WorthQueryApplicationFactKey::Field { entity_id, .. } => Some(*entity_id),
            _ => None,
        })
        .expect("the original old B.Value read, not C's source authorization key");
    let current_b = request
        .query(query::model::Read("b".into()))
        .execute()
        .unwrap();
    assert_eq!(current_b.rows()[0].value, 20);
    assert_eq!(old_value_entity, current_b.rows()[0].native_id);
    assert!(historical[0].keys().all(|key| !matches!(
        key,
        WorthQueryApplicationFactKey::SourceReadAuthorization(_)
    )));
    let reads = PROBE.with(|probe| probe.borrow().values.len());
    drop(mutate(&host, &request, "b", 30));
    settle(&request, &host).unwrap();
    PROBE.with(|probe| {
        assert_eq!(
            probe.borrow().values.len(),
            reads,
            "old B.Value is metadata, never C's dependency"
        )
    });
    assert_eq!(historical[0].operation(), ReadOldInputs::IDENTIFIER);
    assert_eq!(
        historical[0]
            .keys()
            .filter(|key| matches!(key, WorthQueryApplicationFactKey::Field { .. }))
            .count(),
        1
    );
    report.close();
    // Query metadata and both diagnostics cannot lend the independent source metadata policy.
    let denied = request
        .inspect::<output::Binding>(&after)
        .why_observed()
        .caused_by::<Disclosure>(&source)
        .reference_only()
        .execute_governed(
            access(&host, &principal, &scope),
            access(&host, &principal, &scope),
            navigation_access(&host, &principal, &scope),
            access(&host, &principal, &scope),
        )
        .err()
        .expect("a genuine navigation grant cannot authorize source inspection metadata");
    assert!(matches!(
        denied.owner_refusal(),
        Some(WorthQueryApplicationInspectionOwnerRefusal::Governance(
            WorthQueryApplicationQueryGovernanceDenialKind::CapabilityMismatch
        ))
    ));
    assert!(
        denied.producer_connection_refusal().is_none(),
        "the independent source metadata gate precedes protected cause lookup"
    );
    let denied = request
        .inspect::<output::Binding>(&after)
        .why_observed()
        .caused_by::<Disclosure>(&source)
        .reference_only()
        .execute_with_diagnostics(
            access(&host, &principal, &scope),
            access(&host, &principal, &scope),
        )
        .err()
        .expect("both diagnostics cannot replace either metadata admission");
    assert!(matches!(
        denied.owner_refusal(),
        Some(WorthQueryApplicationInspectionOwnerRefusal::Governance(
            WorthQueryApplicationQueryGovernanceDenialKind::Required
        ))
    ));
    assert!(denied.producer_connection_refusal().is_none());
    // Genuine foreign capability custody cannot lend any fresh admitted role.
    let foreign = install();
    let foreign_scope = request_scope();
    let foreign_principal = authenticate_operator(foreign.installed_schema(), &foreign_scope);
    let denied = request
        .inspect::<output::Binding>(&after)
        .why_observed()
        .caused_by::<Disclosure>(&source)
        .reference_only()
        .execute_governed(
            access(&host, &principal, &scope),
            access(&foreign, &foreign_principal, &foreign_scope),
            access(&host, &principal, &scope),
            access(&host, &principal, &scope),
        );
    let denied = denied
        .err()
        .expect("forbidden diagnostics must not yield the connected metadata owner");
    assert!(matches!(
        denied.owner_refusal(),
        Some(WorthQueryApplicationInspectionOwnerRefusal::ForeignIssuedAccess)
    ));
    assert!(
        denied.producer_connection_refusal().is_none(),
        "fresh admission refuses before protected cause lookup"
    );
    after.close();
}
