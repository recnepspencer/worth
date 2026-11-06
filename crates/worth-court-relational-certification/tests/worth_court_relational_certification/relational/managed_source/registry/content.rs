//! The same Native Source schema, with one actual typed content field.
use super::*;
use worth_foundational::facade::{
    CanonicalDigestId, ContentFormat, FieldDeclaration, FieldRequirement, StructAspectShape,
};
pub(super) fn declaration() -> DeclaredAspectContractBinding {
    let value_field = FieldDeclaration::new(
        field("value"),
        ScalarAspectType::ContentRef,
        FieldRequirement::Optional,
        AbsenceLaw::Optional,
        AspectEvolutionPolicy::ExplicitBreakRequired,
    )
    .unwrap()
    .with_codec(CanonicalDigestId::new([12; 32]));
    DeclaredAspectContractBinding {
        binding: AspectBinding::EntityField {
            field: field("history-data"),
        },
        contract: AspectContract::struct_aspect(
            key(),
            AspectIdentity(0x91900401),
            worth_foundational::facade::AspectContractRevision(1),
            StructAspectShape::new([value_field]).unwrap(),
        ),
    }
}
pub(super) fn formats(registry: RelationalSchemaRegistry) -> RelationalSchemaRegistry {
    registry
        .register_content_format(
            CanonicalDigestId::new([11; 32]),
            Arc::new(crate::content::fixture::Codec {
                descriptor: ContentFormat::new("sample-grid", 1).unwrap(),
                alter: false,
            }),
        )
        .unwrap()
        .bind_content_field(
            CanonicalDigestId::new([12; 32]),
            CanonicalDigestId::new([11; 32]),
        )
        .unwrap()
}
