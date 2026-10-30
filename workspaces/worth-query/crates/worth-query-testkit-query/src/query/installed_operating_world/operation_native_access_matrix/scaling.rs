pub use worth_query::facade::{domain, foundation, runtime};

pub use super::fixture::{bind, matrix_workspace, COLLECTION};
pub use super::samples::{matrix_aspect_key, sample_field, scalar_samples};
pub use super::world_scale::assert_unrelated_domains_installed;

pub const TARGET_FIELD: usize = 15;

#[derive(Debug)]
pub struct CaseCounters {
    pub journey: std::collections::BTreeMap<
        String,
        (u64, worth_foundational::FoundationalPerformanceWorkClass),
    >,
    pub access: domain::WorthQueryNativeAccessCounters,
    pub binding: domain::WorthQueryNativeAccessBindingCounters,
    pub resolution: domain::WorthQueryNativeKeyResolutionCounters,
    pub declared_width: usize,
    pub indexed_accesses: usize,
    pub refinement_checks: usize,
    pub resolution_lookups: usize,
}

pub fn run_case(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
    rows: usize,
    unrelated_facts: bool,
    unrelated_world: bool,
) -> CaseCounters {
    let mut workspace = matrix_workspace(request_execution, name, rows, unrelated_world);
    let unrelated_views = if unrelated_world {
        install_unrelated_views(request_execution, &mut workspace, name)
    } else {
        Default::default()
    };
    assert_eq!(unrelated_views.len(), usize::from(unrelated_world) * 8);
    if unrelated_world {
        assert_unrelated_domains_installed(&workspace);
    }
    let bound = bind(&workspace);
    let mut builder = bound
        .consumer_projection_contract()
        .unwrap()
        .projection_request();
    let mut selections = vec![builder
        .select_display_native_field(sample_field(TARGET_FIELD))
        .unwrap()];
    if unrelated_facts {
        for index in 0..scalar_samples().len() {
            if index != TARGET_FIELD {
                selections.push(
                    builder
                        .select_display_native_field(sample_field(index))
                        .unwrap(),
                );
            }
        }
    }
    let request = builder.build().unwrap();
    let mut resolution_lookups = 0;
    let mut first_resolution = None;
    let keys = selections
        .iter()
        .map(|selection| {
            let resolution = request.resolve_native_key(selection).unwrap();
            let counters = resolution.counters();
            assert_constant_resolution(counters);
            resolution_lookups += counters.indexed_slot_lookups;
            first_resolution.get_or_insert(counters);
            resolution.into_key()
        })
        .collect::<Vec<_>>();
    let settled = bound
        .admit_execution_resources(
            (),
            worth_query_testkit_installed_operation_fixture::facade::query::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .execute(&mut workspace)
        .unwrap()
        .publish()
        .unwrap()
        .consume_bound(request)
        .unwrap()
        .settle()
        .unwrap();
    let last_row = rows - 1;
    let last_ordered_value = (0..rows).map(|row| format!("alpha-{row}")).max().unwrap();
    let mut indexed_accesses = 0;
    let mut refinement_checks = 0;
    let mut first_access = None;
    for (index, key) in keys.iter().enumerate() {
        let selected = settled.native_value(key, last_row).unwrap();
        if index == 0 {
            assert_eq!(
                selected.value().scalar(),
                Some(&worth_foundational::facade::AspectValue::String(
                    worth_foundational::facade::InternedString::Raw(last_ordered_value.clone())
                ))
            );
        }
        let counters = selected.counters();
        assert_constant_access(counters);
        indexed_accesses += counters.indexed_accesses;
        refinement_checks += counters.refinement_checks;
        first_access.get_or_insert(counters);
    }
    let access = first_access.expect("every admitted request contains at least one native key");
    let out_of_bounds = settled.native_value(&keys[0], rows).unwrap_err();
    assert_eq!(
        out_of_bounds.kind(),
        domain::WorthQueryNativeAccessDenialKind::RowOutOfBounds
    );
    assert_eq!(out_of_bounds.counters().indexed_accesses, 0);
    CaseCounters {
        journey: settled
            .consumption_cost_snapshot()
            .rows()
            .iter()
            .map(|row| {
                (
                    row.name().to_owned(),
                    (row.observed_count(), row.work_class()),
                )
            })
            .collect(),
        access,
        binding: settled.native_access_binding_counters().unwrap(),
        resolution: first_resolution
            .expect("every admitted request contains at least one native selection"),
        declared_width: keys.len(),
        indexed_accesses,
        refinement_checks,
        resolution_lookups,
    }
}

pub fn assert_constant_resolution(counters: domain::WorthQueryNativeKeyResolutionCounters) {
    assert_eq!(counters.declaration_checks, 1);
    assert_eq!(counters.indexed_slot_lookups, 2);
    assert_eq!(counters.path_matches, 0);
    assert_eq!(counters.key_scans, 0);
    assert_eq!(counters.path_parses, 0);
}

pub fn assert_constant_access(counters: domain::WorthQueryNativeAccessCounters) {
    assert_eq!(counters.authority_checks, 5);
    assert_eq!(counters.indexed_accesses, 1);
    assert_eq!(counters.refinement_checks, 1);
    assert_eq!(counters.fact_scans, 0);
    assert_eq!(counters.row_scans, 0);
    assert_eq!(counters.path_parses, 0);
    assert_eq!(counters.view_registry_inspections, 0);
    assert_eq!(counters.domain_registry_inspections, 0);
}

pub fn install_unrelated_views(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace: &mut runtime::WorthQueryWorkspace,
    prefix: &str,
) -> Vec<runtime::WorthQueryLiveView<runtime::WorthQueryUnrefinedLiveShape>> {
    let mut views = Vec::new();
    for index in 0..8 {
        let aspect = matrix_aspect_key();
        let field = sample_field(index + 1);
        let identity_aspect = worth_foundational::facade::AspectKey::new("identity").unwrap();
        let identity_field = worth_foundational::facade::FieldKey::new("id").unwrap();
        let view = workspace
            .live_view::<runtime::WorthQueryUnrefinedLiveShape>(
                request_execution,
                format!("{prefix}.unrelated-view-{index}"),
                |view| {
                    view.from(COLLECTION)
                        .select([
                            foundation::AspectFieldKey::from_native_keys(
                                &identity_aspect,
                                &identity_field,
                            ),
                            foundation::AspectFieldKey::from_native_keys(&aspect, &field),
                        ])
                        .order_by(foundation::AspectFieldKey::from_native_keys(
                            &identity_aspect,
                            &identity_field,
                        ))
                        .schema_basis(format!("{prefix}.schema-{index}"))
                },
            )
            .unwrap();
        assert_eq!(view.name(), format!("{prefix}.unrelated-view-{index}"));
        views.push(view);
    }
    views
}
