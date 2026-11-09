//! Boundary profiles retain their distinct mandatory orbit laws.
use super::*;
use sheet_boundary::Profile;
use worth_query_decl::facade::application_schema::fragment::{self, *};
pub struct BoundaryModel<B>(PhantomData<fn() -> B>);
impl<B: Profile> Fragment for BoundaryModel<B> {
    const NAME: &'static str = "boundary_model";
    type Bindings = OwnershipModelBindings;
    type Members = (BoundaryOccurrence<B>, sheet_boundary::split_walk::Fork);
}
pub struct BoundaryOccurrence<B>(PhantomData<fn() -> B>);
impl<B: Profile> Entity for BoundaryOccurrence<B> {
    const NAME: &'static str = "Occurrence";
    type Aspects = (BoundaryPosition<B>,);
    type Relations = B::Relations;
}
impl<B: Profile> BoundaryOccurrence<B> {
    pub fn reference<S>() -> ApplicationEntityRef<S, Self> {
        ApplicationEntityRef::from_schema_identifier("Occurrence")
    }
}
pub struct BoundaryPosition<B>(PhantomData<fn() -> B>);
impl<B: Profile> Aspect for BoundaryPosition<B> {
    const NAME: &'static str = "Position";
    type Row = BoundaryRow<B>;
}
pub struct BoundaryRow<B>(u64, PhantomData<fn() -> B>);
impl<B: Profile> fragment::Record<BoundaryRow<B>> for OwnershipModelBindings {
    fn record() -> impl Shape<BoundaryRow<B>, Self> {
        let number = fragment::field("Number", |row: &BoundaryRow<B>| row.0)
            .equality_queryable(true)
            .read_only();
        let number = if B::UNIQUE { number.unique() } else { number };
        number.build(|number, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(BoundaryRow(number, PhantomData))
        })
    }
}
pub type Number<B> =
    FieldSelection<BoundaryModel<B>, BoundaryOccurrence<B>, BoundaryPosition<B>, u64>;
impl fragment::Entity for sheet_boundary::split_walk::Fork {
    const NAME: &'static str = "Fork";
    type Aspects = (sheet_boundary::split_walk::ForkPosition,);
    type Relations = (sheet_boundary::split_walk::Branch,);
}
impl sheet_boundary::split_walk::Fork {
    pub fn reference<S>() -> ApplicationEntityRef<S, Self> {
        ApplicationEntityRef::from_schema_identifier("Fork")
    }
}
impl Aspect for sheet_boundary::split_walk::ForkPosition {
    const NAME: &'static str = "ForkPosition";
    type Row = ForkRow;
}
pub struct ForkRow(u64);
impl fragment::Record<ForkRow> for OwnershipModelBindings {
    fn record() -> impl Shape<ForkRow, Self> {
        fragment::field("ForkNumber", |row: &ForkRow| row.0)
            .equality_queryable(true)
            .read_only()
            .unique()
            .build(|number, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ForkRow(number))
            })
    }
}
pub type ForkNumber = FieldSelection<
    BoundaryModel<currentness_guards::Guards>,
    sheet_boundary::split_walk::Fork,
    sheet_boundary::split_walk::ForkPosition,
    u64,
>;
impl fragment::Relation for sheet_boundary::split_walk::Branch {
    const NAME: &'static str = "Branch";
    type To = sheet_boundary::split_walk::Fork;
}
impl fragment::Relation for sheet_boundary::Successor {
    const NAME: &'static str = "BoundaryNext";
    type To = BoundaryOccurrence<sheet_boundary::Owned>;
    const CARDINALITY: ApplicationRelationCardinality =
        ApplicationRelationCardinality::new(Some(1), Some(1), Some(1), Some(1), None, Some(1));
    const DELETION: ApplicationRelationDeletionPolicy =
        ApplicationRelationDeletionPolicy::RejectDeleteWithLiveRelations;
    const ORBIT: bool = true;
}
impl fragment::Relation for foreign_aggregate::Next {
    const NAME: &'static str = "ForeignNext";
    type To = BoundaryOccurrence<currentness_guards::Guards>;
    const CARDINALITY: ApplicationRelationCardinality =
        ApplicationRelationCardinality::new(Some(1), Some(1), Some(1), Some(1), None, Some(1));
    const DELETION: ApplicationRelationDeletionPolicy =
        ApplicationRelationDeletionPolicy::RejectDeleteWithLiveRelations;
    const ORBIT: bool = true;
}
