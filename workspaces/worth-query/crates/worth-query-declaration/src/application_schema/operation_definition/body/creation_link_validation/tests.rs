//! Cold self edges need equal endpoints and a relation that admits self edges.
use super::validate_links;
use crate::application_schema::{
    ApplicationOperationBodyCreation as Creation, ApplicationOperationBodyCreationLink as Link,
    ApplicationOperationBodyCreationLinkDirection as Direction,
    ApplicationOperationBodyCreationLinkPosture as Posture,
    ApplicationOperationBodyCreationLinkSource as Source, ApplicationOperationBodyProgram as Body,
    ApplicationRelationIntegrity as Integrity, ApplicationSchemaDeclarationDenial as Denial,
    ApplicationSchemaMember as Member,
};
fn self_edge(from: &str) -> Body {
    let link = Link {
        relation: "Watches".into(),
        from: from.into(),
        to: "Project".into(),
        key: "watch-self".into(),
        direction: Direction::SelfEdge,
        posture: Posture::Append,
        source: Source::Scope,
    };
    let creation = Creation::from_untrusted("Project".into(), "project".into(), vec![])
        .with_untrusted_links(vec![link]);
    Body::from_untrusted_in_scope("Workspace".into(), vec![], Some(creation), vec![])
}
fn members(from: &str, integrity: Integrity) -> Vec<Member> {
    let link = self_edge(from).creation().unwrap().links()[0].program_target();
    vec![
        Member::Relation {
            relation: "Watches".into(),
            from: from.into(),
            to: "Project".into(),
            integrity: integrity.into(),
        },
        Member::OperationProgram {
            operation: "NewProject".into(),
            target: link,
        },
    ]
}
#[test]
fn cold_self_edge_requires_a_self_edge_relation_and_equal_endpoints() {
    let admitting = Integrity::same_context_unbounded_retain_dangling();
    let refusing = Integrity::same_context_no_self_edges_unbounded_retain_dangling();
    let refused = Err(Denial::program_body_dependency("NewProject"));
    assert_eq!(
        validate_links(
            "NewProject",
            &self_edge("Project"),
            &members("Project", admitting)
        ),
        Ok(())
    );
    // Fully declared, but the relation itself forbids self edges.
    assert_eq!(
        validate_links(
            "NewProject",
            &self_edge("Project"),
            &members("Project", refusing)
        ),
        refused
    );
    // A forged self edge naming a different source endpoint is not a self edge.
    assert_eq!(
        validate_links(
            "NewProject",
            &self_edge("Workspace"),
            &members("Workspace", admitting)
        ),
        refused
    );
}
