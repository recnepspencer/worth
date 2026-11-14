//! Real handlers declare exactly the later-phase custody their operations use.
use super::*;
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerResult, OperationHandler,
    WorthQueryOperationDurableCapability as Capability,
};
#[derive(Clone, Copy)]
pub(in super::super::super) enum LabelPolicy {
    Original,
    Resource,
}
pub(super) fn install(
    setup: &mut WorthQueryApplicationContributionSetup<'_, IdentityExecutionSchema>,
    policy: LabelPolicy,
    contacts: Arc<AtomicUsize>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    match policy {
        LabelPolicy::Original => setup.handler::<LabelBinding, _>(LabelHandler),

        LabelPolicy::Resource => {
            setup.handler::<LabelBinding, _>(ResourceDeclaringLabelHandler(contacts))
        }
    }
}

struct ResourceDeclaringLabelHandler(Arc<AtomicUsize>);
impl OperationHandler<IdentityExecutionSchema, LabelBinding> for ResourceDeclaringLabelHandler {
    fn declared_durable_capabilities() -> Option<&'static [Capability]> {
        Some(&[Capability::ResourceRequest])
    }
    fn decide(
        &self,
        input: &LabelEdit,
        reader: &mut DecisionReader<'_, '_, '_, IdentityExecutionSchema, LabelBinding>,
    ) -> HandlerResult<LabelEdit, LabelEdit> {
        self.0.fetch_add(1, Ordering::SeqCst);
        LabelHandler.decide(input, reader)
    }
    fn build_candidate(
        &self,
        input: &LabelEdit,
        decision: LabelEdit,
        writer: &mut CandidateWriter<'_, IdentityExecutionSchema, LabelBinding>,
    ) -> HandlerResult<LabelEdit, LabelEdit> {
        LabelHandler.build_candidate(input, decision, writer)
    }
}
