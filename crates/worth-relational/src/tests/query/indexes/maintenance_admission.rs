use super::*;
use crate::branch::AdmittedRelationalBranchBasis;
use crate::indexes::data::{DerivedIndexMaintenanceAdmissionStop, DerivedIndexMaintenanceBudget};

fn fixture() -> (
    RelationalRuntime,
    AdmittedRelationalBranchBasis,
    DerivedIndexBuildRequest,
) {
    let runtime = runtime_with_index_field_aspects();
    let first = create_entity_outcome(&runtime, "selected-field");
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "admitted.selected-field".into(),
        kind: DerivedIndexKind::EntityField {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
        },
        branch_scoped: true,
    });
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let request = DerivedIndexBuildRequest {
        source_commit_id: first.commit.commit_id,
        branch_id: BranchId("main".into()),
        index_ids: vec![index.index_id],
    };
    release_test_commit_snapshot(&runtime, &first);
    (runtime, basis, request)
}

fn budget() -> DerivedIndexMaintenanceBudget {
    DerivedIndexMaintenanceBudget {
        maximum_work_units: 1_000_000,
        maximum_cold_record_slots: 100_000,
        maximum_derived_rows: 100_000,
    }
}

#[test]
fn selected_field_cold_admission_matches_native_build_and_warm_reuse_is_not_cold() {
    let (runtime, basis, request) = fixture();
    let mut work = 0;
    let mut bytes = 0;
    let refreshed = runtime
        .index_authority()
        .refresh_field_indexes_for_basis_admitted(
            request.clone(),
            &basis,
            budget(),
            |units, allocation| {
                work += units;
                bytes += allocation;
                Ok::<_, ()>(())
            },
        )
        .unwrap();
    assert!(work > 0 && bytes > 0);
    assert!(refreshed.work.cold_record_slots > 0);
    let rebuilt = runtime
        .index_authority()
        .build_for_basis(request.clone(), &basis);
    assert!(rebuilt.failed_indexes.is_empty());
    assert_eq!(
        refreshed.generations[0].entries,
        rebuilt.generations[0].entries
    );
    let warm = runtime
        .index_authority()
        .refresh_field_indexes_for_basis_admitted(
            request,
            &basis,
            DerivedIndexMaintenanceBudget {
                maximum_cold_record_slots: 0,
                ..budget()
            },
            |_, _| Ok::<_, ()>(()),
        )
        .unwrap();
    assert_eq!(warm.work.cold_record_slots, 0);
    assert_eq!(warm.work.reused_generations, 1);
    assert_eq!(warm.generations[0].entries, rebuilt.generations[0].entries);
}

#[derive(Debug, Eq, PartialEq)]
enum OriginalStop {
    Work,
    Bytes,
    Cancelled,
}

#[test]
fn selected_field_refusals_preserve_original_stop_and_publish_no_partial_generation() {
    let (measure_runtime, measure_basis, measure_request) = fixture();
    let (mut total_work, mut total_bytes, mut calls) = (0, 0, 0);
    measure_runtime
        .index_authority()
        .refresh_field_indexes_for_basis_admitted(
            measure_request,
            &measure_basis,
            budget(),
            |work, bytes| {
                total_work += work;
                total_bytes += bytes;
                calls += 1;
                Ok::<_, OriginalStop>(())
            },
        )
        .unwrap();
    for (work_limit, byte_limit, cancel_call, expected) in [
        (0, u64::MAX, usize::MAX, OriginalStop::Work),
        (total_work - 1, u64::MAX, usize::MAX, OriginalStop::Work),
        (u64::MAX, total_bytes - 1, usize::MAX, OriginalStop::Bytes),
        (u64::MAX, u64::MAX, calls, OriginalStop::Cancelled),
    ] {
        let (runtime, basis, request) = fixture();
        let before = runtime.current_version_id();
        let (mut work_used, mut bytes_used, mut call) = (0, 0, 0);
        let denied = runtime
            .index_authority()
            .refresh_field_indexes_for_basis_admitted(
                request.clone(),
                &basis,
                budget(),
                |work, bytes| {
                    call += 1;
                    if call == cancel_call {
                        return Err(OriginalStop::Cancelled);
                    }
                    if work > work_limit - work_used {
                        return Err(OriginalStop::Work);
                    }
                    if bytes > byte_limit - bytes_used {
                        return Err(OriginalStop::Bytes);
                    }
                    work_used += work;
                    bytes_used += bytes;
                    Ok(())
                },
            )
            .unwrap_err();
        let DerivedIndexMaintenanceAdmissionStop::Admission(stop) = denied else {
            panic!("original caller stop")
        };
        assert_eq!(stop, expected);
        assert_eq!(runtime.current_version_id(), before);
        assert!(runtime
            .index_access()
            .published_generation_for_observation(request.index_ids[0], &basis.observation(),)
            .is_none());
    }
}
