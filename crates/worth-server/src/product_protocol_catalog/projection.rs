use std::collections::BTreeMap;

use crate::{
    product_adapter::WorthServerProductAdapterRegistry, WorthServerOperationFamily,
    WorthServerProductOperationAuthorityRequirement, WorthServerProductOperationDeclaration,
    WorthServerRouteInventory, WorthServerRouteInventoryRow,
};

use super::{
    catalog::{WorthServerProductProtocolCatalog, WorthServerProductProtocolCatalogError},
    envelope_contract::{envelope_schema, envelope_schema_digest, ENVELOPE_SCHEMA_IDENTITY},
    product_operation::{
        WorthServerProductOperationProtocol, WorthServerProductOperationProtocolParts,
    },
    product_session_operation::WorthServerProductSessionOperationProtocol,
};

pub(crate) fn project_product_protocol_catalog(
    registry: &WorthServerProductAdapterRegistry,
    routes: &WorthServerRouteInventory,
    instance_selection: crate::WorthServerInstanceSelectionPolicy,
) -> Result<WorthServerProductProtocolCatalog, WorthServerProductProtocolCatalogError> {
    let route_index = product_route_index(routes);
    let operations = project_operation_protocols(registry, &route_index, instance_selection)?;
    let session_operations = project_session_operation_protocols(routes, instance_selection)?;
    Ok(assemble_catalog(operations, session_operations))
}

fn project_operation_protocols(
    registry: &WorthServerProductAdapterRegistry,
    route_index: &BTreeMap<&str, Vec<&WorthServerRouteInventoryRow>>,
    instance_selection: crate::WorthServerInstanceSelectionPolicy,
) -> Result<Vec<WorthServerProductOperationProtocol>, WorthServerProductProtocolCatalogError> {
    let mut operations = registry
        .declarations()
        .into_iter()
        .map(|declaration| project_operation_protocol(declaration, route_index, instance_selection))
        .collect::<Result<Vec<_>, _>>()?;
    operations.sort_by(|left, right| left.operation_name().cmp(right.operation_name()));
    Ok(operations)
}

fn project_operation_protocol(
    declaration: &WorthServerProductOperationDeclaration,
    route_index: &BTreeMap<&str, Vec<&WorthServerRouteInventoryRow>>,
    instance_selection: crate::WorthServerInstanceSelectionPolicy,
) -> Result<WorthServerProductOperationProtocol, WorthServerProductProtocolCatalogError> {
    let route = require_exact_route(declaration.operation_name(), route_index)?;
    let result = declaration.result_contract();
    Ok(WorthServerProductOperationProtocol::from_parts(
        WorthServerProductOperationProtocolParts {
            operation_name: declaration.operation_name().to_string(),
            operation_family: declaration.operation_family().as_str().to_string(),
            method: route.method().to_string(),
            route: route.path().to_string(),
            request_schema_identity: declaration.payload_schema_identity().to_string(),
            result_schema_identity: result.schema().identity().to_string(),
            result_schema_version: result.schema().version(),
            result_contract_digest: result.canonical_digest().to_string(),
            result_encoding: result.encoding().as_str().to_string(),
            result_canonicalization: result.canonicalization().as_str().to_string(),
            result_max_inline_bytes: result.max_inline_bytes(),
            basis_kind: declaration.basis_kind().as_str().to_string(),
            requires_product_session: requires_product_session(declaration),
            requires_idempotency_key: declaration.operation_family()
                == WorthServerOperationFamily::ProductApplicationMutation,
            requires_server_instance: instance_selection
                .requires_selection(declaration.operation_family(), declaration.operation_name()),
        },
    ))
}

fn require_exact_route<'a>(
    operation_name: &str,
    route_index: &'a BTreeMap<&str, Vec<&WorthServerRouteInventoryRow>>,
) -> Result<&'a WorthServerRouteInventoryRow, WorthServerProductProtocolCatalogError> {
    let matching_routes = route_index
        .get(operation_name)
        .map(Vec::as_slice)
        .unwrap_or_default();
    match matching_routes {
        [route] => Ok(*route),
        [] => Err(WorthServerProductProtocolCatalogError::MissingRoute {
            operation_name: operation_name.to_string(),
        }),
        _ => Err(WorthServerProductProtocolCatalogError::DuplicateRoute {
            operation_name: operation_name.to_string(),
        }),
    }
}

fn project_session_operation_protocols(
    routes: &WorthServerRouteInventory,
    instance_selection: crate::WorthServerInstanceSelectionPolicy,
) -> Result<Vec<WorthServerProductSessionOperationProtocol>, WorthServerProductProtocolCatalogError>
{
    let declarations = crate::product_session_coordination::product_session_protocol_declarations();
    let session_routes = routes
        .rows()
        .iter()
        .filter(|route| {
            route.operation_family() == Some(WorthServerOperationFamily::ProductSessionCoordination)
        })
        .collect::<Vec<_>>();
    reject_unexpected_session_routes(&session_routes, &declarations)?;
    let mut operations = declarations
        .iter()
        .map(|declaration| {
            project_session_operation_protocol(*declaration, &session_routes, instance_selection)
        })
        .collect::<Result<Vec<_>, _>>()?;
    operations.sort_by(|left, right| left.operation_name().cmp(right.operation_name()));
    Ok(operations)
}

fn reject_unexpected_session_routes(
    routes: &[&WorthServerRouteInventoryRow],
    declarations: &[crate::product_session_coordination::WorthServerProductSessionProtocolDeclaration],
) -> Result<(), WorthServerProductProtocolCatalogError> {
    for route in routes {
        let operation_name = route.operation_name().unwrap_or("<missing-operation-name>");
        if !declarations
            .iter()
            .any(|declaration| declaration.operation_name() == operation_name)
        {
            return Err(
                WorthServerProductProtocolCatalogError::UnexpectedSessionRoute {
                    operation_name: operation_name.to_string(),
                },
            );
        }
    }
    Ok(())
}

fn project_session_operation_protocol(
    declaration: crate::product_session_coordination::WorthServerProductSessionProtocolDeclaration,
    routes: &[&WorthServerRouteInventoryRow],
    instance_selection: crate::WorthServerInstanceSelectionPolicy,
) -> Result<WorthServerProductSessionOperationProtocol, WorthServerProductProtocolCatalogError> {
    let matching_routes = routes
        .iter()
        .filter(|route| route.operation_name() == Some(declaration.operation_name()))
        .copied()
        .collect::<Vec<_>>();
    let [route] = matching_routes.as_slice() else {
        return Err(if matching_routes.is_empty() {
            WorthServerProductProtocolCatalogError::MissingSessionRoute {
                operation_name: declaration.operation_name().to_string(),
            }
        } else {
            WorthServerProductProtocolCatalogError::DuplicateRoute {
                operation_name: declaration.operation_name().to_string(),
            }
        });
    };
    Ok(WorthServerProductSessionOperationProtocol::new(
        declaration.operation_name().to_string(),
        route.method().to_string(),
        route.path().to_string(),
        declaration.request_schema_identity().to_string(),
        declaration.response_schema_identity().to_string(),
        declaration.requires_product_session(),
        instance_selection.requires_selection(
            WorthServerOperationFamily::ProductSessionCoordination,
            declaration.operation_name(),
        ),
    ))
}

fn assemble_catalog(
    operations: Vec<WorthServerProductOperationProtocol>,
    session_operations: Vec<WorthServerProductSessionOperationProtocol>,
) -> WorthServerProductProtocolCatalog {
    let envelope_digest = envelope_schema_digest();
    let catalog_digest = catalog_digest(&envelope_digest, &operations, &session_operations);
    WorthServerProductProtocolCatalog::new(
        catalog_digest,
        ENVELOPE_SCHEMA_IDENTITY.to_string(),
        envelope_digest,
        envelope_schema(),
        operations,
        session_operations,
    )
}

fn product_route_index(
    routes: &WorthServerRouteInventory,
) -> BTreeMap<&str, Vec<&crate::WorthServerRouteInventoryRow>> {
    let mut index = BTreeMap::<_, Vec<_>>::new();
    for route in routes.rows() {
        if let Some(operation_name) = route.operation_name() {
            index.entry(operation_name).or_default().push(route);
        }
    }
    index
}

fn requires_product_session(declaration: &WorthServerProductOperationDeclaration) -> bool {
    declaration.basis_kind() == crate::WorthServerProductOperationBasisKind::ProductSessionDerived
        || matches!(
            declaration.authority_requirement(),
            WorthServerProductOperationAuthorityRequirement::DraftMutation { .. }
                | WorthServerProductOperationAuthorityRequirement::SessionCoordination { .. }
        )
}

fn catalog_digest(
    envelope_digest: &str,
    operations: &[WorthServerProductOperationProtocol],
    session_operations: &[WorthServerProductSessionOperationProtocol],
) -> String {
    let mut digest = crate::canonical_digest::WorthServerCanonicalDigestBuilder::new(
        "worth-server-product-protocol-catalog-v1",
    )
    .field(
        "schema_identity",
        "worth.server.product-protocol-catalog.v1",
    )
    .field("schema_version", "1")
    .field("envelope_schema_digest", envelope_digest);
    for operation in operations {
        digest = digest.field("operation", &serialized_protocol(operation));
    }
    for operation in session_operations {
        digest = digest.field("session_operation", &serialized_protocol(operation));
    }
    digest.finish()
}

fn serialized_protocol(protocol: &impl serde::Serialize) -> String {
    serde_json::to_string(protocol).expect("product protocol rows must serialize")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digest_changes_with_semantic_protocol_drift() {
        let base = protocol("operation.alpha", "result.alpha.v1");
        let changed = protocol("operation.alpha", "result.alpha.v2");
        let envelope_digest = envelope_schema_digest();

        assert_ne!(
            catalog_digest(&envelope_digest, &[base], &[]),
            catalog_digest(&envelope_digest, &[changed], &[]),
        );
    }

    #[test]
    fn digest_changes_with_session_protocol_drift() {
        let base = session_protocol("/compat/mutations/product_session.close");
        let changed = session_protocol("/compat/mutations/product_session.close-v2");
        let envelope_digest = envelope_schema_digest();

        assert_ne!(
            catalog_digest(&envelope_digest, &[], &[base]),
            catalog_digest(&envelope_digest, &[], &[changed]),
        );
    }

    #[test]
    fn envelope_schema_requires_exactly_one_semantic_outcome() {
        let schema = envelope_schema();
        assert_eq!(schema["oneOf"].as_array().map(Vec::len), Some(3));
        assert_eq!(
            schema["properties"]["envelope_kind"]["enum"]
                .as_array()
                .map(Vec::len),
            Some(3),
        );
    }

    fn protocol(operation_name: &str, result_schema: &str) -> WorthServerProductOperationProtocol {
        WorthServerProductOperationProtocol::from_parts(WorthServerProductOperationProtocolParts {
            operation_name: operation_name.to_string(),
            operation_family: "product-application-read".to_string(),
            method: "GET".to_string(),
            route: format!("/compat/reads/{operation_name}"),
            request_schema_identity: "request.v1".to_string(),
            result_schema_identity: result_schema.to_string(),
            result_schema_version: 1,
            result_contract_digest: format!("digest:{result_schema}"),
            result_encoding: "canonical-json".to_string(),
            result_canonicalization: "canonical-json-v1".to_string(),
            result_max_inline_bytes: 1024,
            basis_kind: "query-derived".to_string(),
            requires_product_session: false,
            requires_idempotency_key: false,
            requires_server_instance: false,
        })
    }

    fn session_protocol(route: &str) -> WorthServerProductSessionOperationProtocol {
        WorthServerProductSessionOperationProtocol::new(
            "product_session.close".to_string(),
            "POST".to_string(),
            route.to_string(),
            "worth-server-product-session.v1".to_string(),
            "worth-server-product-session-coordination.response.v1".to_string(),
            true,
            false,
        )
    }
}
