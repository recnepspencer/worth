use super::*;
use crate::domain_computation::primary_graph::tests::fixture::installed_layout;

fn created(
    kind: worth_relational::facade::identity::KindId,
    fields: BTreeMap<worth_foundational::facade::AspectFieldLocator, AspectValue>,
) -> WorthQueryApplicationRealizedEffect {
    WorthQueryApplicationRealizedEffect::CreateEntity {
        kind,
        key: "charged".to_owned(),
        fields,
        partition: WorthQueryApplicationCreationPartition::Issued,
    }
}

fn entity(slot: u64) -> EntityId {
    EntityId::new(
        worth_relational::facade::identity::PartitionId::main(),
        slot,
        1,
    )
}

#[test]
fn evidence_and_each_dependency_charge_their_field_widths_and_nothing_else_does() {
    let primary = installed_layout();
    let layout = primary.workflow();
    let evidence = BTreeMap::from([
        (layout.assessment_evidence.identity.clone(), text("abcd")),
        (
            layout.assessment_evidence.retained_bytes.clone(),
            AspectValue::UInt64(u64::MAX),
        ),
    ]);
    let dependency = BTreeMap::from([
        (layout.evidence_dependency.fact_kind.clone(), text("field")),
        (
            layout.evidence_dependency.native_revision.clone(),
            AspectValue::UInt64(3),
        ),
        (
            layout.evidence_dependency.field_presence.clone(),
            AspectValue::Bool(true),
        ),
    ]);

    let charge = |effect| workflow_evidence_retained_bytes(layout, &effect);
    assert_eq!(
        charge(created(
            layout.assessment_evidence.entity_kind,
            evidence.clone()
        )),
        5 + 9
    );
    assert_eq!(
        charge(created(layout.evidence_dependency.entity_kind, dependency)),
        6 + 9 + 2
    );
    assert_eq!(charge(created(layout.transition.entity_kind, evidence)), 0);
    assert_eq!(
        charge(WorthQueryApplicationRealizedEffect::CreateRelation {
            kind: layout.evidence_dependency_relation,
            key: "evidence-dependency".to_owned(),
            from: EntityReference::Existing(entity(1)),
            to: EntityReference::Existing(entity(2)),
        }),
        0,
    );
}
