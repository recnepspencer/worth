//! An initially empty component member kind, separate from the operation root.
use worth_query_declaration::facade::application_schema::fragment;

pub struct SpareCoupling;
impl fragment::Relation for SpareCoupling {
    const NAME: &'static str = "SpareCoupling";
    type To = Spare;
}
impl SpareCoupling {
    pub fn reference<S: OrbitModel>() -> ApplicationRelationRef<S, Self, Spare, Spare> {
        fragment::relation(OrbitRecords::<true>, Spare, Self)
    }
}
pub struct Spares;
impl Spares {
    pub fn reference<S: OrbitModel>() -> ApplicationComponentSetRef<S, Self, Spare> {
        ApplicationComponentSetRef::new("Spares", Spare::reference())
    }
}
pub(in super::super) fn declare<S: OrbitModel>(
    schema: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    schema
        .components(
            ApplicationComponentSetDefinition::new(Spares::reference())
                .coupled_by(SpareCoupling::reference()),
        )
        .operation_read_entity(ReplaceNext::reference(), Spare::reference())
        .operation_read_field(ReplaceNext::reference(), SpareKey::at("SpareKey"))
        .operation_expected_fact(ReplaceNext::reference(), SpareKey::at("SpareKey"))
        .operation_create(ReplaceNext::reference(), Spare::reference())
        .operation_delete(ReplaceNext::reference(), Spare::reference())
        .operation_write(ReplaceNext::reference(), SpareKey::at("SpareKey"))
}

pub use super::super::schema::records::{Spare, SpareKey};

use super::super::schema::*;
use worth_query_declaration::facade::application_schema::*;
