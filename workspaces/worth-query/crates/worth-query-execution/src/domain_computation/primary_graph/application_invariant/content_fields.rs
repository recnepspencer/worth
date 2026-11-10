//! Installed content field metadata, lowered once alongside each rule.
use std::collections::BTreeMap;
use worth_foundational::facade::AspectFieldLocator;
use worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus;
use worth_query_installation::facade::WorthQueryInstalledApplicationValueBindingCatalog;
use worth_relational::facade::identity::KindId;

pub(in crate::domain_computation::primary_graph) struct InvariantContentFields {
    pub bindings: WorthQueryInstalledApplicationValueBindingCatalog,
    pub(super) fields: BTreeMap<ApplicationFieldBindingLocus, InvariantContentField>,
}
impl InvariantContentFields {
    pub(in crate::domain_computation::primary_graph) fn lower(
        bindings: WorthQueryInstalledApplicationValueBindingCatalog,
        layout: &super::super::schema_layout::WorthQueryPrimaryGraphLayout,
    ) -> Self {
        let fields = bindings
            .bindings()
            .filter_map(|binding| {
                binding.content_format()?;
                let locus = binding.locus();
                let field = layout.field(locus.entity(), locus.aspect(), locus.field())?;
                Some((
                    locus.clone(),
                    InvariantContentField {
                        kind: field.entity_kind,
                        locator: field.locator.clone(),
                        aspect_absence: layout.aspect_contract(locus.entity(), field.locator.aspect().aspect_key())?.absence(),
                        presence: if layout.field_is_optional(locus.entity(), &field.locator) { worth_query_declaration::facade::application_schema::ApplicationFieldPresence::Optional } else { worth_query_declaration::facade::application_schema::ApplicationFieldPresence::Required },
                    },
                ))
            })
            .collect();
        Self { bindings, fields }
    }
}

pub(super) struct InvariantContentField {
    pub kind: KindId,
    pub locator: AspectFieldLocator,
    pub aspect_absence: worth_foundational::facade::AbsenceLaw,
    pub presence: worth_query_declaration::facade::application_schema::ApplicationFieldPresence,
}
