//! Native declaration closure must retain both selection observations and unlink authority.
use crate::application_schema::{
    ApplicationOperationBodyAuthoringDenial as BodyDenial,
    ApplicationOperationDecisionReadTarget as Read, ApplicationOperationProgramTarget as Target,
    ApplicationRelationCardinality, ApplicationRelationIntegrity,
    ApplicationSchemaDeclarationBuilder, ApplicationSchemaDeclarationDenial as Denial,
    ApplicationSchemaMember as Member,
};
struct Schema;
crate::worth_query_entity!(Node for Schema);
crate::worth_query_relation!(Current in Schema, Node => Node; integrity = {
    let base = ApplicationRelationIntegrity::same_context_unbounded_retain_dangling();
    ApplicationRelationIntegrity::new(base.endpoints,
        ApplicationRelationCardinality::new(None, Some(1), None, None, None, None), base.deletion)
});
crate::worth_query_relation!(Unbounded in Schema, Node => Node; integrity = same_context_unbounded_retain_dangling);
crate::worth_query_structured_value_binding!(Input for () { identity: "worth.tests.replacement-input.v1" });
crate::worth_query_operation!(Replace for Schema, input Input);
crate::worth_query_operation_reads!(Replace => [Node, Current, Unbounded]);
crate::worth_query_operation_creates!(Replace => [Node]);
crate::worth_query_operation_links!(Replace => [Current, Unbounded]);
crate::worth_query_operation_unlinks!(Replace => [Current, Unbounded]);

fn declaration() -> crate::application_schema::ApplicationSchemaDeclaration<Schema> {
    let body = Replace::reference()
        .body()
        .in_scope(Node::reference())
        .unwrap()
        .create_in_scope(Node::reference(), "new")
        .unwrap()
        .replace_from_scope(Current::reference(), "current")
        .unwrap()
        .finish()
        .unwrap();
    ApplicationSchemaDeclarationBuilder::<Schema>::from_test_members(vec![])
        .entity(Node::reference())
        .relation(Current::reference(), Node::reference(), Node::reference())
        .operation(
            Replace::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish()
                .with_body(body)
                .unwrap(),
        )
        .operation_projection_work_budget(Replace::reference(), 32)
        .build()
        .unwrap()
}

#[test]
fn replacement_derives_read_and_unlink_membership_and_cold_readmission_requires_both() {
    let declared = declaration();
    for missing in ["read", "unlink"] {
        let mut members = declared.erased().members().to_vec();
        let before = members.len();
        members.retain(|member| match member {
            Member::OperationDecisionRead {
                target: Read::Relation { relation, .. },
                ..
            } if missing == "read" => relation != "Current",
            Member::OperationProgram {
                target: Target::Unlink { relation, .. },
                ..
            } if missing == "unlink" => relation != "Current",
            _ => true,
        });
        assert_eq!(before - members.len(), 1);
        assert!(matches!(
            ApplicationSchemaDeclarationBuilder::<Schema>::from_test_members(members)
                .build()
                .unwrap_err(),
            Denial::MissingOperationProgramDependency { .. }
        ));
    }
}

#[test]
fn replacement_rejects_unbounded_retirement_and_conflicting_selection_instructions() {
    let creation = || {
        Replace::reference()
            .body()
            .in_scope(Node::reference())
            .unwrap()
            .create_in_scope(Node::reference(), "new")
            .unwrap()
    };
    assert!(matches!(
        creation().replace_from_scope(Unbounded::reference(), "current"),
        Err(BodyDenial::InvalidCreationLink)
    ));
    assert!(matches!(
        creation()
            .replace_from_scope(Current::reference(), "current")
            .unwrap()
            .link_from_scope(Current::reference(), "second"),
        Err(BodyDenial::InvalidCreationLink)
    ));
    assert!(matches!(
        creation()
            .link_from_scope(Current::reference(), "first")
            .unwrap()
            .replace_from_scope(Current::reference(), "current"),
        Err(BodyDenial::InvalidCreationLink)
    ));
}
