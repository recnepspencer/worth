//! The Optional Struct aspect `summary` and the fieldless-kind creates that
//! write it, for the world Query-compiled fieldless kinds match.
use super::*;

pub(super) fn optional_summary_struct_aspect() -> (
    worth_foundational::facade::AspectBinding,
    worth_foundational::facade::AspectContract,
) {
    use worth_foundational::facade::*;
    let (binding, base) =
        worth_testkit_relational::aspect_storage::definition::entity_summary_struct_aspect(
            aspect_key("summary"),
            field_key("summary"),
        );
    let AspectShape::Struct(shape) = base.shape().clone() else {
        unreachable!("summary aspect is a struct");
    };
    let contract = aspects()
        .contract()
        .for_key(base.key().clone())
        .identified_by(base.identity())
        .at_revision(base.revision())
        .struct_with(
            shape,
            base.masks().clone(),
            AbsenceLaw::Optional,
            base.equivalence(),
            base.evolution(),
        );
    (binding, contract)
}

pub(super) fn optional_struct_create_intent(key: &str, title: &str) -> MutationIntent {
    MutationIntent::Create(CreateIntent::Entity(EntitySpec {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: ClientKey::raw(key),
        fields: string_aspect_field_patch([(aspect_key("summary"), field_key("title"), title)]),
    }))
}
