//! Actual ordered Source membership/inversion under genuine Native observation.
use super::*;
mod court;
mod fixture;
mod original_registration;
mod pinned;
pub(super) mod probe;
mod refusal;
pub(super) mod registry;

#[derive(Clone, Default)]
pub(super) struct Control {
    // Authored roster keyed only by genuinely issued event/row coordinates.
    pub expected: Arc<Mutex<BTreeMap<EntityId, Vec<EntityId>>>>,
    pub checked: Arc<AtomicUsize>,
}
pub(super) struct Reader {
    pub base: owner::Reader,
    pub path: DerivedRelationHandle,
    pub control: Control,
}
pub(super) struct Scope {
    managed: Arc<PreparedManagedIndexSourceRead>,
    reads: Vec<(bool, EntityId, PreparedDerivedRelationRead)>,
}
impl CustomInvariantRule for Reader {
    type Scope = Scope;
    fn descriptor(&self) -> CustomInvariantDescriptor {
        let mut descriptor = self.base.descriptor();
        // This reader judges both scalar schema facts and event relations.
        // CommitBoundary deletion selects relation groups, not schema admission.
        descriptor.operational.groups = InvariantGroupSet::of(InvariantGroup::SchemaCompliance)
            .union(InvariantGroupSet::of(InvariantGroup::RelationIntegrity));
        descriptor.operational.access.read_relation_kinds = vec![KindId(2), KindId(3), KindId(5)];
        descriptor.operational.access.affected_relation_kinds =
            vec![KindId(2), KindId(3), KindId(5)];
        descriptor
    }
    fn prepare_scope(
        &self,
        planner: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<Scope, CustomInvariantPreparationError> {
        let managed = self.base.prepare_scope(planner)?;
        let expected = self.control.expected.lock().unwrap();
        let mut reads = Vec::new();
        if expected.contains_key(&managed.scope()) {
            let view = planner.relations();
            let forward = view
                .prepare_derived_source(
                    &self.path,
                    managed.scope(),
                    DerivedRelationReadShape::Adjacency { reverse: false },
                )
                .unwrap();
            reads.push((false, managed.scope(), forward));
            let rows = expected
                .values()
                .flatten()
                .copied()
                .collect::<BTreeSet<_>>();
            for row in rows {
                let inverse = view
                    .prepare_derived_source(
                        &self.path,
                        row,
                        DerivedRelationReadShape::Adjacency { reverse: true },
                    )
                    .unwrap();
                reads.push((true, row, inverse));
            }
        }
        Ok(Scope { managed, reads })
    }
    fn evaluate(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        scope: &Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        let result = self.base.evaluate(context, &scope.managed)?;
        let expected = self.control.expected.lock().unwrap();
        let view = context.relations();
        for (reverse, anchor, proof) in &scope.reads {
            let data = view
                .read_prepared_derived_source(
                    proof,
                    DerivedRelationReadShape::Adjacency { reverse: *reverse },
                )
                .unwrap();
            let actual = data
                .endpoints()
                .iter()
                .map(|key| key.entity().unwrap())
                .collect::<Vec<_>>();
            let wanted = if *reverse {
                expected
                    .iter()
                    .filter_map(|(at, rows)| rows.contains(anchor).then_some(*at))
                    .collect()
            } else {
                expected[anchor].clone()
            };
            assert_eq!(actual, wanted, "real ordered membership, reverse={reverse}");
            assert!(
                !data.adjacency_dependencies().is_empty(),
                "empty and terminal reads are retained"
            );
            self.control.checked.fetch_add(1, Ordering::SeqCst);
        }
        Ok(result)
    }
}
