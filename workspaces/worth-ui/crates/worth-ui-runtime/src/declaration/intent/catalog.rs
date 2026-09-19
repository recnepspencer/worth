mod lookup_cost;
mod preparation;
mod semantic_comparison;

pub use lookup_cost::UiIntentRouteResolutionCost;

use std::collections::HashMap;
use std::sync::Arc;

use crate::capability::UiSemanticInteractionFamily;

use super::{
    UiCanonicalIntentDeclaration, UiIntentCatalogPreparationDenial,
    UiIntentConfirmationRouteBinding, UiIntentRouteBinding,
};

pub(super) type RouteKey = (
    crate::graph::UiGraphNodeIdentity,
    UiSemanticInteractionFamily,
);

pub(crate) enum UiIntentCatalogResolvedRoute {
    Product {
        route: UiIntentRouteBinding,
        declaration: Arc<UiCanonicalIntentDeclaration>,
    },
    Confirmation {
        route: UiIntentConfirmationRouteBinding,
        declaration: Arc<UiCanonicalIntentDeclaration>,
    },
}

pub(crate) enum UiIntentCatalogCommandRoute {
    Resolved {
        declaration: Arc<UiCanonicalIntentDeclaration>,
    },
    Ambiguous {
        candidates: usize,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiIntentSingleProductRouteDenial {
    Missing,
    Ambiguous { routes: usize },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiIntentCatalogMetrics {
    definitions: usize,
    declarations: usize,
    product_routes: usize,
    confirmation_routes: usize,
}

pub(crate) struct UiIntentCatalog {
    declarations: Box<[Arc<UiCanonicalIntentDeclaration>]>,
    product_routes: Box<[UiIntentRouteBinding]>,
    confirmation_routes: Box<[UiIntentConfirmationRouteBinding]>,
    product_index: HashMap<RouteKey, usize>,
    command_index: HashMap<crate::capability::UiIntentId, Box<[u32]>>,
    confirmation_index: HashMap<RouteKey, usize>,
    definition_count: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiIntentCatalogSemanticComparison {
    Equivalent,
    Different,
}

impl UiIntentCatalog {
    /// Installation/replacement demand; ordinary turns use the installed owner.
    pub(crate) fn has_activation_routes(&self) -> bool {
        self.product_routes
            .iter()
            .any(|route| route.interaction() == UiSemanticInteractionFamily::Activate)
            || self
                .confirmation_index
                .keys()
                .any(|(_, family)| *family == UiSemanticInteractionFamily::Activate)
    }

    /// A single declared product route is usable without inferring a family.
    /// Multi-route controls require an explicit selector before consumption.
    pub(crate) fn single_product_route_identity(
        &self,
        graph_node: crate::graph::UiGraphNodeIdentity,
    ) -> Result<&super::UiIntentDeclarationIdentity, UiIntentSingleProductRouteDenial> {
        let start = self
            .product_routes
            .partition_point(|route| route.graph_node() < graph_node);
        let end = self
            .product_routes
            .partition_point(|route| route.graph_node() <= graph_node);
        match &self.product_routes[start..end] {
            [] => Err(UiIntentSingleProductRouteDenial::Missing),
            [route] => Ok(self.declarations[route.declaration_index() as usize].identity()),
            routes => Err(UiIntentSingleProductRouteDenial::Ambiguous {
                routes: routes.len(),
            }),
        }
    }

    pub(crate) fn prepare(
        material: &crate::declaration::WorthUiAuthoredIntentMaterial,
        definitions: &crate::capability::FrozenIntentDefinitionCapabilities,
        graph: &crate::graph::UiGraphSnapshot,
        query: &worth_ui_query_binding::WorthUiQueryBindingPlan,
        application_facts: &super::UiIntentApplicationFactPlan,
    ) -> Result<Self, UiIntentCatalogPreparationDenial> {
        preparation::prepare(material, definitions, graph, query, application_facts)
    }

    pub(crate) fn lookup(
        &self,
        graph_node: crate::graph::UiGraphNodeIdentity,
        interaction: UiSemanticInteractionFamily,
    ) -> Option<(UiIntentCatalogResolvedRoute, UiIntentRouteResolutionCost)> {
        let key = (graph_node, interaction);
        if let Some(index) = self.product_index.get(&key).copied() {
            let route = self.product_routes[index];
            return Some((
                UiIntentCatalogResolvedRoute::Product {
                    route,
                    declaration: Arc::clone(&self.declarations[route.declaration_index() as usize]),
                },
                UiIntentRouteResolutionCost::product_route(),
            ));
        }
        self.confirmation_index.get(&key).copied().map(|index| {
            let route = self.confirmation_routes[index];
            (
                UiIntentCatalogResolvedRoute::Confirmation {
                    route,
                    declaration: Arc::clone(&self.declarations[route.declaration_index() as usize]),
                },
                UiIntentRouteResolutionCost::confirmation_route(),
            )
        })
    }

    pub(crate) fn lookup_command(
        &self,
        intent: crate::capability::UiIntentId,
    ) -> Option<(UiIntentCatalogCommandRoute, UiIntentRouteResolutionCost)> {
        let indexes = self.command_index.get(&intent)?;
        if indexes.len() != 1 {
            return Some((
                UiIntentCatalogCommandRoute::Ambiguous {
                    candidates: indexes.len(),
                },
                UiIntentRouteResolutionCost::command_route(indexes.len()),
            ));
        }
        let declaration_index = indexes[0];
        Some((
            UiIntentCatalogCommandRoute::Resolved {
                declaration: Arc::clone(&self.declarations[declaration_index as usize]),
            },
            UiIntentRouteResolutionCost::command_route(1),
        ))
    }

    pub(crate) fn metrics(&self) -> UiIntentCatalogMetrics {
        UiIntentCatalogMetrics {
            definitions: self.definition_count,
            declarations: self.declarations.len(),
            product_routes: self.product_routes.len(),
            confirmation_routes: self.confirmation_routes.len(),
        }
    }

    pub(crate) fn compare_semantic_contract(
        &self,
        candidate: &Self,
    ) -> UiIntentCatalogSemanticComparison {
        semantic_comparison::compare(self, candidate)
    }
}

impl UiIntentCatalogMetrics {
    pub const fn definitions(self) -> usize {
        self.definitions
    }

    pub const fn declarations(self) -> usize {
        self.declarations
    }

    pub const fn product_routes(self) -> usize {
        self.product_routes
    }

    pub const fn confirmation_routes(self) -> usize {
        self.confirmation_routes
    }
}
