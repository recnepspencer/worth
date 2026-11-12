//! Actual typed record omission twins; these selected field rosters grant no authority.
use crate::durability::canonical_commit_artifact::borrowed_payload::{
    omit_nil_record_field, replace_record,
};
use crate::schema::data::*;
use crate::transactions::data::RelationalInvariantCheckMeaning;
use crate::validation::data::*;
use serde::Serialize;

pub(super) fn schema() -> (Vec<u8>, Vec<u8>) {
    let detail = SchemaDiffDetail::AddedField {
        field: worth_foundational::FieldKey::new("optional-field").unwrap(),
        required: false,
        default_expression: None,
    };
    // The enum's genuine single-key map contains the actual three-field map.
    let map_at = 1 + rmp_serde::to_vec("AddedField").unwrap().len();
    let changed = omit_nil_record_field(&detail, "default_expression", map_at);
    assert_eq!(
        rmp_serde::from_slice::<SchemaDiffDetail>(&changed).unwrap(),
        detail
    );
    let atom = SchemaDiffAtom::new(
        SchemaElementRef::new(
            SchemaElementKind::Field,
            SchemaId("schema".into()),
            SchemaVersionId(1),
            None,
            "field",
        ),
        vec![],
        SchemaPublicationImpact::None,
        SchemaSubscriberImpact::None,
        HistoricalInterpretationSensitivity::NotSensitive,
        detail.clone(),
    );
    #[derive(Serialize)]
    struct Transition {
        diff_atoms: Vec<SchemaDiffAtom>,
    }
    #[derive(Serialize)]
    struct Selected {
        schema_transition: Transition,
    }
    let before = rmp_serde::to_vec_named(&Selected {
        schema_transition: Transition {
            diff_atoms: vec![atom],
        },
    })
    .unwrap();
    let after = replace_record(&before, &detail, &changed);
    (before, after)
}

pub(super) fn validation() -> (Vec<u8>, Vec<u8>) {
    use crate::derived_relation::{
        DerivedRelationIdentity, DerivedRelationLaw, DerivedRelationLawViolation,
    };
    use crate::graph_member::NativeMemberKey;
    use crate::identity::data::{EntityId, PartitionId};
    let violation = DerivedRelationLawViolation {
        law: DerivedRelationLaw::SourceMinimum,
        member: NativeMemberKey::Entity(EntityId::new(PartitionId(1), 2, 3)),
        counterpart: None,
        count: 0,
        limit: 1,
        physical_entities: vec![],
        physical_relations: vec![],
    };
    let check = RelationalInvariantCheckMeaning {
        execution_point: InvariantExecutionPoint::CommitBoundary,
        failure_effect: InvariantFailureEffect::BlockCommit,
        rule: InvariantReportedRule::Native(InvariantRule::MaxMergedIntents(1)),
        witness: "typed-omission".into(),
        groups: InvariantGroupSet::empty(),
        declared_cost: InvariantCostClass::Touched,
        custom_provenance: None,
        verdict: InvariantVerdict::Violation(InvariantViolation {
            class: InvariantClass::CommitBoundary,
            code: crate::diagnostics::data::DiagnosticCode::InvariantViolation,
            detail: "descriptive typed value".into(),
            fields: InvariantViolationFields::DerivedRelationGraph {
                definition: DerivedRelationIdentity {
                    schema: SchemaId("schema".into()),
                    version: SchemaVersionId(1),
                    name: "recipe".into(),
                },
                violation: violation.clone(),
            },
        }),
    };
    let changed = omit_nil_record_field(&check, "custom_provenance", 0);
    let without_counterpart = omit_nil_record_field(&violation, "counterpart", 0);
    assert_eq!(
        rmp_serde::from_slice::<DerivedRelationLawViolation>(&without_counterpart).unwrap(),
        violation
    );
    let changed = replace_record(&changed, &violation, &without_counterpart);
    assert_eq!(
        rmp_serde::from_slice::<RelationalInvariantCheckMeaning>(&changed).unwrap(),
        check
    );
    #[derive(Serialize)]
    struct Execution {
        checks: Vec<RelationalInvariantCheckMeaning>,
        decisions: [(); 0],
    }
    #[derive(Serialize)]
    struct Semantic {
        validation: Vec<Execution>,
    }
    #[derive(Serialize)]
    struct Selected {
        semantic_result: Semantic,
    }
    let before = rmp_serde::to_vec_named(&Selected {
        semantic_result: Semantic {
            validation: vec![Execution {
                checks: vec![check.clone()],
                decisions: [],
            }],
        },
    })
    .unwrap();
    let after = replace_record(&before, &check, &changed);
    (before, after)
}
