//! Native read custody is closed; original readonly lineage remains deliverable.
use super::*;
/// Original complete-tree evidence after all native interests and cleanup closed.
/// Fresh delivery validates the original complete tree and selected typed root.
/// This pins readonly history and the authentic runtime, not outstanding cleanup.
/// Drop it and returned observations promptly, before whole-runtime close/OS join.
#[must_use]
pub struct WorthQueryClosedProgramOutputDelivery<S, P, R>
where
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    pub(super) closed: WorthQueryClosedProgramOutputRequest,
    // Runtime is last in the entire destruction order, not just in this core.
    pub(super) delivery: ProgramOutputDeliveryCore<S, P, R>,
}
impl<S, P, R> WorthQueryClosedProgramOutputDelivery<S, P, R>
where
    S: ApplicationSchema + 'static,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    /// The original branch required by every fresh delivery request.
    pub fn branch(&self) -> WorthQueryProductBranch {
        self.delivery.branch
    }
    /// Authentic native close evidence; no cleanup can be reopened from it.
    pub fn close_evidence(&self) -> &WorthQueryClosedProgramOutputRequest {
        &self.closed
    }
    /// Validates all original settlements/discovery and the selected typed root.
    /// The returned observation is current as of this fresh native read phase.
    pub fn deliver_current(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        policy: WorthQueryOperationDeliveryPolicy,
    ) -> Result<WorthQueryApplicationReadObservation, WorthQueryOperationDeliveryDenial> {
        self.delivery.deliver_current(request, policy)
    }
    /// Validates original complete membership and reads at that same basis/phase.
    /// Refusal retains this already-closed carrier without creating new work.
    pub fn read_current<I>(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        intent: I,
    ) -> Result<
        crate::domain_computation::WorthQueryPublishedApplicationResult<
            ReadQuery<S, I>,
            ReadValue<S, I>,
        >,
        WorthQueryOperationDeliveryDenial,
    >
    where
        I: ApplicationQueryIntent<S>,
        <ReadBinding<S, I> as ApplicationQueryBinding<S>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                S,
                <ReadBinding<S, I> as ApplicationQueryBinding<S>>::PrincipalIdentity,
            >,
        ReadValue<S, I>: WorthQueryApplicationProjection<S, ReadQuery<S, I>>,
    {
        self.delivery.read_current(request, intent)
    }
    /// Narrows installed native ceilings without weakening complete currentness.
    pub fn read_current_with_policy<I>(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        intent: I,
        policy: WorthQueryOperationDeliveryPolicy,
    ) -> Result<
        crate::domain_computation::WorthQueryPublishedApplicationResult<
            ReadQuery<S, I>,
            ReadValue<S, I>,
        >,
        WorthQueryOperationDeliveryDenial,
    >
    where
        I: ApplicationQueryIntent<S>,
        <ReadBinding<S, I> as ApplicationQueryBinding<S>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                S,
                <ReadBinding<S, I> as ApplicationQueryBinding<S>>::PrincipalIdentity,
            >,
        ReadValue<S, I>: WorthQueryApplicationProjection<S, ReadQuery<S, I>>,
    {
        self.delivery
            .read_current_with_policy(request, intent, policy)
    }
    /// Original complete historical settlement, including immutable typed root roles.
    /// Borrowing it alone establishes neither currentness nor fresh read authority.
    pub fn settlement(&self) -> &WorthQueryApplicationProgramOutputSettlement<Query<S, R>> {
        self.delivery.settlement()
    }
}
