use super::{DeclaredPreImageDemand, DeclaredPreImageDemandDenial, DeclaredPreImageLocus};
use crate::application_schema::{
    fragment::{self, *},
    *,
};

struct Schema;
struct Fixture;
impl Fragment for Fixture {
    const NAME: &'static str = "inverse_fixture";
    type Bindings = Bindings;
    type Members = (FirstEntity, SecondEntity);
}
struct Bindings;
impl BindingSet for Bindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<u64> for Bindings {
    type Binding = U64ApplicationValueBinding;
}
struct FirstEntity;
impl Entity for FirstEntity {
    const NAME: &'static str = "FirstEntity";
    type Aspects = (Facts, FirstAspect, SecondAspect);
    type Relations = ();
}
struct SecondEntity;
impl Entity for SecondEntity {
    const NAME: &'static str = "SecondEntity";
    type Aspects = (Facts,);
    type Relations = ();
}
struct Facts;
impl Aspect for Facts {
    const NAME: &'static str = "Facts";
    type Row = Pair;
}
struct FirstAspect;
impl Aspect for FirstAspect {
    const NAME: &'static str = "FirstAspect";
    type Row = Pair;
}
struct SecondAspect;
impl Aspect for SecondAspect {
    const NAME: &'static str = "SecondAspect";
    type Row = Pair;
}
struct Pair {
    first: u64,
    second: u64,
}
impl fragment::Record<Pair> for Bindings {
    fn record() -> impl Shape<Pair, Self> {
        field("first", |row: &Pair| row.first)
            .and("second", |row: &Pair| row.second)
            .build(|first, second, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(Pair { first, second })
            })
    }
}
type First = FieldSelection<Fixture, FirstEntity, Facts, u64>;
type Second = FieldSelection<Fixture, SecondEntity, Facts, u64>;
type FirstPart = FieldSelection<Fixture, FirstEntity, FirstAspect, u64>;
type SecondPart = FieldSelection<Fixture, FirstEntity, SecondAspect, u64>;

#[test]
fn duplicate_exact_locus_is_denied() {
    let first = locus(First::at("first"));
    assert_eq!(
        DeclaredPreImageDemand::new([first.clone(), first], 64),
        Err(DeclaredPreImageDemandDenial::DuplicateLocus)
    );
}
#[test]
fn heterogeneous_loci_defer_authored_anchor_admission_to_installation() {
    let demand =
        DeclaredPreImageDemand::new([locus(First::at("first")), locus(Second::at("second"))], 64)
            .unwrap();
    assert_eq!(demand.loci().len(), 2);
}
#[test]
fn multiple_exact_fields_on_one_entity_role_are_admissible() {
    let demand = DeclaredPreImageDemand::new(
        [
            locus(FirstPart::at("first")),
            locus(SecondPart::at("second")),
        ],
        64,
    )
    .unwrap();
    assert_eq!(demand.loci().len(), 2);
}
fn locus<F: RecordSelection>(field: F) -> DeclaredPreImageLocus<Schema> {
    DeclaredPreImageLocus::from_field(field).expect("the selected record column exists")
}
