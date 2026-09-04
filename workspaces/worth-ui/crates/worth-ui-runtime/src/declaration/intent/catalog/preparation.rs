use std::collections::BTreeMap;
use std::sync::Arc;

use crate::capability::{FrozenIntentDefinitionCapabilities, UiSemanticInteractionFamily};

use super::UiIntentCatalog;
use crate::declaration::intent::{
    UiCanonicalIntentDeclaration, UiIntentCatalogPreparationDenial,
    UiIntentConfirmationRouteBinding, UiIntentDeclarationIdentity, UiIntentRouteBinding,
};

#[path = "route_preparation.rs"]
mod route_preparation;

const MAXIMUM_INTENT_ROUTES: usize = 65_536;
const MAXIMUM_INTENT_DECLARATIONS: usize = 65_536;

struct ResolvedIntentRoutes {
    product: Vec<UiIntentRouteBinding>,
    confirmation: Vec<UiIntentConfirmationRouteBinding>,
}

pub(super) fn prepare(
    material: &crate::declaration::WorthUiAuthoredIntentMaterial,
    definitions: &FrozenIntentDefinitionCapabilities,
    graph: &crate::graph::UiGraphSnapshot,
    query: &worth_ui_query_binding::WorthUiQueryBindingPlan,
    application_facts: &super::super::UiIntentApplicationFactPlan,
) -> Result<UiIntentCatalog, UiIntentCatalogPreparationDenial> {
    let (declarations, declaration_index) =
        resolve_declarations(material, definitions, query, application_facts)?;
    let routes =
        route_preparation::bind_routes(material, &declarations, &declaration_index, graph)?;
    let product_index = routes
        .product
        .iter()
        .enumerate()
        .map(|(index, route)| ((route.graph_node(), route.interaction()), index))
        .collect();
    let mut command_index =
        std::collections::HashMap::<crate::capability::UiIntentId, Vec<u32>>::new();
    for (index, declaration) in declarations.iter().enumerate() {
        let intent = definitions.definition_at(declaration.definition()).id();
        command_index.entry(intent).or_default().push(index as u32);
    }
    let command_index = command_index
        .into_iter()
        .map(|(intent, indexes)| (intent, indexes.into_boxed_slice()))
        .collect();
    let confirmation_index = routes
        .confirmation
        .iter()
        .enumerate()
        .map(|(index, route)| {
            (
                (route.graph_node(), UiSemanticInteractionFamily::Activate),
                index,
            )
        })
        .collect();
    Ok(UiIntentCatalog {
        declarations: declarations.into_boxed_slice(),
        product_routes: routes.product.into_boxed_slice(),
        confirmation_routes: routes.confirmation.into_boxed_slice(),
        product_index,
        command_index,
        confirmation_index,
        definition_count: definitions.len(),
    })
}

fn resolve_declarations(
    material: &crate::declaration::WorthUiAuthoredIntentMaterial,
    definitions: &FrozenIntentDefinitionCapabilities,
    query: &worth_ui_query_binding::WorthUiQueryBindingPlan,
    application_facts: &super::super::UiIntentApplicationFactPlan,
) -> Result<
    (
        Vec<Arc<UiCanonicalIntentDeclaration>>,
        BTreeMap<Box<str>, u32>,
    ),
    UiIntentCatalogPreparationDenial,
> {
    if material.declarations().len() > MAXIMUM_INTENT_DECLARATIONS {
        return Err(UiIntentCatalogPreparationDenial::TooManyDeclarations {
            observed: material.declarations().len(),
            maximum: MAXIMUM_INTENT_DECLARATIONS,
        });
    }
    let mut declarations = Vec::with_capacity(material.declarations().len());
    let mut indexes = BTreeMap::new();
    for authored in material.declarations() {
        let identity: Box<str> = authored.identity().into();
        let index = declarations.len() as u32;
        if indexes.insert(identity.clone(), index).is_some() {
            return Err(UiIntentCatalogPreparationDenial::DuplicateDeclaration { identity });
        }
        let resolved = definitions
            .resolve_stable_text(authored.definition_reference())
            .ok_or_else(|| UiIntentCatalogPreparationDenial::UnknownDefinition {
                declaration: identity.clone(),
                definition: authored.definition_reference().into(),
            })?;
        validate_schemas(authored, resolved.descriptor())?;
        let interaction = runtime_family(authored.interaction());
        if !resolved
            .descriptor()
            .accepted_interactions()
            .contains(&interaction)
        {
            return Err(UiIntentCatalogPreparationDenial::InteractionNotAccepted {
                declaration: identity,
                interaction,
            });
        }
        declarations.push(Arc::new(UiCanonicalIntentDeclaration::new(
            UiIntentDeclarationIdentity::new(authored.identity()),
            resolved.slot(),
            interaction,
            super::super::resolve_payload_sources(
                authored,
                resolved.descriptor().payload_fields(),
                query,
                application_facts,
            )?,
            super::super::resolve_operability_contract(
                authored.identity(),
                authored.operability(),
                interaction,
                query,
                application_facts,
            )?,
            super::super::resolve_confirmation_contract(
                authored.identity(),
                authored.confirmation(),
                application_facts,
            )?,
            super::super::UiIntentConcurrencyScope::from_dsl(authored.concurrency()),
            super::super::resolve_consequence_contract(
                authored.identity(),
                authored.consequences(),
                resolved.descriptor(),
                query,
            )?,
        )));
    }
    Ok((declarations, indexes))
}

fn validate_schemas(
    authored: &crate::declaration::WorthUiAuthoredIntentDeclaration,
    definition: &crate::capability::IntentDefinitionDescriptor,
) -> Result<(), UiIntentCatalogPreparationDenial> {
    if let Some(expected) = authored.expected_payload_schema() {
        let registered = definition.payload_schema();
        if expected.identity() != registered.stable_identity()
            || expected.version() != registered.version()
        {
            return Err(UiIntentCatalogPreparationDenial::PayloadSchemaMismatch {
                declaration: authored.identity().into(),
                expected_identity: expected.identity().into(),
                expected_version: expected.version(),
                registered,
            });
        }
    }
    if let Some(expected) = authored.expected_outcome_schema() {
        let registered = definition.product_outcome_schema();
        if expected.identity() != registered.stable_identity()
            || expected.version() != registered.version()
        {
            return Err(UiIntentCatalogPreparationDenial::OutcomeSchemaMismatch {
                declaration: authored.identity().into(),
                expected_identity: expected.identity().into(),
                expected_version: expected.version(),
                registered,
            });
        }
    }
    Ok(())
}

fn bind_routes(
    material: &crate::declaration::WorthUiAuthoredIntentMaterial,
    declarations: &[Arc<UiCanonicalIntentDeclaration>],
    declaration_index: &BTreeMap<Box<str>, u32>,
    graph: &crate::graph::UiGraphSnapshot,
) -> Result<ResolvedIntentRoutes, UiIntentCatalogPreparationDenial> {
    route_preparation::bind_routes(material, declarations, declaration_index, graph)
}

fn runtime_family(
    family: worth_ui_dsl::WorthUiIntentInteractionFamily,
) -> UiSemanticInteractionFamily {
    match family {
        worth_ui_dsl::WorthUiIntentInteractionFamily::Activate => {
            UiSemanticInteractionFamily::Activate
        }
        worth_ui_dsl::WorthUiIntentInteractionFamily::EditCommit => {
            UiSemanticInteractionFamily::EditCommit
        }
        worth_ui_dsl::WorthUiIntentInteractionFamily::SelectionCommit => {
            UiSemanticInteractionFamily::SelectionCommit
        }
        worth_ui_dsl::WorthUiIntentInteractionFamily::Submit => UiSemanticInteractionFamily::Submit,
    }
}
