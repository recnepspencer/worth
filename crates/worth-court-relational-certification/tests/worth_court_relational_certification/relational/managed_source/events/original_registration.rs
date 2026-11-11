//! Receiving compilation preserves genuine registered Source issuers.
use super::*;
use worth_relational::facade::schema::{
    ProjectedMemberDomainDenial, ProjectedMemberDomainIdentity, ProjectedMemberDomainParts,
};

#[test]
fn receiving_compilation_retains_the_original_recipe_and_bound_source_program() {
    let base = registry::base();
    let parts = registry::parts();
    let original = base
        .clone()
        .register_derived_relations(vec![parts.clone()])
        .unwrap();
    let path = original.derived_relation(&parts.identity).unwrap();
    let program = original
        .bind_managed_index_source(&path, vec![], vec![])
        .unwrap();
    let receiving = base
        .clone()
        .register_derived_relations_from_original(vec![parts.clone()], &original)
        .unwrap();
    let retained = receiving.derived_relation(&parts.identity).unwrap();
    assert_eq!(retained, path);
    assert_eq!(
        receiving
            .bind_managed_index_source(&retained, vec![], vec![])
            .unwrap(),
        program
    );
    let independent = base
        .clone()
        .register_derived_relations(vec![parts.clone()])
        .unwrap();
    let foreign = independent.derived_relation(&parts.identity).unwrap();
    assert_ne!(foreign, path);
    assert_ne!(
        independent
            .bind_managed_index_source(&foreign, vec![], vec![])
            .unwrap(),
        program
    );
    let mut changed = parts.clone();
    changed.nodes.push(DerivedRelationNode::Reverse {
        input: changed.nodes.len() - 1,
    });
    assert!(matches!(
        base.register_derived_relations_from_original(vec![changed], &original),
        Err(DerivedRelationDenial::ForeignDefinition)
    ));
    assert!(matches!(
        RelationalSchemaRegistry::new()
            .register_derived_relations_from_original(vec![parts], &original),
        Err(DerivedRelationDenial::InvalidPhysicalSource(_))
    ));
}

#[test]
fn receiving_compilation_retains_the_original_projected_domain_and_refuses_changed_cases() {
    let base = registry::base();
    let parts = ProjectedMemberDomainParts {
        identity: ProjectedMemberDomainIdentity {
            schema: SchemaId("managed-source-court".into()),
            version: SchemaVersionId(1),
            name: "original-domain".into(),
        },
        carrier: KindId(4),
        cases: vec!["first".into(), "second".into()],
    };
    let original = base
        .clone()
        .register_projected_member_domains(vec![parts.clone()])
        .unwrap();
    let issued = original.projected_member_domain(&parts.identity).unwrap();
    let receiving = base
        .clone()
        .register_projected_member_domains_from_original(vec![parts.clone()], &original)
        .unwrap();
    assert_eq!(
        receiving.projected_member_domain(&parts.identity).unwrap(),
        issued
    );
    let independent = base
        .clone()
        .register_projected_member_domains(vec![parts.clone()])
        .unwrap();
    assert_ne!(
        independent
            .projected_member_domain(&parts.identity)
            .unwrap(),
        issued
    );
    let mut changed = parts;
    changed.cases.push("third".into());
    assert!(matches!(
        base.register_projected_member_domains_from_original(vec![changed], &original),
        Err(ProjectedMemberDomainDenial::ForeignDomain)
    ));
}
