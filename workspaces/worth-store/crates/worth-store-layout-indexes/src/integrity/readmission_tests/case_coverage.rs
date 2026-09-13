use super::*;

#[test]
fn readmission_families_declare_exactly_the_cases_ordinary_operations_emit() {
    use std::collections::BTreeSet;

    let import_required = || {
        layout_corruption()
            .require_import_readmission(
                admitted_family(),
                import_witness(family(), "import-case-matrix"),
            )
            .into_import_readmission_requirement()
            .unwrap()
    };
    let import_observed = [
        import_readmission()
            .admit(
                import_required(),
                import_witness(family(), "import-case-matrix"),
            )
            .case_id(),
        import_readmission()
            .admit(
                import_required(),
                quarantine_witness(family(), "import-case-quarantine"),
            )
            .case_id(),
        import_readmission()
            .admit(
                import_required(),
                import_witness(other_family(), "import-case-matrix-other"),
            )
            .case_id(),
    ];

    let quarantine_observation = authoritative_quarantine_observation("quarantine-case-matrix");
    let quarantine_required = || {
        layout_corruption()
            .require_observation_bound_recovery_readmission(
                layout_corruption().assess_quarantine_observation(
                    admitted_family(),
                    quarantine_observation.identity().clone(),
                    quarantine_observation.class(),
                ),
                &current_authority("store.new.strategy", "quarantine-case-matrix"),
                current_security_scope("store.new.strategy", "quarantine-case-matrix").witnesses(),
            )
            .unwrap()
            .into_quarantine_readmission_requirement()
            .unwrap()
    };
    let quarantine_observed = [
        quarantine_readmission()
            .admit(
                quarantine_required(),
                observation_bound_witness(
                    family(),
                    &quarantine_observation,
                    "quarantine-case-matrix",
                ),
            )
            .case_id(),
        quarantine_readmission()
            .admit(
                quarantine_required(),
                import_witness(family(), "quarantine-case-import"),
            )
            .case_id(),
        quarantine_readmission()
            .admit(
                quarantine_required(),
                quarantine_witness(family(), "quarantine-case-matrix-other"),
            )
            .case_id(),
    ];

    assert_eq!(
        crate::integrity::import_readmission_cases().collect::<BTreeSet<_>>(),
        import_observed.into_iter().collect()
    );
    assert_eq!(
        crate::integrity::quarantine_readmission_cases().collect::<BTreeSet<_>>(),
        quarantine_observed.into_iter().collect()
    );
}
