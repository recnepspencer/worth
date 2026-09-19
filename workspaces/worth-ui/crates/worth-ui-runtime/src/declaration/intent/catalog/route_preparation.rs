use std::collections::BTreeMap;
use std::sync::Arc;

use crate::capability::UiSemanticInteractionFamily;
use crate::declaration::intent::{
    UiCanonicalIntentDeclaration, UiIntentCatalogPreparationDenial,
    UiIntentConfirmationRouteBinding, UiIntentRouteBinding,
};

use super::super::RouteKey;
use super::ResolvedIntentRoutes;

struct IntentRouteCatalogBuilder<'a> {
    declarations: &'a [Arc<UiCanonicalIntentDeclaration>],
    declaration_index: &'a BTreeMap<Box<str>, u32>,
    graph: &'a crate::graph::UiGraphSnapshot,
    product: Vec<UiIntentRouteBinding>,
    confirmation: Vec<UiIntentConfirmationRouteBinding>,
    product_keys: BTreeMap<RouteKey, usize>,
    confirmation_keys: BTreeMap<RouteKey, usize>,
}

pub(super) fn bind_routes(
    material: &crate::declaration::WorthUiAuthoredIntentMaterial,
    declarations: &[Arc<UiCanonicalIntentDeclaration>],
    declaration_index: &BTreeMap<Box<str>, u32>,
    graph: &crate::graph::UiGraphSnapshot,
) -> Result<ResolvedIntentRoutes, UiIntentCatalogPreparationDenial> {
    let mut builder = IntentRouteCatalogBuilder::new(declarations, declaration_index, graph);
    for authored in material.routes() {
        builder.bind_authored_route(authored)?;
    }
    Ok(builder.finish())
}

impl<'a> IntentRouteCatalogBuilder<'a> {
    fn new(
        declarations: &'a [Arc<UiCanonicalIntentDeclaration>],
        declaration_index: &'a BTreeMap<Box<str>, u32>,
        graph: &'a crate::graph::UiGraphSnapshot,
    ) -> Self {
        Self {
            declarations,
            declaration_index,
            graph,
            product: Vec::new(),
            confirmation: Vec::new(),
            product_keys: BTreeMap::new(),
            confirmation_keys: BTreeMap::new(),
        }
    }

    fn bind_authored_route(
        &mut self,
        authored: &crate::declaration::WorthUiAuthoredIntentRoute,
    ) -> Result<(), UiIntentCatalogPreparationDenial> {
        let reference = authored.route().declaration_identity();
        let declaration_index =
            self.declaration_index
                .get(reference)
                .copied()
                .ok_or_else(
                    || UiIntentCatalogPreparationDenial::UnknownRouteDeclaration {
                        declaration: reference.into(),
                    },
                )?;
        let declaration = &self.declarations[declaration_index as usize];
        let family = super::runtime_family(authored.route().family());
        validate_route_family(declaration, authored.route().kind(), family)?;
        let portal_declaration = authored.portal_declaration();
        let targets = self
            .graph
            .graph_node_ids_for_authored_provenance(authored.target_provenance_digest());
        if targets.is_empty() {
            return Err(UiIntentCatalogPreparationDenial::MissingRouteTarget {
                authored_provenance_digest: authored.target_provenance_digest(),
            });
        }
        for target in targets {
            self.bind_target(
                *target,
                declaration_index,
                family,
                authored.route().kind(),
                portal_declaration,
            )?;
        }
        Ok(())
    }

    fn bind_target(
        &mut self,
        target: crate::graph::UiGraphNodeIdentity,
        declaration_index: u32,
        family: UiSemanticInteractionFamily,
        kind: worth_ui_dsl::WorthUiIntentInteractionRouteKind,
        portal_declaration: Option<worth_ui_dsl::UiPortalDeclarationId>,
    ) -> Result<(), UiIntentCatalogPreparationDenial> {
        let key = (target, family);
        deny_route_collision(kind, key, &self.product_keys, &self.confirmation_keys)?;
        match kind {
            worth_ui_dsl::WorthUiIntentInteractionRouteKind::Product => {
                self.product_keys.insert(key, self.product.len());
                self.product.push(UiIntentRouteBinding::new(
                    target,
                    declaration_index,
                    family,
                    portal_declaration,
                ));
            }
            worth_ui_dsl::WorthUiIntentInteractionRouteKind::Confirmation => {
                self.confirmation_keys.insert(key, self.confirmation.len());
                self.confirmation
                    .push(UiIntentConfirmationRouteBinding::new(
                        target,
                        declaration_index,
                    ));
            }
        }
        self.enforce_capacity()
    }

    fn enforce_capacity(&self) -> Result<(), UiIntentCatalogPreparationDenial> {
        let observed = self.product.len() + self.confirmation.len();
        if observed > super::MAXIMUM_INTENT_ROUTES {
            Err(UiIntentCatalogPreparationDenial::RouteCapacityExceeded {
                observed,
                maximum: super::MAXIMUM_INTENT_ROUTES,
            })
        } else {
            Ok(())
        }
    }

    fn finish(mut self) -> ResolvedIntentRoutes {
        self.product
            .sort_by_key(|route| (route.graph_node(), route.interaction()));
        self.confirmation.sort_by_key(|route| route.graph_node());
        ResolvedIntentRoutes {
            product: self.product,
            confirmation: self.confirmation,
        }
    }
}

fn validate_route_family(
    declaration: &UiCanonicalIntentDeclaration,
    kind: worth_ui_dsl::WorthUiIntentInteractionRouteKind,
    family: UiSemanticInteractionFamily,
) -> Result<(), UiIntentCatalogPreparationDenial> {
    match kind {
        worth_ui_dsl::WorthUiIntentInteractionRouteKind::Product
            if declaration.interaction() != family =>
        {
            Err(
                UiIntentCatalogPreparationDenial::ProductInteractionMismatch {
                    declaration: declaration.identity().as_str().into(),
                    declared: declaration.interaction(),
                    routed: family,
                },
            )
        }
        worth_ui_dsl::WorthUiIntentInteractionRouteKind::Confirmation
            if family != UiSemanticInteractionFamily::Activate =>
        {
            Err(
                UiIntentCatalogPreparationDenial::ConfirmationRequiresActivate {
                    declaration: declaration.identity().as_str().into(),
                    routed: family,
                },
            )
        }
        _ => Ok(()),
    }
}

fn deny_route_collision(
    kind: worth_ui_dsl::WorthUiIntentInteractionRouteKind,
    key: RouteKey,
    product_keys: &BTreeMap<RouteKey, usize>,
    confirmation_keys: &BTreeMap<RouteKey, usize>,
) -> Result<(), UiIntentCatalogPreparationDenial> {
    let denial = match kind {
        worth_ui_dsl::WorthUiIntentInteractionRouteKind::Product => {
            if confirmation_keys.contains_key(&key) {
                Some(UiIntentCatalogPreparationDenial::RouteKindCrossover {
                    graph_node: key.0,
                    interaction: key.1,
                })
            } else if product_keys.contains_key(&key) {
                Some(UiIntentCatalogPreparationDenial::AmbiguousProductRoute {
                    graph_node: key.0,
                    interaction: key.1,
                })
            } else {
                None
            }
        }
        worth_ui_dsl::WorthUiIntentInteractionRouteKind::Confirmation => {
            if product_keys.contains_key(&key) {
                Some(UiIntentCatalogPreparationDenial::RouteKindCrossover {
                    graph_node: key.0,
                    interaction: key.1,
                })
            } else if confirmation_keys.contains_key(&key) {
                Some(
                    UiIntentCatalogPreparationDenial::AmbiguousConfirmationRoute {
                        graph_node: key.0,
                        interaction: key.1,
                    },
                )
            } else {
                None
            }
        }
    };
    match denial {
        Some(denial) => Err(denial),
        None => Ok(()),
    }
}
