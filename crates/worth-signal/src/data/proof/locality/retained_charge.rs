use super::PartitionScopeSet;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

impl RetainedStorageMeasurement for PartitionScopeSet {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        self.0.retained_heap_charge(work)
    }
}

impl RetainedStorageMeasurement for super::DedupedNodeBatch {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self { nodes } = self;
        nodes.retained_heap_charge(work)
    }
}
impl RetainedStorageMeasurement for super::SortedSourceBatch {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self { sources } = self;
        sources.retained_heap_charge(work)
    }
}
impl RetainedStorageMeasurement for super::TouchedScopeSummary {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            seed_scopes,
            inclusion_scopes,
            direct_dirty_scopes,
            maybe_stale_scopes,
            touched_nodes,
            touched_sources,
        } = self;
        seed_scopes
            .retained_heap_charge(work)?
            .checked_add(inclusion_scopes.retained_heap_charge(work)?)?
            .checked_add(direct_dirty_scopes.retained_heap_charge(work)?)?
            .checked_add(maybe_stale_scopes.retained_heap_charge(work)?)?
            .checked_add(touched_nodes.retained_heap_charge(work)?)?
            .checked_add(touched_sources.retained_heap_charge(work)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::output::PartitionSubscription;

    #[test]
    fn scope_set_wire_sequence_and_owned_slot_charge_survive_storage_cutover() {
        let empty = PartitionScopeSet::default();
        assert_eq!(serde_json::to_string(&empty).unwrap(), "[]");
        assert_eq!(
            empty
                .retained_heap_charge(&mut Preparation::new(2))
                .unwrap(),
            Charge::ZERO
        );

        let scopes = PartitionScopeSet::new([PartitionSubscription::whole_partition("rates")]);
        let wire = r#"[{"path":["rates"],"coverage":"Subtree"}]"#;
        assert_eq!(serde_json::to_string(&scopes).unwrap(), wire);
        let decoded: PartitionScopeSet = serde_json::from_str(wire).unwrap();
        assert_eq!(decoded, scopes);
        assert!(
            scopes
                .retained_heap_charge(&mut Preparation::new(4))
                .unwrap()
                .bytes()
                >= std::mem::size_of::<PartitionSubscription>() as u64
        );
    }
}
