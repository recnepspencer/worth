//! Extension lowering retains owning entity identity and feature-local selection.
use worth_query_testkit_decl::fragment_extension_fixture::*;
use worth_query_testkit_decl::fragment_fixture::*;

#[test]
fn independent_extensions_land_on_one_entity_after_its_owner_is_declared() {
    let schema = declare(Fixture).unwrap();
    let catalog = schema.member_provenance().fragment_catalog();
    let annotation = aspect(Annotations, Item, Annotation)
        .installed(catalog)
        .unwrap();
    let classified = aspect(Classifications, Item, Classified)
        .installed(catalog)
        .unwrap();
    assert_eq!(annotation.entity(), "item");
    assert_eq!(classified.entity(), "item");
    assert!(annotation.optional() && classified.optional());
    let field: FieldSelection<_, _, _, String> =
        select_field(Annotations, Item, Annotation, "text");
    assert!(field.installed(catalog).is_ok());
    assert!(catalog
        .fragment(Fixture)
        .unwrap()
        .aspect::<Item, Annotation>()
        .is_none());
}

struct Composition<Members>(std::marker::PhantomData<Members>);
impl<M: 'static> Fragment for Composition<M> {
    const NAME: &'static str = "composition";
    type Bindings = Bindings;
    type Members = M;
}
impl<M: 'static> ApplicationSchema for Composition<M> {
    const NAME: &'static str = "composition";
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        unreachable!()
    }
}
struct Owner;
impl Fragment for Owner {
    const NAME: &'static str = "owner";
    type Bindings = Bindings;
    type Members = (Item,);
}
struct Extension;
impl Fragment for Extension {
    const NAME: &'static str = "extension";
    type Bindings = Bindings;
    type Members = (Extend<Item, (Option<Metrics>,)>,);
}

#[test]
fn extension_before_owner_is_resolved_but_undeclared_entity_is_refused_with_path() {
    let schema = ApplicationSchemaDeclarationBuilder::<Composition<()>>::from_fragment(
        Composition::<(Classifications, Owner)>(std::marker::PhantomData),
    )
    .unwrap()
    .build()
    .unwrap();
    assert!(aspect(Classifications, Item, Classified)
        .installed(schema.member_provenance().fragment_catalog())
        .is_ok());
    let denial = ApplicationSchemaDeclarationBuilder::<Composition<()>>::from_fragment(
        Composition::<(Classifications,)>(std::marker::PhantomData),
    )
    .err()
    .unwrap();
    assert_eq!(
        denial,
        ApplicationSchemaDeclarationDenial::UndeclaredExtensionEntity {
            path: "composition.classifications.item".to_owned()
        }
    );
}

#[test]
fn extension_cannot_repeat_an_owner_or_another_extensions_aspect_name() {
    let owner = ApplicationSchemaDeclarationBuilder::<Composition<()>>::from_fragment(
        Composition::<(Owner, Extension)>(std::marker::PhantomData),
    )
    .err()
    .unwrap();
    assert_eq!(
        owner,
        ApplicationSchemaDeclarationDenial::DuplicateExtensionMember {
            first: "composition.owner.item.metrics".to_owned(),
            path: "composition.extension.item.metrics".to_owned()
        }
    );
    struct Repeated;
    impl Fragment for Repeated {
        const NAME: &'static str = "repeated";
        type Bindings = Bindings;
        type Members = (Extend<Item, (Option<Classified>,)>,);
    }
    let extension = ApplicationSchemaDeclarationBuilder::<Composition<()>>::from_fragment(
        Composition::<(Owner, Classifications, Repeated)>(std::marker::PhantomData),
    )
    .err()
    .unwrap();
    assert_eq!(
        extension,
        ApplicationSchemaDeclarationDenial::DuplicateExtensionMember {
            first: "composition.classifications.item.classified".to_owned(),
            path: "composition.repeated.item.classified".to_owned()
        }
    );
}

#[test]
fn repeated_names_inside_one_extension_keep_the_extension_denial() {
    struct Repeated;
    impl Fragment for Repeated {
        const NAME: &'static str = "repeated";
        type Bindings = Bindings;
        type Members = (Extend<Item, (Option<Classified>, Option<Classified>)>,);
    }
    let denial = ApplicationSchemaDeclarationBuilder::<Composition<()>>::from_fragment(
        Composition::<(Owner, Repeated)>(std::marker::PhantomData),
    )
    .err()
    .unwrap();
    assert_eq!(
        denial,
        ApplicationSchemaDeclarationDenial::DuplicateExtensionMember {
            first: "composition.repeated.item.classified".to_owned(),
            path: "composition.repeated.item.classified".to_owned(),
        }
    );
}

#[test]
fn an_extension_requires_another_fragment_and_the_exact_entity_marker() {
    struct SelfExtension;
    impl Fragment for SelfExtension {
        const NAME: &'static str = "self_extension";
        type Bindings = Bindings;
        type Members = (Item, Extend<Item, (Option<Classified>,)>);
    }
    let same = ApplicationSchemaDeclarationBuilder::<Composition<()>>::from_fragment(SelfExtension)
        .err()
        .unwrap();
    assert_eq!(
        same,
        ApplicationSchemaDeclarationDenial::UndeclaredExtensionEntity {
            path: "self_extension.item".to_owned()
        }
    );
    struct Impostor;
    impl Entity for Impostor {
        const NAME: &'static str = "item";
        type Aspects = ();
        type Relations = ();
    }
    struct WrongMarker;
    impl Fragment for WrongMarker {
        const NAME: &'static str = "wrong_marker";
        type Bindings = Bindings;
        type Members = (Extend<Impostor, (Option<Classified>,)>,);
    }
    let wrong = ApplicationSchemaDeclarationBuilder::<Composition<()>>::from_fragment(
        Composition::<(Owner, WrongMarker)>(std::marker::PhantomData),
    )
    .err()
    .unwrap();
    assert_eq!(
        wrong,
        ApplicationSchemaDeclarationDenial::UndeclaredExtensionEntity {
            path: "composition.wrong_marker.item".to_owned()
        }
    );
}

#[test]
fn extension_identity_revision_and_physical_name_match_the_owners_optional_aspect() {
    struct Node<A>(std::marker::PhantomData<A>);
    impl<A: 'static> Entity for Node<A> {
        const NAME: &'static str = "node";
        type Aspects = A;
        type Relations = ();
    }
    struct Model<A>(std::marker::PhantomData<A>);
    impl<A: 'static> Fragment for Model<A> {
        const NAME: &'static str = "model";
        type Bindings = Bindings;
        type Members = (Node<A>,);
    }
    struct Addition;
    impl Fragment for Addition {
        const NAME: &'static str = "addition";
        type Bindings = Bindings;
        type Members = (Extend<Node<()>, (Option<Classified>,)>,);
    }
    let owned = ApplicationSchemaDeclarationBuilder::<Composition<()>>::from_fragment(
        Composition::<(Model<(Option<Classified>,)>,)>(std::marker::PhantomData),
    )
    .unwrap()
    .build()
    .unwrap();
    let extended = ApplicationSchemaDeclarationBuilder::<Composition<()>>::from_fragment(
        Composition::<(Model<()>, Addition)>(std::marker::PhantomData),
    )
    .unwrap()
    .build()
    .unwrap();
    // The complete flat members carry identity, revision, physical name and field loci.
    assert_eq!(owned.erased().members(), extended.erased().members());
}

#[test]
fn a_principal_requirement_accepts_an_identity_field_selected_from_an_extension() {
    use worth_query_testkit_decl::fragment_auth_fixture::{IdentityValueBinding, ItemIdentity};
    struct Principal;
    impl Entity for Principal {
        const NAME: &'static str = "extended_principal";
        type Aspects = ();
        type Relations = ();
    }
    struct Model;
    impl Fragment for Model {
        const NAME: &'static str = "principal_model";
        type Bindings = Bindings;
        type Members = (Principal,);
    }
    struct Identity;
    impl Fragment for Identity {
        const NAME: &'static str = "identity";
        type Bindings = Bindings;
        type Members = (Extend<Principal, (Option<ItemIdentity>,)>,);
    }
    struct Mapping;
    impl Entity for Mapping {
        const NAME: &'static str = "mapping";
        type Aspects = (worth_query_testkit_decl::fragment_auth_fixture::MappingIdentity,);
        type Relations = (MapsTo,);
    }
    struct MapsTo;
    impl Relation for MapsTo {
        const NAME: &'static str = "maps_to";
        type To = Principal;
    }
    struct Authentication;
    impl Fragment for Authentication {
        const NAME: &'static str = "extension_authentication";
        type Bindings = Bindings;
        type Members = (Mapping,);
    }
    struct Actor;
    let selection: FieldSelection<_, _, _, u64> =
        select_field(Identity, Principal, ItemIdentity, "id");
    let principal = ApplicationPrincipalBindingRef::<
        Composition<()>,
        Actor,
        Mapping,
        Principal,
        u64,
        IdentityValueBinding,
    >::from_requirements(
        "actor",
        ApplicationPrincipalBindingRequirements {
            mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_record(
                record_field(
                    Authentication,
                    Mapping,
                    worth_query_testkit_decl::fragment_auth_fixture::MappingIdentity,
                    "external",
                ),
            ),
            mapping_status: ApplicationPrincipalMappingStatusRequirement::from_record(
                record_field(
                    Authentication,
                    Mapping,
                    worth_query_testkit_decl::fragment_auth_fixture::MappingIdentity,
                    "status",
                ),
            ),
            target: ApplicationPrincipalTargetRequirement::from_relation(relation(
                Authentication,
                Mapping,
                MapsTo,
            )),
            principal_identity: ApplicationPrincipalIdentityRequirement::from_record(
                Identity, selection,
            ),
        },
    );
    let schema = ApplicationSchemaDeclarationBuilder::<Composition<()>>::from_fragment(
        Composition::<(Model, Identity, Authentication)>(std::marker::PhantomData),
    )
    .unwrap()
    .principal_binding(principal)
    .build()
    .unwrap();
    assert!(selection
        .installed(schema.member_provenance().fragment_catalog())
        .is_ok());
}
