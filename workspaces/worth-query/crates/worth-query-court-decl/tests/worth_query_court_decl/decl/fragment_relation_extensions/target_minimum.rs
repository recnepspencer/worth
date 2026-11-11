//! Target-side requirements may constrain only the extending fragment's own creations.
use super::*;
struct Target;
impl Entity for Target {
    const NAME: &'static str = "target";
    type Aspects = ();
    type Relations = ();
}
struct Targets;
impl Fragment for Targets {
    const NAME: &'static str = "targets";
    type Bindings = Bindings;
    type Members = (Target,);
}
struct Local<Members, Relations>(PhantomData<(Members, Relations)>);
impl<M: 'static, R: 'static> Fragment for Local<M, R> {
    const NAME: &'static str = "local";
    type Bindings = Bindings;
    type Members = (M, Extend<Item, (), R>);
}
struct Incoming<const N: usize, To>(PhantomData<To>);
impl<const N: usize, To: Entity> Relation for Incoming<N, To> {
    const NAME: &'static str = "incoming";
    type To = To;
    const CARDINALITY: ApplicationRelationCardinality = ApplicationRelationCardinality {
        source_min: if N == 0 { Some(1) } else { None },
        target_min: if N == 1 { Some(1) } else { None },
        target_max: if N == 1 { Some(1) } else { None },
        pair_min: if N == 2 { Some(1) } else { None },
        pair_max: if N == 2 { Some(1) } else { None },
        ..ApplicationRelationCardinality::unbounded()
    };
}
#[test]
fn extension_target_and_pair_minimums_accept_direct_and_nested_owned_targets() {
    macro_rules! allowed {
        ($members:ty, $n:literal) => {
            let schema = ApplicationSchemaDeclarationBuilder::<Root<()>>::from_fragment(Root::<(
                Local<$members, (Incoming<$n, Target>,)>,
                Owner,
            )>(
                PhantomData,
            ))
            .unwrap()
            .build()
            .unwrap();
            let actual = schema
                .member_provenance()
                .fragment_catalog()
                .fragment(Local::<$members, (Incoming<$n, Target>,)>(PhantomData))
                .unwrap()
                .relation::<Item, Incoming<$n, Target>>()
                .unwrap()
                .integrity
                .cardinality;
            assert_eq!(actual, Incoming::<$n, Target>::CARDINALITY);
        };
    }
    allowed!((Target,), 1);
    allowed!(((Targets,),), 1);
    allowed!(((Targets,),), 2);
}
#[test]
fn an_owned_target_never_permits_a_positive_source_minimum() {
    let denial = ApplicationSchemaDeclarationBuilder::<Root<()>>::from_fragment(Root::<(
        Owner,
        Local<(Target,), (Incoming<0, Target>,)>,
    )>(PhantomData))
    .err()
    .unwrap();
    assert_eq!(
        denial,
        Denial::ExtensionRelationMinimum {
            path: "root.local.item.incoming".into()
        }
    );
}
struct Sibling;
impl Fragment for Sibling {
    const NAME: &'static str = "local_sibling";
    type Bindings = Bindings;
    type Members = (Target,);
}
#[test]
fn a_sibling_with_the_same_path_prefix_does_not_own_the_extensions_target() {
    let denial = ApplicationSchemaDeclarationBuilder::<Root<()>>::from_fragment(Root::<(
        Owner,
        Sibling,
        Local<(), (Incoming<1, Target>,)>,
    )>(PhantomData))
    .err()
    .unwrap();
    assert_eq!(
        denial,
        Denial::ExtensionRelationMinimum {
            path: "root.local.item.incoming".into()
        }
    );
}
#[test]
fn an_imported_target_is_not_a_declaration_by_the_extending_fragment() {
    let denial = ApplicationSchemaDeclarationBuilder::<Root<()>>::from_fragment(Root::<(
        Owner,
        Sibling,
        Local<(Extend<Target, ()>,), (Incoming<2, Target>,)>,
    )>(PhantomData))
    .err()
    .unwrap();
    assert_eq!(
        denial,
        Denial::ExtensionRelationMinimum {
            path: "root.local.item.incoming".into()
        }
    );
}
