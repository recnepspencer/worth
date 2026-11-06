//! The two cases share one actual carrier and one Native component group.
use super::super::*;
pub struct Uses;
pub struct Alpha;
pub struct Omega;
pub struct CaseRoots;
pub struct CaseAdjacency;
pub struct CaseComponents;
impl Uses {
    pub fn reference<S: Scenario>() -> ApplicationProjectedMemberDomainRef<S, Self, Node<S>> {
        ApplicationProjectedMemberDomainRef::new("NodeUses", schema::node::<S>())
    }
}
impl Alpha {
    pub fn reference<S: Scenario>() -> ApplicationProjectedMemberCaseRef<S, Uses, Self> {
        ApplicationProjectedMemberCaseRef::new(Uses::reference(), "Alpha")
    }
}
impl Omega {
    pub fn reference<S: Scenario>() -> ApplicationProjectedMemberCaseRef<S, Uses, Self> {
        ApplicationProjectedMemberCaseRef::new(Uses::reference(), "Omega")
    }
}
impl CaseRoots {
    pub fn reference<S: Scenario>() -> ApplicationDerivedRelationRef<S, Self, Node<S>, Uses> {
        ApplicationDerivedRelationRef::from_member_identifiers(
            "CaseRoots",
            ApplicationGraphMemberSelector::Entity("Node"),
            ApplicationGraphMemberSelector::Projected("NodeUses"),
        )
    }
}
impl CaseAdjacency {
    pub fn reference<S: Scenario>() -> ApplicationDerivedRelationRef<S, Self, Uses, Uses> {
        ApplicationDerivedRelationRef::projected("CaseAdjacency", Uses::reference())
    }
}
impl CaseComponents {
    pub fn reference<S: Scenario>() -> ApplicationComponentSetRef<S, Self, Uses> {
        ApplicationComponentSetRef::projected("CaseComponents", Uses::reference())
    }
}
pub fn declare<S: Scenario>(
    schema: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    if S::MODE != 29 {
        return schema;
    }
    let alpha =
        || ApplicationDerivedRelation::carrier_case(Uses::reference(), Alpha::reference()).unwrap();
    let omega =
        || ApplicationDerivedRelation::carrier_case(Uses::reference(), Omega::reference()).unwrap();
    let schema = schema
        .projected_member_domain(
            ApplicationProjectedMemberDomainDefinition::new(Uses::reference())
                .case(Alpha::reference())
                .unwrap()
                .case(Omega::reference())
                .unwrap(),
        )
        .derived_relation(
            CaseRoots::reference(),
            alpha().union(omega()),
            ApplicationRelationGraphLaws::unbounded(),
        )
        .derived_relation(
            CaseAdjacency::reference(),
            alpha()
                .reversed()
                .then(omega())
                .union(omega().reversed().then(alpha())),
            ApplicationRelationGraphLaws::unbounded(),
        )
        .derived_components(ApplicationDerivedComponentSetDefinition::new(
            CaseComponents::reference(),
            CaseAdjacency::reference(),
        ));
    super::inspection::declare(schema)
}
