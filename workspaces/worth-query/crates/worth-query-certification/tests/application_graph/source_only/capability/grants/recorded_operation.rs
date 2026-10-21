//! Companion policy: ordinary action-2 grants plus the actual proposal's proposer.
use super::*;
use worth_query_decl::facade::worth_query_capability;
worth_query_capability!(pub RecordedOperationCapability in CapabilitySchema);
impl ApplicationCapabilityRequest<CapabilitySchema, RecordedOperationCapability>
    for SetRetentionInput
{
    type Scope = Document;
    type Context = GrantContext;
    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<CapabilitySchema, Document, GrantContext>,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        Ok(super::projection(2, &self.identity))
    }
}
pub(super) fn contract() -> ApplicationCapabilityContract<
    CapabilitySchema,
    RecordedOperationCapability,
    SetRetention,
    SetRetentionInput,
> {
    super::grant_contract(
        RecordedOperationCapability::reference(),
        SetRetention::reference(),
        2,
        2,
        ApplicationCapabilityRecordedActorRule::required(
            ApplicationCapabilityRecordedActorRequirement::new(
                ApplicationWorkflowRecordedActorSource::BoundProposal,
                ApplicationWorkflowRecordedRole::Proposer,
                ApplicationRecordedActorComparison::DifferentPrincipal,
            ),
        ),
    )
}
