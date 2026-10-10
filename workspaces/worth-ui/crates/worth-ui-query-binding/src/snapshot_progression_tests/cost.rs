#[derive(Debug, Eq, PartialEq)]
struct MeasurementDerivationCost {
    observations: usize,
    declaration_checks: usize,
    indexed_slot_lookups: usize,
    indexed_accesses: usize,
    native_refinement_checks: usize,
    admitted_derivations: usize,
    fact_scans: usize,
    row_scans: usize,
    path_parses: usize,
}

#[test]
fn one_declared_measurement_is_independent_of_unrelated_projected_width() {
    let narrow = measurement_cost("snapshot-cost-narrow", 1);
    let broad = measurement_cost("snapshot-cost-broad", 128);

    assert_eq!(narrow, broad);
    assert_eq!(
        narrow,
        MeasurementDerivationCost {
            observations: 1,
            declaration_checks: 1,
            indexed_slot_lookups: 2,
            indexed_accesses: 1,
            native_refinement_checks: 1,
            admitted_derivations: 1,
            fact_scans: 0,
            row_scans: 0,
            path_parses: 0,
        }
    );
}

fn measurement_cost(label: &str, row_count: usize) -> MeasurementDerivationCost {
    let mut workspace =
        crate::certification::request_custody::with_bootstrap_request(|execution| {
            super::installed_builder()
                .workspace(execution, label)
                .unwrap()
        });
    for index in 0..row_count {
        workspace.with_execution_request_mut(|execution, mut workspace| workspace
            .insert(execution, "WorthUiMeasurement", |measurement| {
                measurement
                    .set_aspect(
                        worth_query::facade::runtime::WorthQueryAspectTouch::from_authoring_ingress_text(
                            "identity.id",
                        )
                        .unwrap(),
                        worth_query::facade::runtime::WorthQueryAuthoredAspectValue::string(
                            format!("measurement-{index:03}"),
                        ),
                    )
                    .set_aspect(
                        worth_query::facade::runtime::WorthQueryAspectTouch::from_authoring_ingress_text(
                            "measurement.value",
                        )
                        .unwrap(),
                        worth_query::facade::runtime::WorthQueryAuthoredAspectValue::native(
                            worth_foundational::AspectValue::Float32(
                                worth_foundational::CanonicalF32::from_f32(240.0 + index as f32),
                            ),
                        ),
                    )
            })
            .unwrap()).expect("workspace host call opens");
    }
    let reference = super::installed_reference(&workspace);
    let settled = super::settle(&reference, &mut workspace);
    let batch = settled.fact().measurement_facts();
    let resolution = batch.key_resolution_counters();
    let access = batch.native_access_counters();
    let refinement = batch.refinement_counters();
    let binding = batch
        .native_access_binding_counters()
        .expect("real Query settlement reports its binding counters");

    assert_eq!(resolution.key_scans(), 0);
    assert_eq!(resolution.path_parses(), 0);
    assert_eq!(binding.declared_key_routes(), 1);
    assert_eq!(binding.declared_key_layout_checks(), 1);
    assert_eq!(binding.lane_shape_checks(), 2);
    assert_eq!(binding.view_registry_inspections(), 0);
    assert_eq!(binding.domain_registry_inspections(), 0);

    MeasurementDerivationCost {
        observations: batch.observations().len(),
        declaration_checks: resolution.declaration_checks(),
        indexed_slot_lookups: resolution.indexed_slot_lookups(),
        indexed_accesses: access.indexed_accesses,
        native_refinement_checks: access.refinement_checks,
        admitted_derivations: refinement.admitted_observation_count(),
        fact_scans: access.fact_scans + binding.fact_scans(),
        row_scans: access.row_scans + binding.row_scans(),
        path_parses: access.path_parses + binding.path_parses(),
    }
}
