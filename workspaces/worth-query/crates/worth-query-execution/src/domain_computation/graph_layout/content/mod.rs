//! Lower actual installed recipes and their full field-meaning aliases into Native.
mod codec;
use std::sync::Arc;
use worth_foundational::facade::{CanonicalDigestId, ContentFormat};
use worth_query_installation::facade::WorthQueryInstalledApplicationValueBindingCatalog as Catalog;
use worth_relational::facade::{
    content::RelationalContentDenial as Denial, schema::RelationalSchemaRegistry,
};
use {
    crate::domain_computation::graph_layout::denial::WorthQueryPrimaryGraphInstallationDenial as Stop,
    crate::domain_computation::graph_layout::denial::WorthQueryPrimaryGraphInstallationDenialKind as Kind,
};

pub(super) fn lower(
    mut registry: RelationalSchemaRegistry,
    bindings: &Catalog,
) -> Result<RelationalSchemaRegistry, Stop> {
    for recipe in bindings.content_formats() {
        let declaration = recipe.declaration();
        let maximum = usize::try_from(declaration.maximum_frame_bytes)
            .map_err(|_| refusal(Denial::ArithmeticOverflow))?;
        let descriptor = ContentFormat::new(&declaration.name, declaration.version)
            .map_err(|_| refusal(Denial::InvalidFormat))?;
        registry = registry
            .register_content_format(
                CanonicalDigestId::new(declaration.identity),
                Arc::new(codec::Codec {
                    descriptor,
                    maximum,
                    recipe: recipe.clone(),
                }),
            )
            .map_err(refusal)?;
    }
    for binding in bindings.bindings() {
        if let Some(recipe) = binding.content_format() {
            let codec = binding
                .content_codec()
                .ok_or_else(|| refusal(Denial::EncodingInconsistency))?;
            registry = registry
                .bind_content_field(codec, CanonicalDigestId::new(recipe.declaration().identity))
                .map_err(refusal)?;
        }
    }
    Ok(registry)
}
fn refusal(cause: Denial) -> Stop {
    Stop::new(
        Kind::ContentAdmissionRejected,
        "Native content format installation refused",
    )
    .with_content_cause(cause)
}
