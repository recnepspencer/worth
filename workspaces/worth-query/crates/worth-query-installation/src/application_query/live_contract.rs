use worth_query_declaration::facade::{
    application_query::{
        ApplicationQueryLiveCauseContract, ApplicationQueryLiveTargetMode,
        ErasedApplicationQueryDefinition,
    },
    application_schema::{ApplicationSchemaMember, ErasedApplicationSchemaDeclaration},
    domain_computation::{
        WorthQueryCancellationSafePointFamily, WorthQueryExecutionMode,
        WorthQueryResourceDimension, WorthQueryResourceLimitRequest,
        WorthQueryRetainedProgressPosture, WorthQuerySemanticScaleRequest,
    },
    portable_identity::WorthQueryPortableTypeIdentity,
};

use crate::domain_computation::WorthQueryExecutionResourceEnvelope;

use super::{
    WorthQueryApplicationQueryInstallationDenial, WorthQueryApplicationQueryInstallationDenialKind,
    WorthQueryInstalledApplicationContinuationContract, WorthQueryInstalledGraphProjection,
    WorthQueryInstalledGraphReadContract,
};

pub struct WorthQueryInstalledApplicationLiveContract {
    binding_type: WorthQueryPortableTypeIdentity,
    effect: String,
    payload_type: WorthQueryPortableTypeIdentity,
    scope_identity: WorthQueryInstalledGraphProjection,
    target_identity: WorthQueryInstalledGraphProjection,
    target_mode: ApplicationQueryLiveTargetMode,
    collection_path: Option<String>,
    resource_envelope: WorthQueryExecutionResourceEnvelope,
}

impl WorthQueryInstalledApplicationLiveContract {
    pub(super) fn compile(
        definition: &ErasedApplicationQueryDefinition,
        schema: &ErasedApplicationSchemaDeclaration,
        graph: &WorthQueryInstalledGraphReadContract,
        continuation: Option<&WorthQueryInstalledApplicationContinuationContract>,
    ) -> Result<Option<Self>, WorthQueryApplicationQueryInstallationDenial> {
        let Some(live) = definition.live_cause() else {
            return Ok(None);
        };
        validate_effect_is_installed(schema, live)?;
        let scope_identity = require_installed_projection(
            graph,
            live.scope_slot_identity(),
            live.scope_field(),
            WorthQueryApplicationQueryInstallationDenialKind::LiveScopeIdentityNotInstalled,
        )?;
        let target_identity = require_installed_projection(
            graph,
            live.target_slot_identity(),
            live.target_field(),
            WorthQueryApplicationQueryInstallationDenialKind::LiveTargetIdentityNotInstalled,
        )?;
        let collection_path = match live.target_mode() {
            ApplicationQueryLiveTargetMode::Root => {
                if target_identity.parent_path() != "root" {
                    return Err(installation_denial(
                        WorthQueryApplicationQueryInstallationDenialKind::LiveTargetIdentityNotInstalled,
                        target_identity.result_path(),
                    ));
                }
                None
            }
            ApplicationQueryLiveTargetMode::Collection => {
                let continuation = continuation.ok_or_else(|| {
                    installation_denial(
                        WorthQueryApplicationQueryInstallationDenialKind::LiveTargetIdentityNotInstalled,
                        definition.name(),
                    )
                })?;
                if target_identity.parent_path() != continuation.collection_path() {
                    return Err(installation_denial(
                        WorthQueryApplicationQueryInstallationDenialKind::LiveTargetIdentityNotInstalled,
                        target_identity.result_path(),
                    ));
                }
                Some(continuation.collection_path().to_string())
            }
        };
        Ok(Some(Self {
            binding_type: live.binding_identity(),
            effect: live.effect().to_string(),
            payload_type: live.payload_identity(),
            scope_identity,
            target_identity,
            target_mode: live.target_mode(),
            collection_path,
            resource_envelope: compile_resource_envelope(live),
        }))
    }

    pub fn binding_type(&self) -> &str {
        self.binding_type.as_str()
    }

    pub fn effect(&self) -> &str {
        &self.effect
    }

    pub fn payload_type(&self) -> &str {
        self.payload_type.as_str()
    }

    pub const fn scope_identity(&self) -> &WorthQueryInstalledGraphProjection {
        &self.scope_identity
    }

    pub const fn target_identity(&self) -> &WorthQueryInstalledGraphProjection {
        &self.target_identity
    }

    pub const fn target_mode(&self) -> ApplicationQueryLiveTargetMode {
        self.target_mode
    }

    pub fn collection_path(&self) -> Option<&str> {
        self.collection_path.as_deref()
    }

    pub const fn resource_envelope(&self) -> &WorthQueryExecutionResourceEnvelope {
        &self.resource_envelope
    }
}

fn validate_effect_is_installed(
    schema: &ErasedApplicationSchemaDeclaration,
    live: &ApplicationQueryLiveCauseContract,
) -> Result<(), WorthQueryApplicationQueryInstallationDenial> {
    let installed = schema.members().iter().any(|member| {
        matches!(
            member,
            ApplicationSchemaMember::Effect {
                effect,
                payload_type,
            } if effect == live.effect() && *payload_type == live.payload_identity()
        )
    });
    if installed {
        Ok(())
    } else {
        Err(installation_denial(
            WorthQueryApplicationQueryInstallationDenialKind::LiveEffectNotInstalled,
            live.effect(),
        ))
    }
}

fn installed_projection(
    graph: &WorthQueryInstalledGraphReadContract,
    slot_type: WorthQueryPortableTypeIdentity,
    field: (&str, &str, &str),
) -> Option<WorthQueryInstalledGraphProjection> {
    graph
        .projections()
        .iter()
        .find(|projection| {
            projection.portable_slot_identity() == slot_type
                && (projection.entity(), projection.aspect(), projection.field()) == field
        })
        .cloned()
}

fn require_installed_projection(
    graph: &WorthQueryInstalledGraphReadContract,
    slot_type: WorthQueryPortableTypeIdentity,
    field: (&str, &str, &str),
    denial_kind: WorthQueryApplicationQueryInstallationDenialKind,
) -> Result<WorthQueryInstalledGraphProjection, WorthQueryApplicationQueryInstallationDenial> {
    installed_projection(graph, slot_type.clone(), field)
        .ok_or_else(|| installation_denial(denial_kind, slot_type.as_str()))
}

fn compile_resource_envelope(
    live: &ApplicationQueryLiveCauseContract,
) -> WorthQueryExecutionResourceEnvelope {
    let resources = live.resources();
    let limits = WorthQueryResourceLimitRequest::bounded(resources.maximum_work_per_delivery())
        .with(
            WorthQueryResourceDimension::QueueDepth,
            resources.maximum_buffered_causes(),
        )
        .with(WorthQueryResourceDimension::ChunkWidth, 1)
        .with(
            WorthQueryResourceDimension::RetainedBytes,
            resources.maximum_retained_payload_bytes(),
        );
    WorthQueryExecutionResourceEnvelope::new(
        WorthQuerySemanticScaleRequest::bounded(resources.maximum_work_per_delivery()),
        limits,
        WorthQueryExecutionMode::Asynchronous,
        None,
        WorthQueryCancellationSafePointFamily::new("application-query-live-delivery")
            .expect("the installed live safe-point family is canonical"),
    )
    .with_retained_progress_posture(WorthQueryRetainedProgressPosture::RetainAttemptCapacity)
}

fn installation_denial(
    kind: WorthQueryApplicationQueryInstallationDenialKind,
    subject: impl Into<String>,
) -> WorthQueryApplicationQueryInstallationDenial {
    WorthQueryApplicationQueryInstallationDenial::new(kind, subject)
}
