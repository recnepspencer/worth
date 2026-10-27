use super::resource_fixture::*;
use crate::domain_computation::execution_resource_admission::*;
use crate::graph_obligation::*;
use std::sync::Arc;
use worth_query_declaration::facade::domain_computation::*;
use worth_query_installation::facade::WorthQueryResourceAccessPurpose as Purpose;

fn support() -> (
    WorthQueryExecutionResourceSupportSnapshot,
    Arc<WorthQueryFixedExecutionCapacity>,
) {
    use worth_query_installation::facade::*;
    let capacity =
        Arc::new(WorthQueryFixedExecutionCapacity::new("resource-read-capacity", 1).unwrap());
    let provider = WorthQueryExecutionResourceSupport::new(
        WorthQueryExecutionProviderFamily::new("resource-read").unwrap(),
        WorthQueryExecutionAccessProductFamily::new("resource-read").unwrap(),
        WorthQueryExecutionAllocatorFamily::new("resource-read").unwrap(),
        WorthQueryExecutionResourceEnvelope::new(
            WorthQuerySemanticScaleRequest::bounded(32),
            WorthQueryResourceLimitRequest::bounded(32),
            WorthQueryExecutionMode::Synchronous,
            None,
            WorthQueryCancellationSafePointFamily::new("resource-read").unwrap(),
        ),
        capacity.clone(),
    );
    (
        WorthQueryExecutionResourceSupportSnapshot::new(
            provider.clone(),
            vec![],
            vec![("primary".to_owned(), provider)],
            vec![],
            None,
        ),
        capacity,
    )
}

#[test]
fn resource_read_retains_exact_capacity_and_drop_or_late_refusal_releases_it() {
    let schema = installed();
    let resource = schema.installed_resource::<Resource>().unwrap();
    let (support, capacity) = support();
    let prepared = prepare_resource_read_graph_work(&resource, Purpose::Close, &support).unwrap();
    assert_eq!(capacity.active_attempts(), 0);
    let mut admit = |_, _| Ok::<(), &'static str>(());
    let first = admit_prepared_resource_read_graph_work(
        &resource,
        Purpose::Close,
        &prepared,
        &support,
        &mut admit,
    )
    .unwrap();
    assert_eq!(
        first.intent().kind(),
        WorthQueryGraphWorkIntentKind::ResourceRead
    );
    assert_eq!(capacity.active_attempts(), 1);
    assert!(matches!(
        admit_prepared_resource_read_graph_work(
            &resource,
            Purpose::Close,
            &prepared,
            &support,
            &mut admit
        ),
        Err(WorthQueryGraphWorkCapacityAdmissionStop::Denial(
            WorthQueryGraphWorkAdmissionDenial::CapacityUnavailable
        ))
    ));
    drop(first);
    assert_eq!(capacity.active_attempts(), 0);
    let mut refuse_after_reservation = |_, _| {
        if capacity.active_attempts() == 1 {
            Err("original admission refused")
        } else {
            Ok(())
        }
    };
    assert!(matches!(
        admit_prepared_resource_read_graph_work(
            &resource,
            Purpose::Close,
            &prepared,
            &support,
            &mut refuse_after_reservation
        ),
        Err(WorthQueryGraphWorkCapacityAdmissionStop::Admission(
            "original admission refused"
        ))
    ));
    assert_eq!(capacity.active_attempts(), 0);
    let plan = admit_prepared_resource_read_graph_work(
        &resource,
        Purpose::Close,
        &prepared,
        &support,
        &mut admit,
    )
    .unwrap();
    let receipt = plan.release();
    assert_eq!(receipt.released_reservation_count(), 1);
    assert_eq!(capacity.active_attempts(), 0);
}

#[test]
fn resource_read_rejects_other_purpose_installation_and_equal_named_support_before_capacity() {
    let schema = installed();
    let resource = schema.installed_resource::<Resource>().unwrap();
    let (support, capacity) = support();
    let prepared = prepare_resource_read_graph_work(&resource, Purpose::Close, &support).unwrap();
    let mut admit = |_, _| Ok::<(), ()>(());
    assert!(matches!(
        admit_prepared_resource_read_graph_work(
            &resource,
            Purpose::State,
            &prepared,
            &support,
            &mut admit
        ),
        Err(WorthQueryGraphWorkCapacityAdmissionStop::Denial(
            WorthQueryGraphWorkAdmissionDenial::ResourceAuthorityMismatch
        ))
    ));
    let foreign = installed();
    let foreign = foreign.installed_resource::<Resource>().unwrap();
    assert!(matches!(
        admit_prepared_resource_read_graph_work(
            &foreign,
            Purpose::Close,
            &prepared,
            &support,
            &mut admit
        ),
        Err(WorthQueryGraphWorkCapacityAdmissionStop::Denial(
            WorthQueryGraphWorkAdmissionDenial::ResourceAuthorityMismatch
        ))
    ));
    let (other_support, other_capacity) = self::support();
    assert_eq!(
        support.identity(),
        other_support.identity(),
        "diagnostic labels deliberately collide"
    );
    assert!(matches!(
        admit_prepared_resource_read_graph_work(
            &resource,
            Purpose::Close,
            &prepared,
            &other_support,
            &mut admit
        ),
        Err(WorthQueryGraphWorkCapacityAdmissionStop::Denial(
            WorthQueryGraphWorkAdmissionDenial::ProviderSupportUnavailable
        ))
    ));
    assert_eq!(capacity.active_attempts(), 0);
    assert_eq!(other_capacity.active_attempts(), 0);
}

#[test]
fn resource_and_query_graph_subjects_cannot_impersonate_each_other() {
    let schema = installed();
    let resource = schema.installed_resource::<Resource>().unwrap();
    let set = resource
        .access_contract(Purpose::Close)
        .retain_graph_obligations_for_admission();
    for intent in [
        WorthQueryGraphWorkIntent::application_query_read(),
        WorthQueryGraphWorkIntent::application_operation_read(),
    ] {
        assert_eq!(
            select_installed_graph_obligations(set.as_ref().clone(), intent)
                .unwrap_err()
                .kind(),
            WorthQueryGraphObligationSelectionDenialKind::SubjectKindMismatch
        );
    }
    let query = schema.installed_query_binding::<Access>().unwrap();
    let set = query.query().retain_graph_obligations_for_admission();
    assert_eq!(
        select_installed_graph_obligations(set, WorthQueryGraphWorkIntent::resource_read())
            .unwrap_err()
            .kind(),
        WorthQueryGraphObligationSelectionDenialKind::SubjectKindMismatch
    );
}
