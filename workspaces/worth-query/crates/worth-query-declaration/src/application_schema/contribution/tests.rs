use crate::application_schema::{
    fragment::{self, *},
    *,
};
#[derive(Debug)]
struct ContributionSchema;
impl Fragment for ContributionSchema {
    const NAME: &'static str = "ContributionSchema";
    type Bindings = NoBindings;
    type Members = (TopologyContribution, MaterialContribution);
}
impl ApplicationSchema for ContributionSchema {
    const OWNER: &'static str = "worth.query.contribution.tests";
    const NAME: &'static str = <Self as Fragment>::NAME;
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
struct TopologyNode;
impl Entity for TopologyNode {
    const NAME: &'static str = "TopologyNode";
    type Aspects = ();
    type Relations = ();
}
struct MaterialNode;
impl Entity for MaterialNode {
    const NAME: &'static str = "MaterialNode";
    type Aspects = ();
    type Relations = ();
}
struct TopologyContribution;
impl ApplicationSchemaContribution<ContributionSchema> for TopologyContribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("worth.query.tests.topology-contribution.v1");
    type Members = (TopologyNode,);
}
struct RenamedTopologyContribution;
impl ApplicationSchemaContribution<ContributionSchema> for RenamedTopologyContribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new(
            "worth.query.tests.renamed-topology-contribution.v1",
        );
    type Members = (TopologyNode,);
}
struct SwappedTopologyContribution;
impl ApplicationSchemaContribution<ContributionSchema> for SwappedTopologyContribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("worth.query.tests.material-contribution.v1");
    type Members = (TopologyNode,);
}
struct SwappedMaterialContribution;
impl ApplicationSchemaContribution<ContributionSchema> for SwappedMaterialContribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("worth.query.tests.topology-contribution.v1");
    type Members = (MaterialNode,);
}
struct MaterialContribution;
impl ApplicationSchemaContribution<ContributionSchema> for MaterialContribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("worth.query.tests.material-contribution.v1");
    type Members = (MaterialNode,);
}
struct DuplicateTopologyIdentity;
impl ApplicationSchemaContribution<ContributionSchema> for DuplicateTopologyIdentity {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("worth.query.tests.topology-contribution.v1");
    type Members = (TopologyNode,);
}
struct InvalidIdentityContribution;
impl ApplicationSchemaContribution<ContributionSchema> for InvalidIdentityContribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new(" worth.query.tests.invalid.v1");
    type Members = ();
}
struct Reordered;
impl Fragment for Reordered {
    const NAME: &'static str = <ContributionSchema as Fragment>::NAME;
    type Bindings = NoBindings;
    type Members = (MaterialContribution, TopologyContribution);
}
struct Renamed;
impl Fragment for Renamed {
    const NAME: &'static str = <ContributionSchema as Fragment>::NAME;
    type Bindings = NoBindings;
    type Members = (RenamedTopologyContribution, MaterialContribution);
}
struct Reassigned;
impl Fragment for Reassigned {
    const NAME: &'static str = <ContributionSchema as Fragment>::NAME;
    type Bindings = NoBindings;
    type Members = (SwappedTopologyContribution, SwappedMaterialContribution);
}
struct Duplicate;
impl Fragment for Duplicate {
    const NAME: &'static str = <ContributionSchema as Fragment>::NAME;
    type Bindings = NoBindings;
    type Members = (TopologyContribution, DuplicateTopologyIdentity);
}
struct Invalid;
impl Fragment for Invalid {
    const NAME: &'static str = <ContributionSchema as Fragment>::NAME;
    type Bindings = NoBindings;
    type Members = (InvalidIdentityContribution,);
}
fn reference<C: ApplicationSchemaContribution<ContributionSchema>>(
    _: C,
) -> ApplicationSchemaContributionRef<ContributionSchema, C> {
    ApplicationSchemaContributionRef::from_contribution()
}

#[test]
fn contribution_references_retain_schema_affinity() {
    let references = [
        reference(TopologyContribution).identity(),
        reference(RenamedTopologyContribution).identity(),
        reference(SwappedTopologyContribution).identity(),
        reference(SwappedMaterialContribution).identity(),
        reference(MaterialContribution).identity(),
        reference(DuplicateTopologyIdentity).identity(),
        reference(InvalidIdentityContribution).identity(),
    ];

    assert_eq!(
        references[0].as_str(),
        "worth.query.tests.topology-contribution.v1"
    );
}

#[test]
fn separately_named_contributions_compose_into_one_canonical_schema() {
    let declared = ContributionSchema::declaration().unwrap();
    let reordered =
        ApplicationSchemaDeclarationBuilder::<ContributionSchema>::from_fragment(Reordered)
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
    let renamed = ApplicationSchemaDeclarationBuilder::<ContributionSchema>::from_fragment(Renamed)
        .unwrap()
        .build()
        .unwrap();
    let reassigned =
        ApplicationSchemaDeclarationBuilder::<ContributionSchema>::from_fragment(Reassigned)
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
    let denial =
        ApplicationSchemaDeclarationBuilder::<ContributionSchema>::from_fragment(Duplicate)
            .unwrap_err();

    assert_eq!(
        denial,
        ApplicationSchemaDeclarationDenial::DuplicateContributionIdentity
    );
}

#[test]
fn contribution_identity_must_be_canonical_portable_text() {
    let denial = ApplicationSchemaDeclarationBuilder::<ContributionSchema>::from_fragment(Invalid)
        .unwrap_err();

    assert_eq!(
        denial,
        ApplicationSchemaDeclarationDenial::InvalidContributionIdentity
    );
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
