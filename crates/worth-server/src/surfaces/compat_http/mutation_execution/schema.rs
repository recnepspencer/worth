use std::sync::Arc;

use serde_json::Value;
use worth_query::facade::foundation::{
    QueryExternalIdentityToken, RelationalBridgeRecordIdentityParts, WorthQueryEntityIdentity,
};
use worth_query::facade::runtime::{
    WorthQueryAspectMutationBuilder, WorthQueryAspectTouch, WorthQueryAuthoredAspectValue,
    WorthQueryDeleteMutationBuilder, WorthQueryWriteCommand,
};

use crate::{
    WorthServerQueryHandoffDenial, WorthServerQueryHandoffDenialCode, WorthServerQueryOperation,
};

use super::request::{
    WorthServerCompatibilityMutationCommand, WorthServerCompatibilityMutationRequest,
};

pub(crate) fn lower_query_operation(
    operation_name: &str,
    request: &WorthServerCompatibilityMutationRequest,
    diagnostics_profile: worth_foundational::facade::DiagnosticRichnessProfile,
) -> Result<WorthServerQueryOperation, WorthServerQueryHandoffDenial> {
    let mut commands = Vec::with_capacity(request.commands().len());
    for command in request.commands() {
        commands.push(lower_command(command, diagnostics_profile)?);
    }
    Ok(if request.is_batch() {
        WorthServerQueryOperation::batch_mutation(operation_name, commands)
    } else {
        WorthServerQueryOperation::single_mutation(
            operation_name,
            commands
                .into_iter()
                .next()
                .expect("single mutation request should produce one command"),
        )
    })
}

fn lower_command(
    command: &WorthServerCompatibilityMutationCommand,
    diagnostics_profile: worth_foundational::facade::DiagnosticRichnessProfile,
) -> Result<WorthQueryWriteCommand, WorthServerQueryHandoffDenial> {
    match command {
        WorthServerCompatibilityMutationCommand::Insert {
            collection,
            aspects,
            metadata,
        } => {
            let mut builder = WorthQueryAspectMutationBuilder::new();
            for (name, value) in aspects {
                builder = builder.set_aspect(
                    lower_aspect_touch(name, diagnostics_profile)?,
                    lower_authored_aspect_value(value, diagnostics_profile)?,
                );
            }
            for (name, value) in metadata {
                builder = builder.metadata(
                    name,
                    lower_metadata_value(value, diagnostics_profile)?,
                );
            }
            builder.build_insert(collection).map_err(|error| {
                WorthServerQueryHandoffDenial::new(
                    WorthServerQueryHandoffDenialCode::CompatibilityMutationRequestInvalid,
                    diagnostics_profile,
                    error.to_string(),
                )
            })
        }
        WorthServerCompatibilityMutationCommand::Update {
            entity_identity,
            aspects,
            metadata,
        } => {
            let mut builder = WorthQueryAspectMutationBuilder::new();
            for (name, value) in aspects {
                builder = builder.set_aspect(
                    lower_aspect_touch(name, diagnostics_profile)?,
                    lower_authored_aspect_value(value, diagnostics_profile)?,
                );
            }
            for (name, value) in metadata {
                builder = builder.metadata(
                    name,
                    lower_metadata_value(value, diagnostics_profile)?,
                );
            }
            builder
                .build_update(admit_compat_entity_identity(entity_identity).map_err(|cause| WorthServerQueryHandoffDenial::new(WorthServerQueryHandoffDenialCode::EvidenceIdentityDerivationDenied(cause), diagnostics_profile, "entity identity exceeded canonical digest bounds"))?)
                .map_err(|error| {
                    WorthServerQueryHandoffDenial::new(
                        WorthServerQueryHandoffDenialCode::CompatibilityMutationRequestInvalid,
                        diagnostics_profile,
                        error.to_string(),
                    )
                })
        }
        WorthServerCompatibilityMutationCommand::Delete {
            entity_identity,
            declared_collection,
            touched_aspect_paths,
            metadata,
        } => {
            let mut builder = WorthQueryDeleteMutationBuilder::new();
            if let Some(collection) = declared_collection {
                builder = builder.target_collection(collection).map_err(|cause| WorthServerQueryHandoffDenial::new(WorthServerQueryHandoffDenialCode::EvidenceIdentityDerivationDenied(cause), diagnostics_profile, "target collection identity exceeded canonical digest bounds"))?;
            }
            let touches = touched_aspect_paths
                .iter()
                .map(|path| lower_aspect_touch(path, diagnostics_profile))
                .collect::<Result<Vec<_>, _>>()?;
            builder = builder.touches(touches);
            for (name, value) in metadata {
                builder = builder.metadata(
                    name,
                    lower_metadata_value(value, diagnostics_profile)?,
                );
            }
            builder
                .build_delete(admit_compat_entity_identity(entity_identity).map_err(|cause| WorthServerQueryHandoffDenial::new(WorthServerQueryHandoffDenialCode::EvidenceIdentityDerivationDenied(cause), diagnostics_profile, "entity identity exceeded canonical digest bounds"))?)
                .map_err(|error| {
                    WorthServerQueryHandoffDenial::new(
                        WorthServerQueryHandoffDenialCode::CompatibilityMutationRequestInvalid,
                        diagnostics_profile,
                        error.to_string(),
                    )
                })
        }
        WorthServerCompatibilityMutationCommand::VerifyExisting { .. } => {
            Err(WorthServerQueryHandoffDenial::new(
                WorthServerQueryHandoffDenialCode::CompatibilityMutationFamilyForbidden,
                diagnostics_profile,
                "compatibility mutation family `verify_existing` is forbidden at the external server boundary",
            ))
        }
    }
}

fn admit_compat_entity_identity(
    entity_identity: &str,
) -> Result<
    WorthQueryEntityIdentity,
    worth_query::facade::runtime::WorthQueryEvidenceIdentityDerivationDenial,
> {
    RelationalBridgeRecordIdentityParts::from_bridge_entity_identity(entity_identity)
        .map(WorthQueryEntityIdentity::from_bridge_record_projection)
        .map(Ok)
        .unwrap_or_else(|| {
            WorthQueryEntityIdentity::admit_authored_entity_token(QueryExternalIdentityToken::new(
                Arc::from(entity_identity),
            ))
        })
}

fn lower_aspect_touch(
    touch: &str,
    diagnostics_profile: worth_foundational::facade::DiagnosticRichnessProfile,
) -> Result<WorthQueryAspectTouch, WorthServerQueryHandoffDenial> {
    WorthQueryAspectTouch::from_authoring_ingress_text(touch).map_err(|error| {
        WorthServerQueryHandoffDenial::new(
            WorthServerQueryHandoffDenialCode::CompatibilityMutationRequestInvalid,
            diagnostics_profile,
            format!("compatibility mutation touch `{touch}` is invalid: {error}"),
        )
    })
}

fn lower_authored_aspect_value(
    value: &Value,
    diagnostics_profile: worth_foundational::facade::DiagnosticRichnessProfile,
) -> Result<WorthQueryAuthoredAspectValue, WorthServerQueryHandoffDenial> {
    match value {
        Value::Null => Ok(WorthQueryAuthoredAspectValue::null()),
        Value::Bool(value) => Ok(WorthQueryAuthoredAspectValue::bool(*value)),
        Value::Number(value) => {
            let Some(int) = value.as_i64() else {
                return Err(WorthServerQueryHandoffDenial::new(
                    WorthServerQueryHandoffDenialCode::CompatibilityMutationRequestInvalid,
                    diagnostics_profile,
                    "compatibility mutation numeric aspect values must fit the Query authored int64 surface",
                ));
            };
            Ok(WorthQueryAuthoredAspectValue::int64(int))
        }
        Value::String(value) => Ok(WorthQueryAuthoredAspectValue::string(value)),
        Value::Array(_) | Value::Object(_) => Err(WorthServerQueryHandoffDenial::new(
            WorthServerQueryHandoffDenialCode::CompatibilityMutationRequestInvalid,
            diagnostics_profile,
            "compatibility mutation aspect values must be scalar JSON values admitted by the Query authored aspect surface",
        )),
    }
}

fn lower_metadata_value(
    value: &Value,
    diagnostics_profile: worth_foundational::facade::DiagnosticRichnessProfile,
) -> Result<String, WorthServerQueryHandoffDenial> {
    match value {
        Value::Null => Ok("null".to_string()),
        Value::Bool(value) => Ok(value.to_string()),
        Value::Number(value) => Ok(value.to_string()),
        Value::String(value) => Ok(value.clone()),
        Value::Array(_) | Value::Object(_) => Err(WorthServerQueryHandoffDenial::new(
            WorthServerQueryHandoffDenialCode::CompatibilityMutationRequestInvalid,
            diagnostics_profile,
            "compatibility mutation metadata values must lower to scalar strings",
        )),
    }
}
