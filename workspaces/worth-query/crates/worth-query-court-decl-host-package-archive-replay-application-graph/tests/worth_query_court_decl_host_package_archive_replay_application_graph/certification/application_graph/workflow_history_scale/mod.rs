pub(crate) use worth_query_testkit_certification_application_graph::facade::certification::application_graph::workflow_history_scale::*;

#[test]
fn workflow_history_reconstruction_denies_work_and_memory_exhaustion_without_effects() {
    for (budget, expected_subject) in [
        (
            WorthQueryWorkflowHistoryReconstructionBudget::new(1, 128 * 1024 * 1024).unwrap(),
            "workflow history reconstruction inventory ceiling",
        ),
        (
            WorthQueryWorkflowHistoryReconstructionBudget::new(100, 32 * 1024).unwrap(),
            "workflow history reconstruction memory ceiling",
        ),
    ] {
        let mut court = HistoryCourt::with_budget(budget);
        court.fill(4);
        court
            .application
            .runtime()
            .release_workflow_instance_progress_for_test();
        let before = court
            .application
            .runtime()
            .workflow_instance_progress_counters();
        let scope = court
            .application
            .with_host_advancement(|host_phase| {
                court
                    .application
                    .runtime()
                    .capture_certification_cost_scope(
                        &host_phase,
                        court.application.current_world(),
                    )
                    .unwrap()
            })
            .expect("installed host call opens");
        for _ in 0..2 {
            let denial = match propose_authoring_instance(
                &court.application,
                court.instance.clone(),
                court.next_key,
            ) {
                Err(WorthQueryWorkflowProposalPreparationDenial::ProposalPreparation(
                    WorkflowProposalPreparationDenial::Attempt(denial),
                )) => denial,
                Err(other) => panic!("unexpected history reconstruction denial: {other:?}"),
                Ok(_) => panic!("history reconstruction must deny before a publication outcome"),
            };
            assert_eq!(
                denial.kind(),
                WorthQueryApplicationAttemptDenialKind::WorkflowHistoryReconstructionBudgetExceeded
            );
            assert_eq!(denial.subject(), expected_subject);
            let after = court
                .application
                .runtime()
                .workflow_instance_progress_counters();
            assert_eq!(after.retained_charge_bytes(), 0);
            assert_eq!(after.cold_retains(), before.cold_retains());
        }
        let cost = court
            .application
            .with_host_advancement(|host_phase| {
                court
                    .application
                    .runtime()
                    .observe_certification_cost(&host_phase, &scope)
                    .unwrap()
            })
            .expect("installed host call opens");
        assert_eq!(
            cost.world_history_before().installed_commits(),
            cost.world_history_after().installed_commits()
        );
        assert_eq!(cost.world_retention_after().observations(), 0);
    }
}
