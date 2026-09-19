use super::{NodeColdData, NodeHotData, NodeWarmData};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

impl RetainedStorageMeasurement for NodeHotData {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            state: _,
            dirty_aspects: _,
            dirty_partition_scope_aspects: _,
            aspect_version_header: _,
            dependencies_id: _,
            subscribers_id: _,
            dep_snapshot_id: _,
            pending_cause_set_id: _,
            dependency_revision: _,
        } = self;
        Ok(Charge::ZERO)
    }
}

impl RetainedStorageMeasurement for NodeColdData {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            retained_artifact,
            causality,
            execution_trace,
        } = self;
        retained_artifact
            .retained_heap_charge(work)?
            .checked_add(causality.retained_heap_charge(work)?)?
            .checked_add(execution_trace.retained_heap_charge(work)?)
    }
}

impl RetainedStorageMeasurement for NodeWarmData {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            pending_dependency_revalidation,
            direct_invalidation_basis,
            direct_invalidation_generation: _,
            aspect_version_overrides,
            dirty_partition_scope_payload,
            runtime_artifact_state,
        } = self;
        pending_dependency_revalidation
            .retained_heap_charge(work)?
            .checked_add(direct_invalidation_basis.retained_heap_charge(work)?)?
            .checked_add(aspect_version_overrides.retained_heap_charge(work)?)?
            .checked_add(dirty_partition_scope_payload.retained_heap_charge(work)?)?
            .checked_add(runtime_artifact_state.retained_heap_charge(work)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejected_staged_warm_node_edit_preserves_nested_owned_payloads() {
        use crate::data::aspect::{Aspect, AspectVersion};
        use crate::data::output::ChangedRegion;
        use crate::data::persistent_vector::{
            RetainedVectorCapacityDenial, RetainedVectorCapacityOutcome,
        };
        let aspect = Aspect::new(0);
        let scope = PartitionSubscription::whole_partition("retained");
        let mut warm = NodeWarmData::default();
        warm.aspect_version_overrides.apply_evaluation(
            AspectVersion::zero().with(aspect, 3),
            &[ChangedRegion {
                partition: scope.partition.clone(),
                detail: None,
            }],
        );
        warm.dirty_partition_scope_payload
            .push((aspect, scope.clone()));
        let mut source: PersistentPagedVector<_> = [warm].into_iter().collect();
        source
            .prepare_retained_charge(&mut Preparation::new(1_000))
            .unwrap();
        let mut selected = source.fork_persistent();
        let sibling = selected.clone();
        let before = selected.prepared_retained_charge().unwrap();
        let outcome = selected
            .edit_with_retained_capacity(0, before, &mut Preparation::new(1_000), |node| {
                node.aspect_version_overrides
                    .set_global(AspectVersion::zero().with(aspect, 9));
                node.dirty_partition_scope_payload[0]
                    .1
                    .partition
                    .0
                    .push_str(&"extra".repeat(1_024));
                node.aspect_version_overrides.version_for_scope(
                    aspect,
                    Some(&scope),
                    AspectVersion::zero(),
                )
            })
            .unwrap();
        assert!(matches!(
            outcome,
            RetainedVectorCapacityOutcome::Rejected {
                output: 9,
                denial: RetainedVectorCapacityDenial::CapacityExhausted { .. }
            }
        ));
        assert_eq!(selected[0], sibling[0]);
        assert_eq!(
            selected[0].aspect_version_overrides.version_for_scope(
                aspect,
                Some(&scope),
                AspectVersion::zero()
            ),
            3
        );
        assert_eq!(selected[0].dirty_partition_scope_payload[0].1, scope);
        assert_eq!(selected.prepared_retained_charge().unwrap(), before);
        assert!(selected.shares_storage_with(&sibling));
    }

    #[test]
    fn cold_node_charges_unused_string_capacity_in_artifacts_and_causality() {
        use crate::data::trace::{CausalityMetadata, RetainedDiagnosticArtifact};
        let mut cold = NodeColdData {
            retained_artifact: Some(RetainedDiagnosticArtifact {
                labels: vec![String::new()],
                keyed_family: Some(String::new()),
                keyed_key: Some(String::new()),
                ..Default::default()
            }),
            causality: Some(CausalityMetadata {
                kind: String::new(),
                fields: [(String::new(), String::new())].into_iter().collect(),
            }),
            execution_trace: None,
        };
        let before = cold
            .retained_heap_charge(&mut Preparation::new(1_000))
            .unwrap()
            .bytes();
        let artifact = cold.retained_artifact.as_mut().unwrap();
        let causality = cold.causality.as_mut().unwrap();
        let mut added = 0;
        for payload in [
            &mut artifact.labels[0],
            artifact.keyed_family.as_mut().unwrap(),
            artifact.keyed_key.as_mut().unwrap(),
            &mut causality.kind,
            causality.fields.values_mut().next().unwrap(),
        ] {
            payload.reserve_exact(8_192);
            assert!(payload.is_empty());
            added += payload.capacity() as u64;
        }
        let after = cold
            .retained_heap_charge(&mut Preparation::new(1_000))
            .unwrap()
            .bytes();
        assert_eq!(after - before, added);
        let mut parent: PersistentPagedVector<_> = [Some(Box::new(cold))].into_iter().collect();
        let mut retained = parent.fork_persistent();
        retained[0] = None;
        drop(parent);
        assert!(
            retained
                .retained_heap_charge(&mut Preparation::new(1_000))
                .unwrap()
                .bytes()
                >= after
        );
    }
    use crate::data::output::{ArtifactContinuityToken, OutputIdentity, PartitionSubscription};
    use crate::data::persistent_paged_vector::PersistentPagedVector;
    use crate::data::proof::PartitionScopeSet;
    use crate::data::reuse::{ArtifactFamilyId, ReuseBasis};
    use crate::data::trace::{
        CompactChangedScopeProof, ContinuityAuthorityToken, ReuseOperationalBasis,
        RuntimeArtifactHot, RuntimeArtifactState, RuntimeArtifactWarm,
    };

    #[test]
    fn warm_node_accounts_each_artifact_payload_and_retains_it_under_fork_replacement() {
        let payload = "retained".repeat(4_096);
        let capacity = payload.len() as u64;
        let artifact = RuntimeArtifactState::new(
            RuntimeArtifactHot {
                changed_scopes: CompactChangedScopeProof::new(PartitionScopeSet::new([
                    PartitionSubscription::whole_partition(payload.clone()),
                ])),
                ..Default::default()
            },
            RuntimeArtifactWarm {
                output_identity: Some(OutputIdentity::new(payload.clone())),
                continuity_token: ContinuityAuthorityToken::new(Some(
                    ArtifactContinuityToken::new(payload.clone()),
                )),
                reuse_basis: ReuseOperationalBasis::new(ReuseBasis {
                    artifact_family_basis: Some(ArtifactFamilyId::new(payload)),
                    ..Default::default()
                }),
                ..Default::default()
            },
        );
        assert_eq!(
            artifact
                .retained_heap_charge(&mut Preparation::new(1_000))
                .unwrap()
                .bytes(),
            capacity * 4
        );
        let warm = NodeWarmData {
            runtime_artifact_state: Some(artifact),
            ..Default::default()
        };
        let mut parent: PersistentPagedVector<_> = [warm].into_iter().collect();
        let mut retained = parent.fork_persistent();
        retained[0] = NodeWarmData::default();
        drop(parent);
        let mut work = Preparation::new(1_000);
        assert!(retained.retained_heap_charge(&mut work).unwrap().bytes() >= capacity * 4);
        assert!(matches!(
            retained.retained_heap_charge(&mut Preparation::new(work.visits() - 1)),
            Err(Denial::WorkExhausted { .. })
        ));
    }
}
