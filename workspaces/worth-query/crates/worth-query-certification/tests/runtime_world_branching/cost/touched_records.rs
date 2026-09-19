use worth_query_host::facade::primary_graph;

use crate::world::CourtroomWorld;

pub(crate) fn assert_touched_record_slopes() {
    for touched_records in [1, 8, 64] {
        let world = CourtroomWorld::publish_with_intent_population("blocked", touched_records);
        let branch = world.application.current_world();
        let receipt = world.change_inputs_on_branch(branch, touched_records);
        let work = receipt
            .mutation_work()
            .expect("the performed public mutation must carry owner-derived work");
        eprintln!(
            "axis=TouchedRecords requested={touched_records} commit_actual={} changed={} application_touch_scopes={} installed_touch_targets={} decision={} proposed={} invariant_state={} invariant_work={} covariation=1 provider idempotency record",
            work.touched_record_count(),
            receipt.changed_record_count(),
            work.performed_application_touches_admitted(),
            work.performed_touch_targets_materialized(),
            work.decision_fact_count(),
            work.proposed_fact_count(),
            work.invariant_state_fact_count(),
            work.invariant_work_units()
        );
        assert_eq!(
            work.performed_application_touches_admitted(),
            1,
            "one installed application touch scope admits the scaled edit"
        );
        assert_eq!(receipt.changed_record_count(), touched_records + 1);
        assert_eq!(work.touched_record_count(), touched_records + 1);
        assert_eq!(
            work.preimage_mutation_targets_materialized(),
            0,
            "this mutation requests no aftermath preimage"
        );
        assert_eq!(
            work.performed_touch_targets_materialized(),
            6,
            "the installed amendment scope checks its six declared fields once"
        );
        assert_eq!(work.decision_fact_count(), touched_records + 1);
        assert_eq!(work.proposed_fact_count(), touched_records);
        assert_eq!(work.invariant_state_fact_count(), touched_records);
        assert_eq!(
            work.invariant_work_units(),
            touched_records as u64,
            "one fixed Input decision read is charged for each touched record"
        );
        assert_eq!(
            receipt.terminal().kind(),
            primary_graph::WorthQueryApplicationCommitTerminalKind::Executed
        );
        assert_eq!(receipt.terminal().attempt_resources_released(), Some(true));
    }
}
