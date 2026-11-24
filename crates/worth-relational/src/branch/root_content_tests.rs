impl super::RelationalBranchRoot {
    pub(crate) fn assert_derived_inventory_matches_cold(&self) {
        let mut unavailable = 0;
        for region in self.regions.values() {
            let lease = region
                .partition
                .pin_lease()
                .expect("resident root fixture has Full custody");
            let partition = lease.full_view().partition();
            assert_eq!(
                region.allocation_inventory().unwrap(),
                partition.allocation_inventory().unwrap(),
                "partition {:?}: incremental allocation inventory",
                region.partition_id
            );
            let cold_unavailable = partition.entity_arena.lifecycle_counts().unavailable
                + partition.relation_arena.lifecycle_counts().unavailable;
            assert_eq!(
                region.materialization_unavailable_records(),
                cold_unavailable,
                "partition {:?}: incremental unavailable count",
                region.partition_id
            );
            unavailable += cold_unavailable;
        }
        assert_eq!(
            self.regions.materialization_unavailable_records(),
            unavailable
        );
    }
}
