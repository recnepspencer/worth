use super::{
    collection_validated_bundle, direct_validated_bundle, expanded_validated_bundle,
    runtime_basis_intent, runtime_resolved_identity,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::facade::foundation::{BasisResolutionMode, CollectionResultFamily};
use crate::facade::policy::{plan_validated_bundle, planning_request_context_for_direct};

fn preflight_for_bundle(
    bundle: &crate::facade::runtime::ValidatedQueryBundle,
) -> Result<crate::facade::foundation::ExecutionPreflightBundle, IdentityDerivationDenial> {
    let request = planning_request_context_for_direct(bundle, runtime_basis_intent()).unwrap();
    let planned = plan_validated_bundle(bundle, request).unwrap();
    let basis = crate::facade::foundation::resolve_snapshot_basis(
        runtime_basis_intent(),
        runtime_resolved_identity(bundle.query().schema_basis().clone())?,
        BasisResolutionMode::RuntimeDirect,
    )
    .unwrap();
    Ok(crate::facade::foundation::preflight_execution_basis(planned, basis).unwrap())
}

#[test]
fn execution_is_deterministic_for_same_preflight_bundle() -> Result<(), IdentityDerivationDenial> {
    let bundle = direct_validated_bundle();
    let preflight = preflight_for_bundle(&bundle)?;

    let first = crate::facade::foundation::execute_preflight_bundle(&preflight).unwrap();
    let second = crate::facade::foundation::execute_preflight_bundle(&preflight).unwrap();

    assert_eq!(
        first.report().result_digest(),
        second.report().result_digest()
    );
    assert_eq!(first.rows(), second.rows());
    assert_eq!(first.counters(), second.counters());
    assert_eq!(first.counters().executor_semantic_rediscovery_count(), 0);

    Ok(())
}

#[test]
fn execution_counters_reflect_planned_shape() -> Result<(), IdentityDerivationDenial> {
    let bundle = direct_validated_bundle();
    let preflight = preflight_for_bundle(&bundle)?;
    let envelope = crate::facade::foundation::execute_preflight_bundle(&preflight).unwrap();

    assert_eq!(envelope.counters().execution_read_operation_count(), 1);
    assert_eq!(envelope.counters().execution_records_examined_count(), 1);
    assert_eq!(envelope.counters().execution_records_emitted_count(), 1);
    assert_eq!(
        envelope.counters().execution_result_shape_binding_count(),
        1
    );
    assert_eq!(envelope.counters().page_width(), 1);
    assert_eq!(envelope.counters().page_truncation_count(), 0);
    assert_eq!(envelope.counters().cursor_advance_count(), 0);
    assert_eq!(envelope.counters().post_read_shape_field_count(), 1);

    Ok(())
}

#[test]
fn expanded_runtime_route_increases_execution_read_surface_counts(
) -> Result<(), IdentityDerivationDenial> {
    let bundle = expanded_validated_bundle();
    let preflight = preflight_for_bundle(&bundle)?;
    let envelope = crate::facade::foundation::execute_preflight_bundle(&preflight).unwrap();

    assert_eq!(envelope.counters().execution_read_operation_count(), 3);
    assert_eq!(envelope.counters().execution_records_examined_count(), 3);
    assert_eq!(envelope.counters().execution_records_emitted_count(), 2);

    Ok(())
}

#[test]
fn collection_execution_counters_reflect_materialization_and_aggregate_breadth(
) -> Result<(), IdentityDerivationDenial> {
    let bundle = collection_validated_bundle();
    let preflight = preflight_for_bundle(&bundle)?;
    let envelope = crate::facade::foundation::execute_preflight_bundle(&preflight).unwrap();

    assert_eq!(envelope.counters().materialized_relation_count(), 1);
    assert_eq!(envelope.counters().page_width(), 2);
    assert_eq!(envelope.counters().page_truncation_count(), 0);
    assert_eq!(envelope.counters().cursor_advance_count(), 1);
    assert_eq!(
        envelope.counters().aggregate_input_count(),
        preflight
            .plan()
            .collection()
            .unwrap()
            .post_read_shaping()
            .aggregate_shape()
            .input_breadth()
            .value()
    );
    assert_eq!(envelope.counters().rollup_input_count(), 0);
    assert_eq!(envelope.counters().derived_field_evaluation_count(), 0);
    assert_eq!(envelope.counters().cdc_output_count(), 0);

    Ok(())
}

#[test]
fn cdc_collection_execution_emits_distinct_payload_and_cdc_counters(
) -> Result<(), IdentityDerivationDenial> {
    let bundle = collection_validated_bundle();
    let request = planning_request_context_for_direct(&bundle, runtime_basis_intent()).unwrap();
    let planned = crate::facade::policy::plan_validated_bundle_for_collection_family(
        &bundle,
        request,
        CollectionResultFamily::CdcCollection,
    )
    .unwrap();
    let basis = crate::facade::foundation::resolve_snapshot_basis(
        runtime_basis_intent(),
        runtime_resolved_identity(bundle.query().schema_basis().clone())?,
        BasisResolutionMode::RuntimeDirect,
    )
    .unwrap();
    let preflight = crate::facade::foundation::preflight_execution_basis(planned, basis).unwrap();
    let envelope = crate::facade::foundation::execute_preflight_bundle(&preflight).unwrap();

    assert!(envelope
        .rows()
        .iter()
        .all(|entry| entry.starts_with("cdc:")));
    assert_eq!(envelope.counters().cursor_advance_count(), 1);
    assert_eq!(envelope.counters().cdc_output_count(), 1);

    Ok(())
}

#[test]
fn aggregate_preflight_carries_distinct_family_and_rollup_shape(
) -> Result<(), IdentityDerivationDenial> {
    let bundle = collection_validated_bundle();
    let request = planning_request_context_for_direct(&bundle, runtime_basis_intent()).unwrap();
    let planned =
        crate::planning::plan_validated_bundle_for_count_aggregate(&bundle, request).unwrap();
    let basis = crate::facade::foundation::resolve_snapshot_basis(
        runtime_basis_intent(),
        runtime_resolved_identity(bundle.query().schema_basis().clone())?,
        BasisResolutionMode::RuntimeDirect,
    )
    .unwrap();
    let preflight = crate::facade::foundation::preflight_execution_basis(planned, basis).unwrap();
    let envelope = crate::facade::foundation::execute_preflight_bundle(&preflight).unwrap();

    assert!(envelope
        .rows()
        .iter()
        .all(|entry| entry.starts_with("result:")));
    assert_eq!(
        preflight
            .plan()
            .collection()
            .unwrap()
            .planning_context()
            .result_family(),
        &CollectionResultFamily::CountAggregate
    );
    assert_eq!(envelope.counters().cursor_advance_count(), 1);
    assert_eq!(envelope.counters().rollup_input_count(), 1);

    Ok(())
}
