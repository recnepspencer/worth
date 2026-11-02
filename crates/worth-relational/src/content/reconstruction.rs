//! Until complete byte reconstruction lands, no decoded content schema can select a root.
use crate::branch::RelationalBranchRootCaptureDenial as Denial;
use crate::schema::data::RelationalSchemaRegistry;
use worth_foundational::facade::{AspectShape, ScalarAspectType};

pub(crate) fn require_content_free_readmission(
    registry: &RelationalSchemaRegistry,
) -> Result<(), Denial> {
    if !registry.content_formats.snapshot().is_empty() {
        return Err(Denial::UnadmittedContent);
    }
    let declarations = registry
        .entity_kinds
        .values()
        .map(|kind| &kind.aspect_contract_declarations)
        .chain(
            registry
                .relation_kinds
                .values()
                .map(|kind| &kind.aspect_contract_declarations),
        );
    for declaration in declarations {
        for aspect in &declaration.aspects {
            let content = match aspect.contract.shape() {
                AspectShape::Content | AspectShape::Scalar(ScalarAspectType::ContentRef) => true,
                AspectShape::Struct(shape) => shape
                    .fields()
                    .iter()
                    .any(|field| field.value_type() == ScalarAspectType::ContentRef),
                _ => false,
            };
            if content {
                return Err(Denial::UnadmittedContent);
            }
        }
    }
    Ok(())
}
