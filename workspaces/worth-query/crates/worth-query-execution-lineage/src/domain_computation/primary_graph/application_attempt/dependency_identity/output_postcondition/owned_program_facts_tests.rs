//! Storage construction and carry meaning; no graph-currentness authority is minted here.
use super::OutputCurrentnessFacts;
use crate::domain_computation::primary_graph::{
    application_attempt::{
        retained_decision_facts::{StorageControl, StoreDenial},
        WorthQueryApplicationObservedFact as Fact,
    },
    application_checkpoint::facts::{self, FactPayload},
};
use std::sync::Arc;
use worth_execution::ExecutionAllocationPolicy;
use worth_relational::facade::identity::{EntityId, PartitionId};

#[test]
fn owned_program_facts_prepare_native_rebase_without_relabeling_opaque_payloads() {
    let policy = ExecutionAllocationPolicy::SystemAllocation;
    let control = StorageControl::new(policy, None);
    let expected = vec![Fact::SourceEntity {
        entity_id: EntityId::new(PartitionId::main(), 7, 1),
    }];
    let owned = OutputCurrentnessFacts::admit_owned_program_facts(expected.clone(), control)
        .expect("Query-owned program facts complete native construction");
    assert!(owned.facts().iter().eq(expected.iter()));
    assert!(
        owned.rebase_carry().is_none(),
        "no previous postcondition proof"
    );
    assert!(
        owned.moved_by_own_effect().is_empty(),
        "no normalized owner-call reads"
    );
    let signing_basis = owned.facts().clone();
    assert!(signing_basis.same_backing(owned.facts()));
    let separate =
        OutputCurrentnessFacts::admit_owned_program_facts(expected.clone(), control).unwrap();
    assert!(
        !signing_basis.same_backing(separate.facts()),
        "equal facts do not share custody"
    );

    let selected = im::ordset![0];
    let mut patch = owned
        .facts()
        .prepare_native_patch(&selected, control)
        .unwrap();
    patch.replace(0, &expected[0], control).unwrap();
    assert!(patch.seal(control).unwrap().iter().eq(expected.iter()));

    let original: FactPayload = Arc::<[Fact]>::from(expected.clone()).into();
    let decoded = facts::decode(&facts::encode(&expected).unwrap(), None, policy).unwrap();
    for opaque in [original, decoded] {
        assert!(opaque.iter().eq(expected.iter()));
        assert!(
            matches!(
                opaque.prepare_native_patch(&selected, control),
                Err(StoreDenial::ConflictingBody)
            ),
            "opaque fact equality cannot confer native patch ownership"
        );
    }
}
