//! An initially empty component member kind, separate from the operation root.
use super::super::schema::*;
use worth_query_declaration::{facade::application_schema::*, *};
worth_query_entity!(pub Spare for S:OrbitModel);
worth_query_aspect!(pub SpareData for S:OrbitModel, Spare; identity=AspectIdentity(0x9181_4151), revision=AspectContractRevision(1));
worth_query_field!(pub SpareKey for S:OrbitModel, Spare, SpareData: u64 => U64ApplicationValueBinding, read_only, equality);
pub struct SpareCoupling;
impl SpareCoupling {
    pub fn reference<S: OrbitModel>() -> ApplicationRelationRef<S, Self, Spare, Spare> {
        ApplicationRelationRef::from_schema_identifiers(
            "SpareCoupling",
            "Spare",
            "Spare",
            ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
        )
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
        .entity(Spare::reference())
        .aspect(Spare::reference(), SpareData::reference())
        .field(Spare::reference(), SpareKey::reference())
        .relation(
            SpareCoupling::reference(),
            Spare::reference(),
            Spare::reference(),
        )
        .components(
            ApplicationComponentSetDefinition::new(Spares::reference())
                .coupled_by(SpareCoupling::reference()),
        )
        .operation_read_entity(ReplaceNext::reference(), Spare::reference())
        .operation_read_field(ReplaceNext::reference(), SpareKey::reference())
        .operation_expected_fact(ReplaceNext::reference(), SpareKey::reference())
        .operation_create(ReplaceNext::reference(), Spare::reference())
        .operation_delete(ReplaceNext::reference(), Spare::reference())
        .operation_write(ReplaceNext::reference(), SpareKey::reference())
}
