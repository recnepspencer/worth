//! Work and memory metering courts for optional query parameters (`G7`),
//! asserting exact nested typed resource exhaustion causes.
use super::super::world::{funded_due, funded_due_with_budget, Host, Seed};
use super::super::*;
use worth_execution::{LeaseDenial, MapKernelStop, WorkCeilingDenial};
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryAdmissionClock, WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_host::facade::application_contribution::{
    WorthQueryAdvancementDenial, WorthQueryManagedComputationResourceDenial as ResourceDenial,
};
use worth_query_host::facade::application_entry::WorthQueryApplicationRequestQueryDenial as Denial;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationOneShotDenialKind as Kind,
    WorthQueryApplicationProjectionDenialKind as ProjectionKind,
    WorthQueryApplicationQueryAdmissionDenialKind as AdmissionKind,
    WorthQueryEntityResolutionDenialKind as EntityResolutionKind,
    WorthQueryPrincipalResolutionDenialKind as PrincipalResolutionKind,
    WorthQueryProviderSessionDenialKind as SessionKind, WorthQueryValueDecodeDenial,
};

fn wide_notes_board(width: usize) -> Vec<Seed> {
    (1..=16)
        .map(|key| {
            (
                key,
                1,
                "task",
                Some("n".repeat(width)),
                "open",
                1,
                1_000_000,
                Some(100),
            )
        })
        .collect()
}

fn cancellable() -> WorthQueryRequestScope {
    let source = WorthQueryCancellationSource::new();
    WorthQueryRequestScope::new(None, source.token(), WorthQueryAdmissionClock::system())
}

fn is_work_resource(denial: &ResourceDenial) -> bool {
    matches!(
        denial,
        ResourceDenial::WorkExhausted
            | ResourceDenial::WorkLimit
            | ResourceDenial::WorkCounterOverflow
    )
}

fn is_memory_resource(denial: &ResourceDenial) -> bool {
    matches!(
        denial,
        ResourceDenial::MemoryLimit { .. }
            | ResourceDenial::PolicyMemoryLimit
            | ResourceDenial::RetainedBytesExhausted
            | ResourceDenial::ScratchCapacityExceeded
            | ResourceDenial::ResultCapacityExceeded
            | ResourceDenial::NativeAllocationRefused { .. }
    )
}

fn is_typed_work_exhaustion(denial: &Denial) -> bool {
    match denial {
        Denial::Execution(denial) => match denial.kind() {
            Kind::WorkLimitExceeded => true,
            Kind::ExecutionRequest(SessionKind::ExecutionResource { denial, .. }) => {
                is_work_resource(&denial)
            }
            Kind::Projection(proj) => matches!(
                proj,
                ProjectionKind::ValueDecode(
                    WorthQueryValueDecodeDenial::Checkpoint(MapKernelStop::WorkCeiling)
                        | WorthQueryValueDecodeDenial::Scope(WorkCeilingDenial::Stopped(
                            MapKernelStop::WorkCeiling
                        ))
                        | WorthQueryValueDecodeDenial::ContentProjectionWorkExceeded
                )
            ),
            _ => false,
        },
        Denial::ExecutionRequest(WorthQueryAdvancementDenial::Resource(denial)) => {
            is_work_resource(denial)
        }
        Denial::Admission(denial) => match denial.kind() {
            AdmissionKind::WorkLimitExceeded => true,
            AdmissionKind::ExecutionRequest(SessionKind::ExecutionResource { denial, .. }) => {
                is_work_resource(&denial)
            }
            _ => false,
        },
        Denial::ScopeResolution(denial) => matches!(
            denial.kind(),
            EntityResolutionKind::Stopped(cause)
                if matches!(
                    cause.as_request_denial(),
                    Some(
                        WorkCeilingDenial::Stopped(
                            MapKernelStop::WorkCeiling | MapKernelStop::WorkCounterOverflow
                        ) | WorkCeilingDenial::Admission(LeaseDenial::WorkLimitExceedsParent)
                    )
                )
        ),
        Denial::PrincipalResolution(denial) => matches!(
            denial.kind(),
            PrincipalResolutionKind::Stopped(Ok(SessionKind::ExecutionResource { denial, .. }))
                if is_work_resource(&denial)
        ),
        _ => false,
    }
}

fn is_typed_memory_exhaustion(denial: &Denial) -> bool {
    match denial {
        Denial::Execution(denial) => match denial.kind() {
            Kind::ResultBufferLimitExceeded => true,
            Kind::ExecutionRequest(SessionKind::ExecutionResource { denial, .. }) => {
                is_memory_resource(&denial)
            }
            Kind::Projection(proj) => matches!(
                proj,
                ProjectionKind::ValueDecode(
                    WorthQueryValueDecodeDenial::Memory(_)
                        | WorthQueryValueDecodeDenial::Reservation(LeaseDenial::MemoryExhausted(_))
                        | WorthQueryValueDecodeDenial::Scope(WorkCeilingDenial::Admission(
                            LeaseDenial::MemoryExhausted(_)
                        ))
                )
            ),
            _ => false,
        },
        Denial::ExecutionRequest(WorthQueryAdvancementDenial::Resource(denial)) => {
            is_memory_resource(denial)
        }
        Denial::Admission(denial) => match denial.kind() {
            AdmissionKind::ReadmissionPreparationMemoryExhausted => true,
            AdmissionKind::ExecutionRequest(SessionKind::ExecutionResource { denial, .. }) => {
                is_memory_resource(&denial)
            }
            _ => false,
        },
        Denial::ScopeResolution(denial) => matches!(
            denial.kind(),
            EntityResolutionKind::Stopped(cause)
                if matches!(
                    cause.as_request_denial(),
                    Some(WorkCeilingDenial::Admission(
                        LeaseDenial::MemoryExhausted(_) | LeaseDenial::MemoryLimitExceedsParent
                    ))
                )
        ),
        Denial::PrincipalResolution(denial) => matches!(
            denial.kind(),
            PrincipalResolutionKind::Stopped(Ok(SessionKind::ExecutionResource { denial, .. }))
                if is_memory_resource(&denial)
        ),
        _ => false,
    }
}

fn minimal_due_budget(host: &Host, intent: &DueBy) -> u64 {
    let (mut low, mut high) = (1_u64, 1 << 24);
    let scope = cancellable();
    assert!(funded_due(host, &scope, high, intent.clone()).is_ok());
    while low < high {
        let middle = low + (high - low) / 2;
        match funded_due(host, &scope, middle, intent.clone()) {
            Ok(_) => high = middle,
            Err(denial) if is_typed_work_exhaustion(&denial) => low = middle + 1,
            Err(other) => panic!("unexpected denial while calibrating work threshold: {other:?}"),
        }
    }
    low
}

fn minimal_due_memory_budget(host: &Host, intent: &DueBy) -> u64 {
    let (mut low, mut high) = (1024_u64, 16 * 1024 * 1024);
    let scope = cancellable();
    assert!(funded_due_with_budget(host, &scope, high, 1 << 24, intent.clone()).is_ok());
    while low < high {
        let middle = low + (high - low) / 2;
        match funded_due_with_budget(host, &scope, middle, 1 << 24, intent.clone()) {
            Ok(_) => high = middle,
            Err(denial) if is_typed_memory_exhaustion(&denial) => low = middle + 1,
            Err(other) => panic!("unexpected denial while calibrating memory threshold: {other:?}"),
        }
    }
    low
}

#[test]
fn where_filter_refuses_when_memory_budget_exhausts_after_parameter_admission() {
    let mut single = wide_notes_board(16);
    single.truncate(1);
    let host = install(&single);
    let absent = DueBy {
        project: 1,
        max_due_days: None,
        text: None,
    };
    let present = DueBy {
        project: 1,
        max_due_days: None,
        text: Some("m".repeat(60 * 1024)),
    };
    let absent_mem = minimal_due_memory_budget(&host, &absent);
    let present_mem = minimal_due_memory_budget(&host, &present);
    assert!(
        present_mem > 60 * 1024 && present_mem > absent_mem,
        "60 KiB optional parameter must raise peak memory above absent baseline: {absent_mem} vs {present_mem}"
    );
    let scope = cancellable();
    assert!(funded_due_with_budget(&host, &scope, present_mem, 1 << 24, present.clone()).is_ok());
    // One byte under the calibrated memory floor passes parameter admission
    // (which peaked earlier) and refuses specifically inside `where_filter`
    // with the nested typed `MemoryLimit` cause.
    match funded_due_with_budget(&host, &scope, present_mem - 1, 1 << 24, present) {
        Err(Denial::Execution(denial)) => {
            assert!(
                matches!(
                    denial.kind(),
                    Kind::ExecutionRequest(SessionKind::ExecutionResource {
                        denial: ResourceDenial::MemoryLimit { .. },
                        ..
                    })
                ),
                "{denial:?}"
            );
            assert_eq!(denial.subject(), "where_filter");
        }
        other => panic!("finite memory exhaustion must refuse inside where_filter: {other:?}"),
    }
}

#[test]
fn optional_parameter_work_scales_when_present_and_short_circuits_when_absent() {
    let host = install(&wide_notes_board(2048));
    // Both `absent_intent` and `matching_large` return all 16 rows, holding
    // projection work identical while `Some(large)` pays parameter and scan work.
    let absent_intent = DueBy {
        project: 1,
        max_due_days: None,
        text: None,
    };
    let matching_large = DueBy {
        project: 1,
        max_due_days: None,
        text: Some("n".repeat(512)),
    };
    let none_budget = minimal_due_budget(&host, &absent_intent);
    let large_budget = minimal_due_budget(&host, &matching_large);
    assert!(
        large_budget > none_budget + 16 * 128,
        "Some(large text) must charge parameter and scanning work beyond short-circuited None: {none_budget} vs {large_budget}"
    );
    // With a non-matching optional needle, residual evaluation is the last
    // metered work so one unit short refuses inside `where_filter`.
    let non_matching = DueBy {
        project: 1,
        max_due_days: None,
        text: Some("absent".repeat(64)),
    };
    let non_matching_budget = minimal_due_budget(&host, &non_matching);
    let scope = cancellable();
    let (rows, _) = funded_due(&host, &scope, non_matching_budget, non_matching.clone())
        .expect("exact minimal budget completes");
    assert!(rows.is_empty());
    match funded_due(&host, &scope, non_matching_budget - 1, non_matching) {
        Err(Denial::Execution(denial)) => {
            assert!(
                matches!(
                    denial.kind(),
                    Kind::ExecutionRequest(SessionKind::ExecutionResource {
                        denial: ResourceDenial::WorkExhausted,
                        ..
                    }) | Kind::WorkLimitExceeded
                ),
                "{denial:?}"
            );
            assert_eq!(denial.subject(), "where_filter");
        }
        other => panic!("one unit short must refuse inside where_filter: {other:?}"),
    }
}
