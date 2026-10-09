//! Auto-observed unique child identities use the installed equality index.
use super::*;
use crate::domain_computation::primary_graph::tests::merge_unique_values::MergeWorld;
use worth_foundational::facade::{AspectValue, InternedString};
use worth_relational::facade::history::BranchId;
#[test]
fn delegation_child_identity_records_absence_and_taken_values_once() {
    let world = MergeWorld::new();
    let held = world.create("main", "held-child", 7);
    let unique = world.unique(Some(world.index));
    world.mutate(|runtime| {
        let identity = runtime.branch_identity(&BranchId("main".to_owned())).unwrap();
        let (_, basis) = runtime.observe_branch(&identity).unwrap();
        let snapshot = runtime.snapshots().snapshot_for_observation(&basis.observation()).unwrap();
        for (ordinal, holders) in [(8, Vec::new()), (7, vec![held])] {
            let value = AspectValue::String(InternedString::from(format!("value-{ordinal}")));
            let effect = || WorthQueryApplicationRealizedEffect::CreateEntity {
                kind: world.kind, key: "delegated-child".to_owned(),
                fields: std::collections::BTreeMap::from([(world.label.clone(), value.clone())]),
                partition: super::super::super::effect_program::WorthQueryApplicationCreationPartition::Issued,
            };
            let mut calls = 0;
            let facts = observe_created_values(&[], unique.fields(), &[effect(), effect()], 1, "child", |index, kind, locator, value| {
                calls += 1;
                observe_indexed_entity_selection(runtime, &snapshot, index, kind, locator, value, CHILD_IDENTITY_CANDIDATE_LIMIT)
            }).unwrap();
            assert_eq!(calls, 1, "one distinct created identity has one lookup");
            assert_eq!(facts.len(), 1);
            let WorthQueryApplicationObservedFact::IndexedEntitySelection { candidates, .. } = &facts[0] else { panic!("the exact indexed selection is carried") };
            assert_eq!(candidates, &holders);
            let lower = |facts, effects: Vec<WorthQueryApplicationRealizedEffect>| {
                super::super::super::provider_binding::prepare_provider_attempt(
                    unique.fields(), worth_relational::facade::identity::PartitionId::main(),
                    effects.len(), Vec::new(), facts, Vec::new(), effects, 0, 0, None, None,
                    super::super::super::effect_program::WorthQueryCandidateValidatorWorkAdmission::unreserved_internal(),
                    Default::default(), false, false, &[], None,
                ).map(|_| ()).map_err(|denial| denial.kind())
            };
            assert_eq!(lower(facts.clone(), vec![effect()]),
                if holders.is_empty() { Ok(()) } else { Err(WorthQueryApplicationAttemptDenialKind::UniqueValueTaken) });
            if holders.is_empty() {
                let mut other = effect();
                let WorthQueryApplicationRealizedEffect::CreateEntity { key, .. } = &mut other else { unreachable!() };
                *key = "delegated-other-child".to_owned();
                assert_eq!(lower(facts.clone(), vec![effect(), other]),
                    Err(WorthQueryApplicationAttemptDenialKind::UniqueValueTaken),
                    "one delegation cannot write its unique child identity twice");
            }

            assert!(observe_created_values(&facts, unique.fields(), &[effect()], 1, "child", |_, _, _, _| panic!("already observed identity is not read again")).unwrap().is_empty());
            let stop = observe_created_values(&[], unique.fields(), &[effect()], 0, "child", |_, _, _, _| panic!("budget rejects before the read")).unwrap_err();
            assert_eq!(stop.kind(), WorthQueryApplicationAttemptDenialKind::DecisionFactBudgetExceeded);
        }
        runtime.snapshots().release_snapshot(&snapshot).unwrap();
    });
}
