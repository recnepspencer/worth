//! Real handlers declare exactly the later-phase custody their operations use.
use super::*;
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerResult, OperationHandler,
    WorthQueryOperationDurableCapability as Capability,
};
#[derive(Clone, Copy)]
pub(in super::super::super) enum LabelPolicy {
    Original,
    Output,
    Adoption,
    Managed,
    Resource,
    Conditional,
    Temporal,
    Workflow,
    OriginalOwner,
}
pub(super) fn install(
    setup: &mut WorthQueryApplicationContributionSetup<'_, IdentityExecutionSchema>,
    policy: LabelPolicy,
    contacts: Arc<AtomicUsize>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    match policy {
        LabelPolicy::Original => setup.handler::<LabelBinding, _>(LabelHandler),
        LabelPolicy::Output => setup.handler::<LabelBinding, _>(OutputHandler(contacts)),
        LabelPolicy::Adoption => setup.handler::<LabelBinding, _>(AdoptionHandler(contacts)),
        LabelPolicy::Managed => setup.handler::<LabelBinding, _>(ManagedHandler(contacts)),
        LabelPolicy::Resource => setup.handler::<LabelBinding, _>(ResourceHandler(contacts)),
        LabelPolicy::Conditional => setup.handler::<LabelBinding, _>(ConditionalHandler(contacts)),
        LabelPolicy::Temporal => setup.handler::<LabelBinding, _>(TemporalHandler(contacts)),
        LabelPolicy::Workflow => setup.handler::<LabelBinding, _>(WorkflowHandler(contacts)),
        LabelPolicy::OriginalOwner => {
            setup.handler::<LabelBinding, _>(OriginalOwnerHandler(contacts))
        }
    }
}
struct OutputHandler(Arc<AtomicUsize>);
impl OperationHandler<IdentityExecutionSchema, LabelBinding> for OutputHandler {
    fn declared_durable_capabilities() -> Option<&'static [Capability]> {
        Some(&[Capability::RequiredOutputDemand])
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
struct AdoptionHandler(Arc<AtomicUsize>);
impl OperationHandler<IdentityExecutionSchema, LabelBinding> for AdoptionHandler {
    fn declared_durable_capabilities() -> Option<&'static [Capability]> {
        Some(&[Capability::ProgramAdoption])
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
struct ManagedHandler(Arc<AtomicUsize>);
impl OperationHandler<IdentityExecutionSchema, LabelBinding> for ManagedHandler {
    fn declared_durable_capabilities() -> Option<&'static [Capability]> {
        Some(&[Capability::ManagedExecution])
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
struct ResourceHandler(Arc<AtomicUsize>);
impl OperationHandler<IdentityExecutionSchema, LabelBinding> for ResourceHandler {
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
struct ConditionalHandler(Arc<AtomicUsize>);
impl OperationHandler<IdentityExecutionSchema, LabelBinding> for ConditionalHandler {
    fn declared_durable_capabilities() -> Option<&'static [Capability]> {
        Some(&[Capability::ConditionalExecution])
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
struct TemporalHandler(Arc<AtomicUsize>);
impl OperationHandler<IdentityExecutionSchema, LabelBinding> for TemporalHandler {
    fn declared_durable_capabilities() -> Option<&'static [Capability]> {
        Some(&[Capability::TemporalExecution])
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
struct WorkflowHandler(Arc<AtomicUsize>);
impl OperationHandler<IdentityExecutionSchema, LabelBinding> for WorkflowHandler {
    fn declared_durable_capabilities() -> Option<&'static [Capability]> {
        Some(&[Capability::WorkflowExecution])
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
struct OriginalOwnerHandler(Arc<AtomicUsize>);
impl OperationHandler<IdentityExecutionSchema, LabelBinding> for OriginalOwnerHandler {
    fn declared_durable_capabilities() -> Option<&'static [Capability]> {
        Some(&[Capability::RetainedOriginalOperationOwner])
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
