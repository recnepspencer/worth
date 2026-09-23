//! Lowering of the platform-owned entity kinds that sit above the application
//! catalog.
//!
//! These kinds are never application meaning: they are reserved after the
//! highest application aspect identity and the highest application kind, so an
//! application declaration can neither name them nor collide with them.

use worth_query_installation::facade::WorthQueryInstalledApplicationSchemaContractCatalog;
use worth_relational::facade::identity::KindId;
use worth_relational::facade::schema::{RelationalSchemaRegistry, SchemaId, SchemaVersionId};

use super::platform_identity_allocator::allocate_platform_aspect_identities;
use super::program_activation::{lower_program_activation, WorthQueryProgramActivationLayout};
use super::provider_aftermath_causality::{
    lower_provider_aftermath_causality, WorthQueryAftermathCausalityLayout,
};
use super::provider_dispatch_outbox::lower_provider_dispatch_outbox;
use super::provider_idempotency::{
    lower_provider_idempotency, WorthQueryProviderIdempotencyLayout,
};
use super::{kind_space_exhausted, WorthQueryPrimaryGraphInstallationDenial};
use crate::domain_computation::application_aftermath::WorthQueryDispatchOutboxLayout;
use crate::domain_computation::primary_graph::workflow::schema::{
    lower_workflow, WorthQueryWorkflowLayout,
};

/// Every platform-owned record layout one lowered application graph carries.
pub(super) struct WorthQueryPlatformEntityLayouts {
    pub(super) provider_idempotency: WorthQueryProviderIdempotencyLayout,
    pub(super) provider_dispatch_outbox: WorthQueryDispatchOutboxLayout,
    pub(super) provider_aftermath_causality: WorthQueryAftermathCausalityLayout,
    pub(super) program_activation: WorthQueryProgramActivationLayout,
    pub(super) workflow: WorthQueryWorkflowLayout,
}

pub(super) fn lower_platform_entities(
    registry: RelationalSchemaRegistry,
    schema_id: &SchemaId,
    schema_version_id: SchemaVersionId,
    native_contracts: &WorthQueryInstalledApplicationSchemaContractCatalog,
    first_kind: KindId,
) -> Result<
    (RelationalSchemaRegistry, WorthQueryPlatformEntityLayouts),
    WorthQueryPrimaryGraphInstallationDenial,
> {
    let identities = allocate_platform_aspect_identities(native_contracts)?;
    let (registry, provider_idempotency) = lower_provider_idempotency(
        registry,
        schema_id,
        schema_version_id,
        first_kind,
        identities[0],
    )?;
    let dispatch_outbox_kind = next_kind(first_kind)?;
    let (registry, provider_dispatch_outbox) = lower_provider_dispatch_outbox(
        registry,
        schema_id,
        schema_version_id,
        dispatch_outbox_kind,
        identities[1],
    )?;
    let aftermath_causality_kind = next_kind(dispatch_outbox_kind)?;
    let (registry, provider_aftermath_causality) = lower_provider_aftermath_causality(
        registry,
        schema_id,
        schema_version_id,
        aftermath_causality_kind,
        identities[2],
    )?;
    let program_activation_kind = next_kind(aftermath_causality_kind)?;
    let (registry, program_activation) = lower_program_activation(
        registry,
        schema_id,
        schema_version_id,
        program_activation_kind,
        identities[3],
    )?;
    let workflow_kind = next_kind(program_activation_kind)?;
    let (registry, workflow) = lower_workflow(
        registry,
        schema_id,
        schema_version_id,
        workflow_kind,
        [
            identities[4],
            identities[5],
            identities[6],
            identities[7],
            identities[8],
            identities[9],
            identities[10],
            identities[11],
            identities[12],
            identities[13],
        ],
    )?;
    Ok((
        registry,
        WorthQueryPlatformEntityLayouts {
            provider_idempotency,
            provider_dispatch_outbox,
            provider_aftermath_causality,
            program_activation,
            workflow,
        },
    ))
}

fn next_kind(previous: KindId) -> Result<KindId, WorthQueryPrimaryGraphInstallationDenial> {
    previous
        .0
        .checked_add(1)
        .map(KindId)
        .ok_or_else(kind_space_exhausted)
}
