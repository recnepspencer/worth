use std::collections::BTreeMap;

use crate::{
    CompatHttpSurfaceRoot, WorthServerCompatHttpRouteFamily, WorthServerCompatibilityFacade,
    WorthServerOperationFamily, WorthServerOperationInventory, WorthServerOperationRegistry,
    WorthServerProductAdapterRegistry, WorthServerProductReadTransport,
};

use super::{
    assembly_error::WorthServerRouteAssemblyError,
    declared_route::WorthServerDeclaredRoute,
    execution_bridge::WorthServerRouteExecutionBridge,
    inventory::{WorthServerRouteInventory, WorthServerRouteInventoryRow},
    operational_route::{WorthServerOperationalRoute, WorthServerOperationalRouteKind},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthServerRouteAssembly {
    declared_routes: Vec<WorthServerDeclaredRoute>,
    operational_routes: Vec<WorthServerOperationalRoute>,
    inventory: WorthServerRouteInventory,
}

impl WorthServerRouteAssembly {
    pub(crate) fn assemble(
        compat_http_root: &CompatHttpSurfaceRoot,
        operation_registry: &WorthServerOperationRegistry,
        product_adapter_registry: &WorthServerProductAdapterRegistry,
    ) -> Result<Self, WorthServerRouteAssemblyError> {
        let mut declared_routes = Vec::new();
        let mut route_keys = BTreeMap::new();
        if compat_http_root.capabilities().is_registered()
            && !compat_http_root.capabilities().is_disabled()
        {
            append_declared_product_routes(
                &mut declared_routes,
                &mut route_keys,
                compat_http_root,
                operation_registry,
                product_adapter_registry,
            )?;
            append_session_coordination_routes(
                &mut declared_routes,
                &mut route_keys,
                compat_http_root,
                operation_registry,
            )?;
        }
        let operational_routes = operational_routes();
        let inventory = WorthServerRouteInventory::new(
            declared_routes
                .iter()
                .map(|route| {
                    WorthServerRouteInventoryRow::semantic(
                        super::WorthServerSemanticRouteInventoryRowParts {
                            method: route.method().to_string(),
                            path: route.path().to_string(),
                            operation_family: route.operation_family(),
                            operation_name: route.operation_name().to_string(),
                            payload_schema_identity: route.payload_schema_identity().to_string(),
                            result_contract_digest: route
                                .result_contract_digest()
                                .map(str::to_string),
                            durability_contract_digest: route
                                .durability_contract_digest()
                                .map(str::to_string),
                            support_row: route.support_row().to_string(),
                            diagnostics_policy: route.diagnostics_policy().to_string(),
                            response_transform: route.response_transform(),
                            evidence_policy: route.evidence_policy().to_string(),
                        },
                    )
                })
                .chain(operational_routes.iter().map(|route| {
                    WorthServerRouteInventoryRow::operational(
                        route.method(),
                        route.path(),
                        format!("{:?}", route.kind()),
                    )
                }))
                .collect(),
        );
        Ok(Self {
            declared_routes,
            operational_routes,
            inventory,
        })
    }

    pub fn declared_routes(&self) -> &[WorthServerDeclaredRoute] {
        &self.declared_routes
    }

    pub fn operational_routes(&self) -> &[WorthServerOperationalRoute] {
        &self.operational_routes
    }

    pub fn inventory(&self) -> &WorthServerRouteInventory {
        &self.inventory
    }
}

#[derive(Clone, Debug)]
pub struct WorthServerOperationRouter {
    instance_selection: crate::WorthServerInstanceSelectionPolicy,
    instance: crate::WorthServerInstanceIdentity,
    assembly: WorthServerRouteAssembly,
    compat_http: WorthServerCompatibilityFacade,
    caller_admission: crate::transport::WorthServerTransportCallerAdmission,
}

impl WorthServerOperationRouter {
    pub(crate) fn new(
        instance: crate::WorthServerInstanceIdentity,
        instance_selection: crate::WorthServerInstanceSelectionPolicy,
        assembly: WorthServerRouteAssembly,
        compat_http: WorthServerCompatibilityFacade,
        caller_admission: crate::transport::WorthServerTransportCallerAdmission,
    ) -> Self {
        Self {
            instance_selection,
            instance,
            assembly,
            compat_http,
            caller_admission,
        }
    }

    pub fn inventory(&self) -> &WorthServerRouteInventory {
        self.assembly.inventory()
    }

    pub fn bridge_for(&self, method: &str, path: &str) -> Option<WorthServerRouteExecutionBridge> {
        self.assembly
            .declared_routes()
            .iter()
            .find(|route| route.method() == method && route.path() == path)
            .cloned()
            .map(|route| {
                WorthServerRouteExecutionBridge::semantic(
                    route,
                    self.compat_http.clone(),
                    self.instance.clone(),
                    self.instance_selection,
                )
            })
            .or_else(|| {
                self.assembly
                    .operational_routes()
                    .iter()
                    .find(|route| route.method() == method && route.path() == path)
                    .cloned()
                    .map(|route| {
                        WorthServerRouteExecutionBridge::operational(
                            route,
                            self.compat_http.clone(),
                            self.instance.clone(),
                        )
                    })
            })
    }

    pub(crate) fn caller_admission(
        &self,
    ) -> &crate::transport::WorthServerTransportCallerAdmission {
        &self.caller_admission
    }
}

fn append_declared_product_routes(
    declared_routes: &mut Vec<WorthServerDeclaredRoute>,
    route_keys: &mut BTreeMap<(String, String), ()>,
    compat_http_root: &CompatHttpSurfaceRoot,
    operation_registry: &WorthServerOperationRegistry,
    product_adapter_registry: &WorthServerProductAdapterRegistry,
) -> Result<(), WorthServerRouteAssemblyError> {
    for declaration in product_adapter_registry.declarations() {
        let route = match declaration.operation_family() {
            WorthServerOperationFamily::ProductApplicationRead => {
                let (method, path, route_family) = match declaration.read_transport() {
                    Some(WorthServerProductReadTransport::FlatQuery) => (
                        "GET",
                        format!("/compat/reads/{}", declaration.operation_name()),
                        WorthServerCompatHttpRouteFamily::Read,
                    ),
                    Some(WorthServerProductReadTransport::StructuredQuery) => (
                        "POST",
                        format!("/compat/queries/{}", declaration.operation_name()),
                        WorthServerCompatHttpRouteFamily::Query,
                    ),
                    None => {
                        return Err(WorthServerRouteAssemblyError::OperationNameNotAdmitted {
                            family: declaration.operation_family(),
                            operation_name: declaration.operation_name().to_string(),
                        })
                    }
                };
                require_route_family(compat_http_root, route_family, declaration.operation_name())?;
                WorthServerDeclaredRoute::new(
                    method,
                    path,
                    route_family,
                    declaration.operation_family(),
                    declaration.operation_name(),
                    declaration.payload_schema_identity(),
                    declaration.support_snapshot().support_row(),
                )
                .with_product_contracts(
                    declaration.result_contract().canonical_digest(),
                    declaration
                        .durable_mutation_contract()
                        .map(crate::WorthServerDurableProductMutationContract::canonical_digest),
                )
            }
            WorthServerOperationFamily::ProductApplicationMutation => {
                require_route_family(
                    compat_http_root,
                    WorthServerCompatHttpRouteFamily::Mutation,
                    declaration.operation_name(),
                )?;
                WorthServerDeclaredRoute::new(
                    "POST",
                    format!("/compat/mutations/{}", declaration.operation_name()),
                    WorthServerCompatHttpRouteFamily::Mutation,
                    declaration.operation_family(),
                    declaration.operation_name(),
                    declaration.payload_schema_identity(),
                    declaration.support_snapshot().support_row(),
                )
                .with_product_contracts(
                    declaration.result_contract().canonical_digest(),
                    declaration
                        .durable_mutation_contract()
                        .map(crate::WorthServerDurableProductMutationContract::canonical_digest),
                )
            }
            WorthServerOperationFamily::ProductSessionCoordination => continue,
            _ => continue,
        };
        require_unique_route(route_keys, route.method(), route.path())?;
        operation_registry
            .admit_operation_name(route.operation_family(), route.operation_name())
            .map_err(
                |_| WorthServerRouteAssemblyError::OperationNameNotAdmitted {
                    family: route.operation_family(),
                    operation_name: route.operation_name().to_string(),
                },
            )?;
        declared_routes.push(route);
    }
    Ok(())
}

fn append_session_coordination_routes(
    declared_routes: &mut Vec<WorthServerDeclaredRoute>,
    route_keys: &mut BTreeMap<(String, String), ()>,
    compat_http_root: &CompatHttpSurfaceRoot,
    operation_registry: &WorthServerOperationRegistry,
) -> Result<(), WorthServerRouteAssemblyError> {
    let operation_inventory: WorthServerOperationInventory = operation_registry.inventory();
    let Some(coordination_row) = operation_inventory.rows().iter().find(|row| {
        row.family() == WorthServerOperationFamily::ProductSessionCoordination && row.enabled()
    }) else {
        return Ok(());
    };
    if !coordination_row
        .exposed_surfaces()
        .contains(&crate::WorthServerSurfaceFamily::CompatHttp)
    {
        return Ok(());
    }
    require_route_family(
        compat_http_root,
        WorthServerCompatHttpRouteFamily::Mutation,
        "product_session.open_mutation",
    )?;
    for protocol in crate::product_session_coordination::product_session_protocol_declarations() {
        let operation_name = protocol.operation_name();
        let route = WorthServerDeclaredRoute::new(
            "POST",
            format!("/compat/mutations/{operation_name}"),
            WorthServerCompatHttpRouteFamily::Mutation,
            WorthServerOperationFamily::ProductSessionCoordination,
            operation_name,
            protocol.request_schema_identity(),
            "product-session",
        );
        require_unique_route(route_keys, route.method(), route.path())?;
        operation_registry
            .admit_operation_name(route.operation_family(), route.operation_name())
            .map_err(
                |_| WorthServerRouteAssemblyError::OperationNameNotAdmitted {
                    family: route.operation_family(),
                    operation_name: route.operation_name().to_string(),
                },
            )?;
        declared_routes.push(route);
    }
    Ok(())
}

fn require_route_family(
    compat_http_root: &CompatHttpSurfaceRoot,
    route_family: WorthServerCompatHttpRouteFamily,
    operation_name: &str,
) -> Result<(), WorthServerRouteAssemblyError> {
    if compat_http_root.route_families().contains(route_family) {
        return Ok(());
    }
    match route_family {
        WorthServerCompatHttpRouteFamily::Read => Err(
            WorthServerRouteAssemblyError::MissingCompatReadRouteFamily {
                operation_name: operation_name.to_string(),
            },
        ),
        WorthServerCompatHttpRouteFamily::Query => Err(
            WorthServerRouteAssemblyError::MissingCompatQueryRouteFamily {
                operation_name: operation_name.to_string(),
            },
        ),
        WorthServerCompatHttpRouteFamily::Mutation => Err(
            WorthServerRouteAssemblyError::MissingCompatMutationRouteFamily {
                operation_name: operation_name.to_string(),
            },
        ),
        _ => Ok(()),
    }
}

fn require_unique_route(
    route_keys: &mut BTreeMap<(String, String), ()>,
    method: &str,
    path: &str,
) -> Result<(), WorthServerRouteAssemblyError> {
    let key = (method.to_string(), path.to_string());
    if route_keys.insert(key.clone(), ()).is_some() {
        return Err(WorthServerRouteAssemblyError::DuplicateMethodPath {
            method: key.0,
            path: key.1,
        });
    }
    Ok(())
}

fn operational_routes() -> Vec<WorthServerOperationalRoute> {
    vec![
        WorthServerOperationalRoute::new(
            WorthServerOperationalRouteKind::Health,
            "GET",
            "/healthz",
        ),
        WorthServerOperationalRoute::new(
            WorthServerOperationalRouteKind::Metrics,
            "GET",
            "/metrics",
        ),
        WorthServerOperationalRoute::new(
            WorthServerOperationalRouteKind::Preflight,
            "OPTIONS",
            "/compat/preflight",
        ),
        WorthServerOperationalRoute::new(
            WorthServerOperationalRouteKind::DocsExport,
            "GET",
            "/openapi.json",
        ),
    ]
}
