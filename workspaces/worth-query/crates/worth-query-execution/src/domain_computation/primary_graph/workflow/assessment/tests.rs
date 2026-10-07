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

#[test]
fn performed_entity_kind_dependency_uses_a_distinct_durable_shape() {
    use crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact as Fact;

    let primary = installed_layout();
    let layout = primary.workflow();
    let dependent = entity(7);
    let kind = worth_relational::facade::identity::KindId::new(19);
    let make_meaning = |fact| WorkflowAssessmentEvidenceMeaning {
        identity: String::new(),
        producer: String::new(),
        family: String::new(),
        query: String::new(),
        parameter_type: String::new(),
        result_type: String::new(),
        binding: String::new(),
        subject: dependent,
        proposal_identity: String::new(),
        coverage_identity: String::new(),
        source_identity: String::new(),
        passing: true,
        publication_identity: String::new(),
        output_content_identity: String::new(),
        program_revision: String::new(),
        retained_bytes: 0,
        currentness_facts: std::sync::Arc::from([fact]),
    };
    let evidence = CreatedEntityRef {
        partition_id: worth_relational::facade::identity::PartitionId::main(),
        kind_id: layout.assessment_evidence.entity_kind,
        client_key: ClientKey::raw("evidence"),
    };
    let projected = |fact| {
        let mut effects = Vec::new();
        super::dependency::visit_dependency_facts(
            layout,
            &evidence,
            &make_meaning(fact),
            &mut |effect| {
                effects.push(effect);
                Ok::<(), ()>(())
            },
        )
        .unwrap();
        let WorthQueryApplicationRealizedEffect::CreateEntity { fields, .. } = effects.remove(0)
        else {
            panic!("dependency projection must create its descriptor");
        };
        fields
    };
    let legacy = projected(Fact::SourceEntity {
        entity_id: dependent,
    });
    assert_eq!(
        legacy.get(&layout.evidence_dependency.fact_kind),
        Some(&text("entity"))
    );
    assert!(!legacy.contains_key(&layout.evidence_dependency.native_entity_kind));

    let exact = projected(Fact::Entity {
        entity_id: dependent,
        kind,
    });
    assert_eq!(
        exact.get(&layout.evidence_dependency.fact_kind),
        Some(&text("entity-kind"))
    );
    assert_eq!(
        exact.get(&layout.evidence_dependency.native_entity_kind),
        Some(&AspectValue::UInt64(u64::from(kind.as_u32())))
    );
}
