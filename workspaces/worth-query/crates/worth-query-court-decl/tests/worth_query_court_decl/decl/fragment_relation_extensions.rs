//! Outgoing extensions retain owned identity and refuse incomplete creation contracts.
use std::marker::PhantomData;
use worth_query_testkit_decl::fragment_extension_fixture::*;
use worth_query_testkit_decl::fragment_fixture::*;

struct RelationAnnotations;
impl Fragment for RelationAnnotations {
    const NAME: &'static str = "annotations";
    type Bindings = Bindings;
    type Members = ((Extend<Item, (), (WithAspects<Annotates, (Option<EdgeWeight>,)>,)>,),);
}
type Denial = ApplicationSchemaDeclarationDenial;
struct Root<M>(PhantomData<M>);
impl<M: 'static> Fragment for Root<M> {
    const NAME: &'static str = "root";
    type Bindings = Bindings;
    type Members = M;
}
impl ApplicationSchema for Root<()> {
    const NAME: &'static str = "root";
    fn declaration() -> Result<ApplicationSchemaDeclaration<Self>, Denial> {
        unreachable!()
    }
}
struct Owner;
impl Fragment for Owner {
    const NAME: &'static str = "owner";
    type Bindings = Bindings;
    type Members = (
        Item,
        worth_query_testkit_decl::fragment_auth_fixture::OptionalItem,
    );
}

#[test]
fn extension_relations_from_two_fragments_land_on_the_shared_entity() {
    let schema = ApplicationSchemaDeclarationBuilder::<Root<()>>::from_fragment(Root::<(
        RelationAnnotations,
        Owner,
        Classifications,
    )>(PhantomData))
    .unwrap()
    .build()
    .unwrap();
    let catalog = schema.member_provenance().fragment_catalog();
    let first = catalog
        .fragment(RelationAnnotations)
        .unwrap()
        .relation::<Item, Annotates>()
        .unwrap();
    let second = catalog
        .fragment(Classifications)
        .unwrap()
        .relation::<Item, ClassifiedBy>()
        .unwrap();
    assert_eq!(first.from, "item");
    assert_eq!(second.from, "item");
    assert_eq!(second.to, "item");
    let field: RelationFieldSelection<_, _, _, _, u64> =
        select_relation_field(RelationAnnotations, Item, Annotates, EdgeWeight, "value");
    assert!(field.checked().is_ok());
    struct Nested;
    impl Fragment for Nested {
        const NAME: &'static str = "nested";
        type Bindings = Bindings;
        type Members = ((Extend<Item, (), (WithAspects<Annotates, (Option<EdgeWeight>,)>,)>,),);
    }
    let nested: RelationFieldSelection<_, _, _, _, u64> =
        select_relation_field(Nested, Item, Annotates, EdgeWeight, "value");
    assert!(nested.checked().is_ok());
    assert!(catalog
        .fragment(Owner)
        .unwrap()
        .relation::<Item, Annotates>()
        .is_none());
}
struct Addition<R>(PhantomData<R>);
impl<R: 'static> Fragment for Addition<R> {
    const NAME: &'static str = "addition";
    type Bindings = Bindings;
    type Members = (Extend<Item, (), R>,);
}
#[test]
fn extension_relation_duplicates_name_both_paths_and_precede_minimum_refusal() {
    let owned = ApplicationSchemaDeclarationBuilder::<Root<()>>::from_fragment(Root::<(
        Owner,
        Addition<(Next,)>,
    )>(PhantomData))
    .err()
    .unwrap();
    assert_eq!(
        owned,
        Denial::DuplicateExtensionMember {
            first: "root.owner.item.next".into(),
            path: "root.addition.item.next".into()
        }
    );
    let extended =
        ApplicationSchemaDeclarationBuilder::<Root<()>>::from_fragment(Root::<(
            Owner,
            RelationAnnotations,
            Addition<(WithAspects<Annotates, (Option<EdgeWeight>,)>,)>,
        )>(PhantomData))
        .err()
        .unwrap();
    assert_eq!(
        extended,
        Denial::DuplicateExtensionMember {
            first: "root.annotations.item.annotates".into(),
            path: "root.addition.item.annotates".into()
        }
    );
    let repeated =
        ApplicationSchemaDeclarationBuilder::<Root<()>>::from_fragment(Root::<(
            Owner,
            Addition<(Annotates, Annotates)>,
        )>(PhantomData))
        .err()
        .unwrap();
    assert_eq!(
        repeated,
        Denial::DuplicateExtensionMember {
            first: "root.addition.item.annotates".into(),
            path: "root.addition.item.annotates".into()
        }
    );
    struct RequiredDuplicate;
    impl Relation for RequiredDuplicate {
        const NAME: &'static str = "next";
        type To = Item;
        const CARDINALITY: ApplicationRelationCardinality = ApplicationRelationCardinality {
            source_min: Some(1),
            ..ApplicationRelationCardinality::unbounded()
        };
    }
    let precedence =
        ApplicationSchemaDeclarationBuilder::<Root<()>>::from_fragment(Root::<(
            Owner,
            Addition<(RequiredDuplicate,)>,
        )>(PhantomData))
        .err()
        .unwrap();
    assert_eq!(precedence, owned);
}
struct Minimum<const N: usize>;
impl<const N: usize> Relation for Minimum<N> {
    const NAME: &'static str = "minimum";
    type To = Item;
    const CARDINALITY: ApplicationRelationCardinality = ApplicationRelationCardinality {
        source_min: if N == 0 { Some(1) } else { None },
        target_min: if N == 1 { Some(1) } else { None },
        pair_min: if N == 2 { Some(1) } else { None },
        ..ApplicationRelationCardinality::unbounded()
    };
}
#[test]
fn extension_relation_source_target_and_pair_minimums_are_refused_with_the_path() {
    macro_rules! refused {
        ($relation:ty) => {
            let denial = ApplicationSchemaDeclarationBuilder::<Root<()>>::from_fragment(Root::<(
                Owner,
                Addition<($relation,)>,
            )>(
                PhantomData,
            ))
            .err()
            .unwrap();
            assert_eq!(
                denial,
                Denial::ExtensionRelationMinimum {
                    path: "root.addition.item.minimum".into()
                }
            );
        };
    }
    refused!(Minimum<0>);
    refused!(WithAspects<Minimum<1>, (Option<EdgeWeight>,)>);
    refused!(Minimum<2>);
    let missing =
        ApplicationSchemaDeclarationBuilder::<Root<()>>::from_fragment(Root::<(
            Addition<(Minimum<0>,)>,
        )>(PhantomData))
        .err()
        .unwrap();
    assert_eq!(
        missing,
        Denial::UndeclaredExtensionEntity {
            path: "root.addition.item".into()
        }
    );
}
struct Node<R>(PhantomData<R>);
impl<R: 'static> Entity for Node<R> {
    const NAME: &'static str = "node";
    type Aspects = ();
    type Relations = R;
}
struct Model<R>(PhantomData<R>);
impl<R: 'static> Fragment for Model<R> {
    const NAME: &'static str = "model";
    type Bindings = Bindings;
    type Members = (Node<R>, Item);
}
struct Policies;
impl Relation for Policies {
    const NAME: &'static str = "policies";
    type To = Item;
    const SELF_EDGES: bool = false;
    const ACYCLIC: bool = true;
    const CARDINALITY: ApplicationRelationCardinality = ApplicationRelationCardinality {
        source_max: Some(3),
        target_max: Some(4),
        pair_max: Some(1),
        ..ApplicationRelationCardinality::unbounded()
    };
}
struct Feature;
impl Fragment for Feature {
    const NAME: &'static str = "feature";
    type Bindings = Bindings;
    type Members = (Extend<Node<()>, (), (WithAspects<Policies, (EdgeWeight,)>,)>,);
}
#[test]
fn extension_relation_and_payload_identity_match_owned_members_and_keep_policies() {
    let owned = ApplicationSchemaDeclarationBuilder::<Root<()>>::from_fragment(Root::<(
        Model<(WithAspects<Policies, (EdgeWeight,)>,)>,
    )>(PhantomData))
    .unwrap()
    .build()
    .unwrap();
    let extended =
        ApplicationSchemaDeclarationBuilder::<Root<()>>::from_fragment(
            Root::<(Model<()>, Feature)>(PhantomData),
        )
        .unwrap()
        .build()
        .unwrap();
    assert_eq!(owned.erased().members(), extended.erased().members());
    let expected: ApplicationRelationRef<Root<()>, _, _, _> = relation(
        Model::<(WithAspects<Policies, (EdgeWeight,)>,)>(PhantomData),
        Node::<(WithAspects<Policies, (EdgeWeight,)>,)>(PhantomData),
        Policies,
    );
    let actual: ApplicationRelationRef<Root<()>, _, _, _> =
        relation(Feature, Node::<()>(PhantomData), Policies);
    assert_eq!(actual.integrity(), expected.integrity());
}

#[test]
fn extension_member_collisions_across_aspects_and_relations_name_both_paths() {
    struct MetricsLink;
    impl Relation for MetricsLink {
        const NAME: &'static str = "metrics";
        type To = Item;
    }
    let denial = ApplicationSchemaDeclarationBuilder::<Root<()>>::from_fragment(Root::<(
        Owner,
        Addition<(MetricsLink,)>,
    )>(PhantomData))
    .err()
    .unwrap();
    assert_eq!(
        denial,
        Denial::DuplicateExtensionMember {
            first: "root.owner.item.metrics".into(),
            path: "root.addition.item.metrics".into()
        }
    );
    struct RelationName;
    impl Aspect for RelationName {
        const NAME: &'static str = "annotates";
        type Row = u64;
    }
    struct Later;
    impl Fragment for Later {
        const NAME: &'static str = "later";
        type Bindings = Bindings;
        type Members = (Extend<Item, (Option<RelationName>,)>,);
    }
    let denial = ApplicationSchemaDeclarationBuilder::<Root<()>>::from_fragment(Root::<(
        Owner,
        RelationAnnotations,
        Later,
    )>(PhantomData))
    .err()
    .unwrap();
    assert_eq!(
        denial,
        Denial::DuplicateExtensionMember {
            first: "root.annotations.item.annotates".into(),
            path: "root.later.item.annotates".into()
        }
    );
}

mod target_minimum;
