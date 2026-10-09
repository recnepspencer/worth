//! Checked promotion of the original program-source carrier.

use std::any::TypeId;
use std::sync::Arc;

use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_query_declaration::facade::application_program::{
    ApplicationOutputGraphShape, ApplicationProgramDefinition, ApplicationProgramOutputsShape,
};
use worth_query_installation::facade::ApplicationSchema;

use super::super::{WorthQueryProgramApplicationRuntime, WorthQuerySelectedProgramOwner};
use super::{
    PreparedProgramSource, RootConnection, WorthQueryRecoveredProgramOutputSource,
    WorthQueryUnpublishedProgramOutputSource,
};
use crate::domain_computation::primary_graph::application_output_demand::PreparedOutputRootKind;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryApplicationDiscoveredOutputConnection,
    WorthQueryApplicationIdempotencyBinding, WorthQueryOutputDemandDenial,
    WorthQueryOutputDemandDenialKind,
};
use crate::publication_boundary::WorthQueryProgramPublicationAccess;

impl<Schema, Program> WorthQueryProgramApplicationRuntime<Schema, Program>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Program::Outputs: ApplicationProgramOutputsShape<Schema>,
{
    /// Checks the same installed root, source binding, original request and
    /// selected canonical program before admitting a discovered recovery.
    pub fn validate_unpublished_discovered_source<Root, Source>(
        &self,
        _: &WorthQueryProgramPublicationAccess,
        owner: &WorthQuerySelectedProgramOwner<'_, Schema>,
        source: &WorthQueryUnpublishedProgramOutputSource,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        branch: crate::basis::WorthQueryProductBranch,
    ) -> Result<(), WorthQueryOutputDemandDenial>
    where
        Root: ApplicationOutputGraphShape<Schema>,
        Source: ApplicationMutationBinding<Schema>,
        RootConnection<Schema, Root>:
            WorthQueryApplicationDiscoveredOutputConnection<Schema, Source = Source>,
    {
        let presented = self
            .selected_source_owner::<Source>(owner, TypeId::of::<Root>())
            .map_err(|_| {
                denied(
                    WorthQueryOutputDemandDenialKind::ForeignSource,
                    "selected program does not own the original discovered source",
                )
            })?;
        if source.runtime_authority != self.runtime.runtime.authority_identity().as_u64()
            || source.program != *presented.rendering()
            || source.binding != TypeId::of::<Source>()
            || source.payload.root_kind() != PreparedOutputRootKind::Discovered(TypeId::of::<Root>())
            || source.branch != branch
            || source.idempotency != idempotency
            || !source.payload.has_discovery::<<RootConnection<Schema, Root> as WorthQueryApplicationDiscoveredOutputConnection<Schema>>::Discovery>()
        {
            return Err(denied(WorthQueryOutputDemandDenialKind::ForeignSource,
                "recovery differs from the original discovered source admission"));
        }
        Ok(())
    }

    /// Checks the same installed root, source binding, original request and
    /// selected canonical program before admitting a required recovery.
    pub fn validate_unpublished_required_source<Root, Source>(
        &self,
        _: &WorthQueryProgramPublicationAccess,
        owner: &WorthQuerySelectedProgramOwner<'_, Schema>,
        source: &WorthQueryUnpublishedProgramOutputSource,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        branch: crate::basis::WorthQueryProductBranch,
    ) -> Result<(), WorthQueryOutputDemandDenial>
    where
        Root: ApplicationOutputGraphShape<Schema>,
        Source: ApplicationMutationBinding<Schema>,
        RootConnection<Schema, Root>:
            crate::domain_computation::primary_graph::WorthQueryApplicationRequiredOutputConnection<
                Schema,
            >,
        Source: crate::domain_computation::primary_graph::WorthQueryApplicationRequiredOutputSource<
            Schema,
            RootConnection<Schema, Root>,
        >,
    {
        let presented = self
            .selected_source_owner::<Source>(owner, TypeId::of::<Root>())
            .map_err(|_| {
                denied(
                    WorthQueryOutputDemandDenialKind::ForeignSource,
                    "selected program does not own the original required source",
                )
            })?;
        if source.runtime_authority != self.runtime.runtime.authority_identity().as_u64()
            || source.program != *presented.rendering()
            || source.binding != TypeId::of::<Source>()
            || source.payload.root_kind() != PreparedOutputRootKind::Required(TypeId::of::<Root>())
            || source.branch != branch
            || source.idempotency != idempotency
        {
            return Err(denied(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                "recovery differs from the original required source admission",
            ));
        }
        Ok(())
    }

    /// Transfers the original carrier only after a matching published keyed
    /// receipt exists. Every refusal returns both native owners unchanged.
    pub fn promote_program_output_source(
        &self,
        _: &WorthQueryProgramPublicationAccess,
        source: WorthQueryUnpublishedProgramOutputSource,
        carrier: WorthQueryRecoveredProgramOutputSource,
        receipt: WorthQueryApplicationCommitReceipt,
    ) -> Result<
        PreparedProgramSource,
        (
            WorthQueryOutputDemandDenial,
            WorthQueryUnpublishedProgramOutputSource,
            WorthQueryRecoveredProgramOutputSource,
        ),
    > {
        let retained = (|| {
            if source.runtime_authority != self.runtime.runtime.authority_identity().as_u64()
                || source.branch != receipt.product_branch()
                || carrier.publication.composite_commit()
                    != receipt.committed_product_publication().composite_commit()
            {
                return Err(denied(
                    WorthQueryOutputDemandDenialKind::ForeignSource,
                    "published receipt does not name the original recovered source",
                ));
            }
            let change = carrier.change.as_ref().ok_or_else(|| {
                denied(
                    WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                    "original recovered source has no one-use performed carrier",
                )
            })?;
            let observation = carrier.observation.as_ref().ok_or_else(|| {
                denied(
                    WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                    "original recovered source has no retained output observation",
                )
            })?;
            self.runtime.retain_required_output_source_carrier(
                receipt,
                Arc::clone(change),
                observation.clone(),
                &source.preparation,
                source.payload.root_kind(),
                source.payload.discovery(),
            )
        })();
        retained.map_err(|denial| (denial, source, carrier))
    }
}

fn denied(
    kind: WorthQueryOutputDemandDenialKind,
    message: &'static str,
) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(kind, message)
}
