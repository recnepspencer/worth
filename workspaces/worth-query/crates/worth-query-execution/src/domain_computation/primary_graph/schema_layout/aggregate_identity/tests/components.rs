//! Real typed schema installation shares one Native aggregate owner with Orbit.
use super::*;

struct ComponentSchema<const REVERSED: bool, const SINGLE: bool>;
struct CouplingA;
struct CouplingB;
struct Shells;
struct OnlyA;
impl<const R: bool, const S: bool> ApplicationEntityMarkerIdentity<ComponentSchema<R, S>> for Node {
    const IDENTIFIER: &'static str = "Node";
}
impl<const R: bool, const S: bool> ApplicationSchema for ComponentSchema<R, S> {
    const OWNER: &'static str = "worth.tests.components";
    const NAME: &'static str = "ComponentLowering";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        let node = ApplicationEntityRef::<Self, Node>::from_schema_identifier("Node");
        let base = ApplicationRelationIntegrity::same_context_unbounded_retain_dangling();
        let a = ApplicationRelationRef::<Self, CouplingA, Node, Node>::from_schema_identifiers(
            "CouplingA",
            "Node",
            "Node",
            base,
        );
        let b = ApplicationRelationRef::<Self, CouplingB, Node, Node>::from_schema_identifiers(
            "CouplingB",
            "Node",
            "Node",
            base,
        );
        let shells = ApplicationComponentSetRef::<Self, Shells, Node>::new("Shells", node);
        let definition = ApplicationComponentSetDefinition::new(shells);
        let definition = if S {
            definition.coupled_by(a)
        } else if R {
            definition.coupled_by(b).coupled_by(a).coupled_by(b)
        } else {
            definition.coupled_by(a).coupled_by(b)
        };
        ApplicationSchemaDeclarationBuilder::<Self>::for_schema()
            .entity(node)
            .relation(a, node, node)
            .relation(b, node, node)
            .relation(
                ApplicationRelationRef::<Self, Next, Node, Node>::from_schema_identifiers(
                    "Next",
                    "Node",
                    "Node",
                    base.orbit(),
                ),
                node,
                node,
            )
            .components(definition)
            .components(
                ApplicationComponentSetDefinition::new(ApplicationComponentSetRef::<
                    Self,
                    OnlyA,
                    Node,
                >::new("OnlyA", node))
                .coupled_by(a),
            )
            .build()
    }
}

fn contracts(registry: &RelationalSchemaRegistry) -> Vec<AggregateIdentityContractDeclaration> {
    registry
        .authority_snapshot()
        .relation_kinds
        .iter()
        .filter_map(|kind| {
            registry
                .relation_registration(kind.kind_id)
                .unwrap()
                .relation_integrity
                .aggregate_identity
                .clone()
        })
        .collect()
}

#[test]
fn mixed_orbit_and_normalized_component_sets_install_one_private_identity_owner() {
    let declaration = ComponentSchema::<false, false>::declaration().unwrap();
    let reordered = ComponentSchema::<true, false>::declaration().unwrap();
    assert_eq!(declaration.identity(), reordered.identity());
    let (layout, registry) = install_declaration(declaration);
    let (_, reversed) = install_declaration(reordered);
    let actual = contracts(&registry);
    assert_eq!(actual, contracts(&reversed));
    assert_eq!(actual.len(), 3);
    assert!(matches!(
        actual[0].source,
        AggregateIdentitySource::Orbit(_)
    ));
    assert_eq!(actual[1].source.relations().len(), 1);
    assert_eq!(actual[2].source.relations().len(), 2);
    assert_eq!(
        actual
            .iter()
            .map(|c| c.storage)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        1
    );
    assert_eq!(
        actual
            .iter()
            .map(|c| c.definition)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        3
    );
    for contract in &actual {
        assert!(!layout.is_application_entity_kind(contract.storage.aggregate_kind));
        assert!(!layout.is_application_relation_kind(contract.storage.lineage_kind));
        assert!(!layout.is_application_relation_kind(contract.membership_kind));
        assert!(contract
            .source
            .relations()
            .iter()
            .all(|kind| layout.is_application_relation_kind(*kind)));
    }
    // Frozen independently framed SHA-256 fixtures, not values derived by the issuer.
    assert_eq!(
        *actual[0].definition.bytes(),
        [
            67, 165, 30, 32, 167, 94, 127, 246, 3, 28, 152, 92, 70, 125, 235, 239, 165, 22, 183,
            128, 15, 18, 119, 223, 128, 216, 55, 1, 45, 82, 127, 252
        ]
    );
    assert_eq!(
        *actual[2].definition.bytes(),
        [
            137, 25, 134, 25, 54, 148, 218, 62, 0, 255, 196, 3, 138, 13, 195, 55, 231, 96, 86, 41,
            219, 197, 82, 126, 30, 190, 245, 43, 55, 11, 171, 27
        ]
    );
    let (_, changed) = install_declaration(ComponentSchema::<false, true>::declaration().unwrap());
    let changed = contracts(&changed);
    assert_eq!(actual[0].definition, changed[0].definition);
    assert_eq!(actual[1].definition, changed[1].definition);
    assert_ne!(actual[2].definition, changed[2].definition);
}

#[test]
fn authored_empty_set_is_refused_before_native_installation() {
    let set = ApplicationComponentSetRef::<Schema<false>, Shells, Node>::from_schema_identifiers(
        "Shells", "Node",
    );
    let node = ApplicationEntityRef::<Schema<false>, Node>::from_schema_identifier("Node");
    assert_eq!(
        ApplicationSchemaDeclarationBuilder::<Schema<false>>::for_schema()
            .entity(node)
            .components(ApplicationComponentSetDefinition::new(set))
            .build()
            .unwrap_err(),
        ApplicationSchemaDeclarationDenial::InvalidComponentSet,
    );
}
