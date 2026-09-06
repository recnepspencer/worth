use super::*;
use std::num::NonZeroUsize;
use worth_relational::facade::branch::RelationalBranchBasisAdmissionIdentity;
use worth_runtime_bridge::facade::BridgeCorrespondenceAdmissionIdentity;
use worth_signal::facade::branch::SignalBranchBasisAdmissionIdentity;

/// Normalization is injective in BOTH directions. Learning a new production
/// identity is permitted only for an identity the pure transition already issued.
struct IdentityMap<T>(BTreeMap<usize, T>);
impl<T> Default for IdentityMap<T> {
    fn default() -> Self {
        Self(BTreeMap::new())
    }
}
impl<T: Clone + Eq + std::fmt::Debug> IdentityMap<T> {
    fn check(&mut self, expected: usize, observed: &T) {
        if let Some(known) = self.0.get(&expected) {
            assert_eq!(observed, known);
        } else {
            assert!(
                !self.0.values().any(|known| known == observed),
                "distinct semantic identities cannot alias: {observed:?}"
            );
            self.0.insert(expected, observed.clone());
        }
    }
}
#[derive(Default)]
pub(super) struct ProductObserver {
    commits: IdentityMap<CompositeCommitIdentity>,
    incarnations: IdentityMap<ProductBranchIncarnation>,
    relational: IdentityMap<RelationalBranchBasisAdmissionIdentity>,
    signal: IdentityMap<SignalBranchBasisAdmissionIdentity>,
    correspondence: IdentityMap<BridgeCorrespondenceAdmissionIdentity>,
    relational_keys: BTreeMap<usize, RuntimeWorldRetentionKey>,
    signal_keys: BTreeMap<usize, RuntimeWorldRetentionKey>,
}
impl ProductObserver {
    pub fn commit(&self, identity: usize) -> CompositeCommitIdentity {
        self.commits.0[&identity].clone()
    }
    pub fn check(
        &mut self,
        court: &CompositeSupplyChainCourt,
        model: &ProductModel,
        heads: &BTreeMap<String, ProductBranchObservation>,
        held: &BTreeMap<usize, ProductBranchObservation>,
    ) {
        assert_eq!(heads.len(), model.branches.len());
        assert_eq!(held.len(), model.observations.len());
        let inspection = court.world.inspection_port();
        assert_eq!(
            inspection.history_snapshot().unwrap().installed_commits(),
            model.retained.len()
        );
        assert_eq!(
            inspection.recovery_snapshot().unwrap().reserved(),
            model.attempts.len()
        );
        let page = inspection
            .recovery_page(None, NonZeroUsize::new(128).unwrap())
            .unwrap();
        assert_eq!(
            page.rows().len(),
            model.partials.len() + model.attempts.len()
        );
        for (name, head) in heads {
            let expected = &model.branches[name];
            let fresh = court.observe(head);
            self.incarnations
                .check(expected.incarnation, &fresh.lifecycle_incarnation());
            assert_eq!(fresh.reference_generation().get(), expected.generation);
            self.check_head(court, model, expected.head, &fresh);
            drop(fresh);
            self.incarnations
                .check(expected.incarnation, &head.lifecycle_incarnation());
            assert_eq!(head.reference_generation().get(), expected.generation);
            self.check_head(court, model, expected.head, head);
        }
        for (tag, observation) in held {
            self.check_head(court, model, model.observations[tag], observation);
        }
        // Inspection itself adds no observation or pin. All traversals above
        // have dropped before measuring the persistent semantic dependencies.
        for (relational, keys) in [(true, &self.relational_keys), (false, &self.signal_keys)] {
            for (basis, key) in keys {
                let retained = inspection.inspect_retention(key).unwrap();
                let expected = model.dependencies(relational, *basis);
                for (index, class) in [
                    ComponentBasisDependencyClass::ProductBranchHead,
                    ComponentBasisDependencyClass::RetainedCompositeHistory,
                    ComponentBasisDependencyClass::AdmittedObservation,
                    ComponentBasisDependencyClass::ActivePublicationAttempt,
                    ComponentBasisDependencyClass::ProductUnpublishedOwnerEffects,
                ]
                .into_iter()
                .enumerate()
                {
                    let actual = retained
                        .as_ref()
                        .map_or(0, |entry| entry.dependencies().get(class));
                    assert_eq!(
                        actual, expected[index],
                        "semantic dependency {basis}/{relational}/{class:?}"
                    );
                }
            }
        }
    }

    fn check_head(
        &mut self,
        court: &CompositeSupplyChainCourt,
        model: &ProductModel,
        expected: usize,
        head: &ProductBranchObservation,
    ) {
        self.commits.check(expected, head.selected_commit());
        let occurrence = &model.commits[&expected];
        assert_eq!(
            court.records.read(head.basis().relational_basis()),
            occurrence.cargo.records
        );
        self.check_history(court, model, expected, head.selected_commit());
    }
    pub fn check_unpublished(
        &mut self,
        court: &CompositeSupplyChainCourt,
        model: &ProductModel,
        expected: usize,
        effects: &ProductUnpublishedOwnerEffects,
    ) {
        let occurrence = &model.commits[&expected];
        let basis = effects.successor_basis().unwrap();
        assert_eq!(
            court.records.read(basis.relational_basis()),
            occurrence.cargo.records
        );
        self.relational.check(
            occurrence.relational,
            basis.relational_basis().admission_identity(),
        );
        self.signal
            .check(occurrence.signal, basis.signal_basis().admission_identity());
        self.check_history(
            court,
            model,
            expected,
            effects
                .successor_commit()
                .expect("healthy intermediate retention saves its unpublished successor"),
        );
    }
    fn check_history(
        &mut self,
        court: &CompositeSupplyChainCourt,
        model: &ProductModel,
        expected: usize,
        selected: &CompositeCommitIdentity,
    ) {
        let trace = court
            .world
            .inspection_port()
            .trace_ancestry(selected.clone(), NonZeroUsize::new(128).unwrap())
            .unwrap();
        let ancestry = model.ancestry(expected);
        assert_eq!(trace.visited_count(), ancestry.len());
        assert!(trace.is_complete());
        for (commit, expected) in trace.commits().zip(ancestry) {
            let occurrence = &model.commits[&expected];
            self.relational_keys
                .entry(occurrence.relational)
                .or_insert_with(|| RuntimeWorldRetentionKey::relational(commit.basis()));
            self.signal_keys
                .entry(occurrence.signal)
                .or_insert_with(|| RuntimeWorldRetentionKey::signal(commit.basis()));
            self.commits.check(expected, commit.identity());
            match (commit.parent(), occurrence.parent) {
                (CompositeCommitParent::Root, None) => {}
                (CompositeCommitParent::Ordinary(parent), Some(id)) => {
                    self.commits.check(id, parent.commit())
                }
                other => panic!("independent parentage mismatch: {other:?}"),
            }
            self.relational.check(
                occurrence.relational,
                commit.basis().relational_basis().admission_identity(),
            );
            self.signal.check(
                occurrence.signal,
                commit.basis().signal_basis().admission_identity(),
            );
            self.correspondence.check(
                0,
                commit.basis().correspondence_basis().admission_identity(),
            );
        }
    }
}
