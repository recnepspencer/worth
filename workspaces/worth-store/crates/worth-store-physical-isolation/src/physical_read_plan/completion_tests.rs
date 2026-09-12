use crate::stable_physical_read_plan_for_certification_seed;

#[test]
fn completion_preserves_the_admitted_plan_without_claiming_byte_execution() {
    for (root_seed, resident_bytes) in [(17, 64), (29, 8192)] {
        let plan = stable_physical_read_plan_for_certification_seed(root_seed, resident_bytes);
        let root = plan.root();
        let footprint = plan.footprint().declared_footprint_basis();
        let counters = plan.counters();

        let completed = plan.into_execution_ready_handle().complete_plan();

        assert_eq!(completed.read_plan_release().root(), root);
        assert_eq!(completed.read_plan_release().footprint_basis(), footprint);
        assert_eq!(
            completed
                .read_plan_release()
                .protected_references_released(),
            1
        );
        assert_eq!(completed.counters(), counters);
        assert_eq!(completed.counters().resident_bytes(), resident_bytes);
    }
}
