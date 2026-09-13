use super::{ApplicationSchemaContributionAuthoring, ApplicationSchemaContributionDenial};
use crate::application_schema::{
    validate_portable_application_schema_freshly, ApplicationSchemaContributionProvenance,
    ApplicationSchemaDeclarationBuilder, ApplicationSchemaDeclarationDenial,
    WorthQueryPortableApplicationSchemaRecord,
};

crate::worth_query_application! {
    ContributionSchema {
        owner: "worth.query.contribution.tests",
        version: (1, 0),
        contributions: [TopologyContribution, MaterialContribution],
    }
}

crate::worth_query_entity!(TopologyNode for ContributionSchema);
crate::worth_query_entity!(MaterialNode for ContributionSchema);

crate::worth_query_application_contribution! {
    contribution TopologyContribution in ContributionSchema {
        identity: "worth.query.tests.topology-contribution.v1",
        members: |schema| {
            schema.entity(TopologyNode::reference())
        }
    }
}

crate::worth_query_application_contribution! {
    contribution RenamedTopologyContribution in ContributionSchema {
        identity: "worth.query.tests.renamed-topology-contribution.v1",
        members: |schema| { schema.entity(TopologyNode::reference()) }
    }
}

crate::worth_query_application_contribution! {
    contribution SwappedTopologyContribution in ContributionSchema {
        identity: "worth.query.tests.material-contribution.v1",
        members: |schema| { schema.entity(TopologyNode::reference()) }
    }
}

crate::worth_query_application_contribution! {
    contribution SwappedMaterialContribution in ContributionSchema {
        identity: "worth.query.tests.topology-contribution.v1",
        members: |schema| { schema.entity(MaterialNode::reference()) }
    }
}

crate::worth_query_application_contribution! {
    contribution MaterialContribution in ContributionSchema {
        identity: "worth.query.tests.material-contribution.v1",
        members: |schema| {
            schema.entity(MaterialNode::reference())
        }
    }
}

crate::worth_query_application_contribution! {
    contribution DuplicateTopologyIdentity in ContributionSchema {
        identity: "worth.query.tests.topology-contribution.v1",
        members: |schema| {
            schema.entity(MaterialNode::reference())
        }
    }
}

crate::worth_query_application_contribution! {
    contribution InvalidIdentityContribution in ContributionSchema {
        identity: " worth.query.tests.invalid.v1",
        members: |schema| { schema }
    }
}

#[test]
fn macro_references_retain_generic_schema_affinity() {
    let references = [
        TopologyContribution::reference().identity(),
        RenamedTopologyContribution::reference().identity(),
        SwappedTopologyContribution::reference().identity(),
        SwappedMaterialContribution::reference().identity(),
        MaterialContribution::reference().identity(),
        DuplicateTopologyIdentity::reference().identity(),
        InvalidIdentityContribution::reference().identity(),
    ];

    assert_eq!(
        references[0].as_str(),
        "worth.query.tests.topology-contribution.v1"
    );
}

#[test]
fn separately_named_contributions_compose_into_one_canonical_schema() {
    let declared = ContributionSchema::declaration().unwrap();
    let reordered = ApplicationSchemaDeclarationBuilder::<ContributionSchema>::for_schema()
        .contributions()
        .register::<MaterialContribution>()
        .unwrap()
        .register::<TopologyContribution>()
        .unwrap()
        .build()
        .unwrap();

    assert_eq!(declared.identity(), reordered.identity());
    assert_eq!(declared.erased().members(), reordered.erased().members());
    assert_eq!(declared.erased().members().len(), 2);
    assert_eq!(declared.contributions().len(), 2);
}

#[test]
fn contribution_identity_and_member_ownership_are_schema_identity() {
    let original = ContributionSchema::declaration().unwrap();
    let renamed = ApplicationSchemaDeclarationBuilder::<ContributionSchema>::for_schema()
        .contributions()
        .register::<RenamedTopologyContribution>()
        .unwrap()
        .register::<MaterialContribution>()
        .unwrap()
        .build()
        .unwrap();
    let reassigned = ApplicationSchemaDeclarationBuilder::<ContributionSchema>::for_schema()
        .contributions()
        .register::<SwappedTopologyContribution>()
        .unwrap()
        .register::<SwappedMaterialContribution>()
        .unwrap()
        .build()
        .unwrap();

    assert_eq!(original.erased().members(), renamed.erased().members());
    assert_eq!(original.erased().members(), reassigned.erased().members());
    assert_ne!(original.identity(), renamed.identity());
    assert_ne!(original.identity(), reassigned.identity());
}

#[test]
fn portable_readmission_preserves_exact_contribution_closure() {
    let declaration = ContributionSchema::declaration().unwrap().into_erased();
    let record = WorthQueryPortableApplicationSchemaRecord::project(&declaration);
    let readmitted = validate_portable_application_schema_freshly(record).unwrap();

    assert_eq!(readmitted.identity(), declaration.identity());
    assert_eq!(readmitted.contributions(), declaration.contributions());
}

#[test]
fn portable_readmission_rejects_hostile_contribution_closures() {
    assert_portable_denial(
        vec![
            ApplicationSchemaContributionProvenance::from_untrusted_parts(
                "invalid identity".to_owned(),
                vec![0, 1],
            ),
        ],
        ApplicationSchemaDeclarationDenial::InvalidContributionIdentity,
    );
    assert_portable_denial(
        vec![contribution("b", vec![0]), contribution("a", vec![1])],
        ApplicationSchemaDeclarationDenial::InvalidContributionOrdering,
    );
    assert_portable_denial(
        vec![contribution("a", vec![0]), contribution("a", vec![1])],
        ApplicationSchemaDeclarationDenial::DuplicateContributionIdentity,
    );
    assert_portable_denial(
        vec![contribution("a", Vec::new())],
        ApplicationSchemaDeclarationDenial::EmptyContributionClosure,
    );
    assert_portable_denial(
        vec![contribution("a", vec![1, 0])],
        ApplicationSchemaDeclarationDenial::InvalidContributionOrdering,
    );
    assert_portable_denial(
        vec![contribution("a", vec![0]), contribution("b", vec![0])],
        ApplicationSchemaDeclarationDenial::OverlappingContributionClosure,
    );
    assert_portable_denial(
        vec![contribution("a", vec![0])],
        ApplicationSchemaDeclarationDenial::IncompleteContributionClosure,
    );
    assert_portable_denial(
        vec![contribution("a", vec![0, 2])],
        ApplicationSchemaDeclarationDenial::InvalidContributionMemberOrdinal,
    );
}

#[test]
fn duplicate_contribution_identity_is_rejected_before_duplicate_membership() {
    let denial = ApplicationSchemaDeclarationBuilder::<ContributionSchema>::for_schema()
        .contributions()
        .register::<TopologyContribution>()
        .unwrap()
        .register::<DuplicateTopologyIdentity>()
        .unwrap_err();

    assert_eq!(
        denial,
        ApplicationSchemaContributionDenial::DuplicateIdentity
    );
}

#[test]
fn contribution_identity_must_be_canonical_portable_text() {
    let denial = ApplicationSchemaDeclarationBuilder::<ContributionSchema>::for_schema()
        .contributions()
        .register::<InvalidIdentityContribution>()
        .unwrap_err();

    assert_eq!(denial, ApplicationSchemaContributionDenial::InvalidIdentity);
}

fn contribution(identity: &str, ordinals: Vec<u32>) -> ApplicationSchemaContributionProvenance {
    ApplicationSchemaContributionProvenance::from_untrusted_parts(identity.to_owned(), ordinals)
}

fn assert_portable_denial(
    contributions: Vec<ApplicationSchemaContributionProvenance>,
    expected: ApplicationSchemaDeclarationDenial,
) {
    let declaration = ContributionSchema::declaration().unwrap().into_erased();
    let mut parts = WorthQueryPortableApplicationSchemaRecord::project(&declaration).into_parts();
    parts.contributions = contributions;
    let denial = validate_portable_application_schema_freshly(
        WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(parts),
    )
    .unwrap_err();
    assert_eq!(denial, expected);
}
