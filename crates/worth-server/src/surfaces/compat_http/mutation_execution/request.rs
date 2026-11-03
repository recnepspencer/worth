use std::collections::BTreeMap;

use serde_json::Value;

use crate::{WorthServerQueryHandoffDenial, WorthServerQueryHandoffDenialCode};

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct WorthServerCompatibilityMutationRequest {
    commands: Vec<WorthServerCompatibilityMutationCommand>,
    batch: bool,
    canonical_digest: String,
}

impl WorthServerCompatibilityMutationRequest {
    pub(crate) fn parse(
        body: Value,
        diagnostics_profile: worth_foundational::facade::DiagnosticRichnessProfile,
    ) -> Result<Self, WorthServerQueryHandoffDenial> {
        let object = body.as_object().ok_or_else(|| {
            WorthServerQueryHandoffDenial::new(
                WorthServerQueryHandoffDenialCode::CompatibilityMutationRequestInvalid,
                diagnostics_profile,
                "compatibility mutation body must be a JSON object",
            )
        })?;
        let command = object.get("command");
        let commands = object.get("commands");
        match (command, commands) {
            (Some(_), Some(_)) => Err(WorthServerQueryHandoffDenial::new(
                WorthServerQueryHandoffDenialCode::CompatibilityMutationRequestInvalid,
                diagnostics_profile,
                "compatibility mutation body may define `command` or `commands`, but not both",
            )),
            (Some(command), None) => {
                let parsed =
                    WorthServerCompatibilityMutationCommand::parse(command, diagnostics_profile)?;
                Ok(Self::new(vec![parsed], false))
            }
            (None, Some(commands)) => {
                let rows = commands.as_array().ok_or_else(|| {
                    WorthServerQueryHandoffDenial::new(
                        WorthServerQueryHandoffDenialCode::CompatibilityMutationRequestInvalid,
                        diagnostics_profile,
                        "compatibility mutation `commands` must be an array",
                    )
                })?;
                if rows.is_empty() {
                    return Err(WorthServerQueryHandoffDenial::new(
                        WorthServerQueryHandoffDenialCode::CompatibilityMutationRequestInvalid,
                        diagnostics_profile,
                        "compatibility batch mutation must contain at least one command",
                    ));
                }
                let mut parsed = Vec::with_capacity(rows.len());
                for command in rows {
                    parsed.push(WorthServerCompatibilityMutationCommand::parse(
                        command,
                        diagnostics_profile,
                    )?);
                }
                Ok(Self::new(parsed, true))
            }
            (None, None) => Err(WorthServerQueryHandoffDenial::new(
                WorthServerQueryHandoffDenialCode::CompatibilityMutationRequestInvalid,
                diagnostics_profile,
                "compatibility mutation body must define `command` or `commands`",
            )),
        }
    }

    fn new(commands: Vec<WorthServerCompatibilityMutationCommand>, batch: bool) -> Self {
        let canonical_digest = format!(
            "compat-http-mutation-request-v1|kind:{}|commands:{}",
            if batch { "batch" } else { "single" },
            commands
                .iter()
                .map(WorthServerCompatibilityMutationCommand::canonical_digest)
                .collect::<Vec<_>>()
                .join("||"),
        );
        Self {
            commands,
            batch,
            canonical_digest,
        }
    }

    pub fn commands(&self) -> &[WorthServerCompatibilityMutationCommand] {
        &self.commands
    }

    pub fn is_batch(&self) -> bool {
        self.batch
    }

    pub fn canonical_digest(&self) -> &str {
        &self.canonical_digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub enum WorthServerCompatibilityMutationCommand {
    Insert {
        collection: String,
        aspects: BTreeMap<String, Value>,
        metadata: BTreeMap<String, Value>,
    },
    Update {
        entity_identity: String,
        aspects: BTreeMap<String, Value>,
        metadata: BTreeMap<String, Value>,
    },
    Delete {
        entity_identity: String,
        declared_collection: Option<String>,
        touched_aspect_paths: Vec<String>,
        metadata: BTreeMap<String, Value>,
    },
    VerifyExisting {
        authoritative_identity: String,
        resolved_entity_identity: String,
        target_collection: String,
        asserted_aspects: BTreeMap<String, Value>,
    },
}

impl WorthServerCompatibilityMutationCommand {
    fn parse(
        value: &Value,
        diagnostics_profile: worth_foundational::facade::DiagnosticRichnessProfile,
    ) -> Result<Self, WorthServerQueryHandoffDenial> {
        let object = value.as_object().ok_or_else(|| {
            WorthServerQueryHandoffDenial::new(
                WorthServerQueryHandoffDenialCode::CompatibilityMutationRequestInvalid,
                diagnostics_profile,
                "compatibility mutation command must be a JSON object",
            )
        })?;
        let family = string_field(object, "family", diagnostics_profile)?;
        match family.as_str() {
            "insert" => Ok(Self::Insert {
                collection: string_field(object, "collection", diagnostics_profile)?,
                aspects: aspects_field(object, "aspects", diagnostics_profile)?,
                metadata: metadata_field(object),
            }),
            "update" => Ok(Self::Update {
                entity_identity: string_field(object, "entity_identity", diagnostics_profile)?,
                aspects: aspects_field(object, "aspects", diagnostics_profile)?,
                metadata: metadata_field(object),
            }),
            "delete" => Ok(Self::Delete {
                entity_identity: string_field(object, "entity_identity", diagnostics_profile)?,
                declared_collection: optional_string_field(
                    object,
                    "declared_collection",
                    diagnostics_profile,
                )?,
                touched_aspect_paths: string_array_field(
                    object,
                    "touched_aspect_paths",
                    diagnostics_profile,
                )?,
                metadata: metadata_field(object),
            }),
            "verify_existing" => Ok(Self::VerifyExisting {
                authoritative_identity: string_field(
                    object,
                    "authoritative_identity",
                    diagnostics_profile,
                )?,
                resolved_entity_identity: string_field(
                    object,
                    "resolved_entity_identity",
                    diagnostics_profile,
                )?,
                target_collection: string_field(object, "target_collection", diagnostics_profile)?,
                asserted_aspects: aspects_field(object, "asserted_aspects", diagnostics_profile)?,
            }),
            other => Err(WorthServerQueryHandoffDenial::new(
                WorthServerQueryHandoffDenialCode::CompatibilityMutationFamilyUnsupported,
                diagnostics_profile,
                format!("compatibility mutation family `{other}` is not supported"),
            )),
        }
    }

    pub fn canonical_digest(&self) -> String {
        match self {
            Self::Insert {
                collection,
                aspects,
                metadata,
            } => format!(
                "family=insert|collection={collection}|aspects={}|metadata={}",
                canonical_map_digest(aspects),
                canonical_map_digest(metadata),
            ),
            Self::Update {
                entity_identity,
                aspects,
                metadata,
            } => format!(
                "family=update|entity={entity_identity}|aspects={}|metadata={}",
                canonical_map_digest(aspects),
                canonical_map_digest(metadata),
            ),
            Self::Delete {
                entity_identity,
                declared_collection,
                touched_aspect_paths,
                metadata,
            } => format!(
                "family=delete|entity={entity_identity}|collection={}|touched={}|metadata={}",
                declared_collection.as_deref().unwrap_or("none"),
                touched_aspect_paths.join(","),
                canonical_map_digest(metadata),
            ),
            Self::VerifyExisting {
                authoritative_identity,
                resolved_entity_identity,
                target_collection,
                asserted_aspects,
            } => format!(
                "family=verify_existing|authority={authoritative_identity}|resolved={resolved_entity_identity}|collection={target_collection}|asserted={}",
                canonical_map_digest(asserted_aspects),
            ),
        }
    }
}

fn string_field(
    object: &serde_json::Map<String, Value>,
    field: &str,
    diagnostics_profile: worth_foundational::facade::DiagnosticRichnessProfile,
) -> Result<String, WorthServerQueryHandoffDenial> {
    let value = object.get(field).ok_or_else(|| {
        WorthServerQueryHandoffDenial::new(
            WorthServerQueryHandoffDenialCode::CompatibilityMutationRequestInvalid,
            diagnostics_profile,
            format!("compatibility mutation field `{field}` is required"),
        )
    })?;
    let value = value.as_str().ok_or_else(|| {
        WorthServerQueryHandoffDenial::new(
            WorthServerQueryHandoffDenialCode::CompatibilityMutationRequestInvalid,
            diagnostics_profile,
            format!("compatibility mutation field `{field}` must be a string"),
        )
    })?;
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(WorthServerQueryHandoffDenial::new(
            WorthServerQueryHandoffDenialCode::CompatibilityMutationRequestInvalid,
            diagnostics_profile,
            format!("compatibility mutation field `{field}` may not be blank"),
        ));
    }
    Ok(normalized.to_string())
}

fn optional_string_field(
    object: &serde_json::Map<String, Value>,
    field: &str,
    diagnostics_profile: worth_foundational::facade::DiagnosticRichnessProfile,
) -> Result<Option<String>, WorthServerQueryHandoffDenial> {
    let Some(value) = object.get(field) else {
        return Ok(None);
    };
    let normalized = value.as_str().ok_or_else(|| {
        WorthServerQueryHandoffDenial::new(
            WorthServerQueryHandoffDenialCode::CompatibilityMutationRequestInvalid,
            diagnostics_profile,
            format!("compatibility mutation field `{field}` must be a string"),
        )
    })?;
    Ok(Some(normalized.trim().to_string()))
}

fn aspects_field(
    object: &serde_json::Map<String, Value>,
    field: &str,
    diagnostics_profile: worth_foundational::facade::DiagnosticRichnessProfile,
) -> Result<BTreeMap<String, Value>, WorthServerQueryHandoffDenial> {
    let value = object.get(field).ok_or_else(|| {
        WorthServerQueryHandoffDenial::new(
            WorthServerQueryHandoffDenialCode::CompatibilityMutationRequestInvalid,
            diagnostics_profile,
            format!("compatibility mutation field `{field}` is required"),
        )
    })?;
    let rows = value.as_object().ok_or_else(|| {
        WorthServerQueryHandoffDenial::new(
            WorthServerQueryHandoffDenialCode::CompatibilityMutationRequestInvalid,
            diagnostics_profile,
            format!("compatibility mutation field `{field}` must be an object"),
        )
    })?;
    if rows.is_empty() {
        return Err(WorthServerQueryHandoffDenial::new(
            WorthServerQueryHandoffDenialCode::CompatibilityMutationRequestInvalid,
            diagnostics_profile,
            format!("compatibility mutation field `{field}` may not be empty"),
        ));
    }
    let mut aspects = BTreeMap::new();
    for (name, value) in rows {
        let normalized = name.trim();
        if normalized.is_empty() {
            return Err(WorthServerQueryHandoffDenial::new(
                WorthServerQueryHandoffDenialCode::CompatibilityMutationRequestInvalid,
                diagnostics_profile,
                format!(
                    "compatibility mutation field `{field}` may not contain blank aspect names"
                ),
            ));
        }
        aspects.insert(normalized.to_string(), value.clone());
    }
    Ok(aspects)
}

fn metadata_field(object: &serde_json::Map<String, Value>) -> BTreeMap<String, Value> {
    let mut metadata = BTreeMap::new();
    if let Some(rows) = object.get("metadata").and_then(Value::as_object) {
        for (name, value) in rows {
            metadata.insert(name.trim().to_string(), value.clone());
        }
    }
    metadata
}

fn string_array_field(
    object: &serde_json::Map<String, Value>,
    field: &str,
    diagnostics_profile: worth_foundational::facade::DiagnosticRichnessProfile,
) -> Result<Vec<String>, WorthServerQueryHandoffDenial> {
    let Some(value) = object.get(field) else {
        return Ok(Vec::new());
    };
    let rows = value.as_array().ok_or_else(|| {
        WorthServerQueryHandoffDenial::new(
            WorthServerQueryHandoffDenialCode::CompatibilityMutationRequestInvalid,
            diagnostics_profile,
            format!("compatibility mutation field `{field}` must be an array"),
        )
    })?;
    let mut parsed = Vec::with_capacity(rows.len());
    for row in rows {
        let value = row.as_str().ok_or_else(|| {
            WorthServerQueryHandoffDenial::new(
                WorthServerQueryHandoffDenialCode::CompatibilityMutationRequestInvalid,
                diagnostics_profile,
                format!("compatibility mutation field `{field}` must contain only strings"),
            )
        })?;
        parsed.push(value.trim().to_string());
    }
    parsed.sort();
    parsed.dedup();
    Ok(parsed)
}

fn canonical_map_digest(entries: &BTreeMap<String, Value>) -> String {
    entries
        .iter()
        .map(|(name, value)| format!("{name}={}", canonical_json(value)))
        .collect::<Vec<_>>()
        .join(",")
}

fn canonical_json(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::String(value) => format!("{value:?}"),
        Value::Array(values) => format!(
            "[{}]",
            values
                .iter()
                .map(canonical_json)
                .collect::<Vec<_>>()
                .join(",")
        ),
        Value::Object(values) => {
            let rows = values
                .iter()
                .map(|(name, value)| (name.as_str(), value))
                .collect::<BTreeMap<_, _>>();
            format!(
                "{{{}}}",
                rows.iter()
                    .map(|(name, value)| format!("{name:?}:{}", canonical_json(value)))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        }
    }
}
