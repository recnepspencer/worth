//! Request-local typed connection values; no durable producer input is retained.
use super::{
    WorthQueryApplicationAttemptDenial as Denial, WorthQueryApplicationAttemptDenialKind as Kind,
};
use crate::domain_computation::primary_graph::admitted_capture::checkpoint;
use crate::equality_type_id::EqualityTypeId;
use std::any::Any;
use worth_query_declaration::facade::{
    application_operation::ApplicationMutationBinding,
    application_program::{
        ApplicationCoupledOutputVisitor, WorthQueryApplicationOutputDemand,
        WorthQueryApplicationRequiredOutputConnection, WorthQueryApplicationRequiredOutputSource,
        WorthQueryProducerOutputFamily,
    },
    application_schema::ApplicationSchema,
};
mod preimage;
pub(in crate::domain_computation::primary_graph) mod source_arguments;
pub(super) use preimage::CoupledPreImage;
pub(in crate::domain_computation::primary_graph) use source_arguments::CoupledSourceArguments;

pub(in crate::domain_computation::primary_graph) struct DeclaredCoupledOutputDemand {
    pub(in crate::domain_computation::primary_graph) connection: &'static str,
    pub(in crate::domain_computation::primary_graph) family: &'static str,
    pub(in crate::domain_computation::primary_graph) family_type: EqualityTypeId,
    pub(in crate::domain_computation::primary_graph) source_binding_type: EqualityTypeId,
    pub(in crate::domain_computation::primary_graph) demand: Box<dyn Any + Send>,
    pub(in crate::domain_computation::primary_graph) lower_source:
        fn(&dyn Any, &dyn Any) -> Result<CoupledSourceArguments, Denial>,
}
#[derive(Default)]
pub struct DeclaredCoupledOutputDemands {
    pub(in crate::domain_computation::primary_graph) values: Vec<DeclaredCoupledOutputDemand>,
    _memory: Vec<worth_execution::ExecutionMemoryReservation>,
    failure: std::cell::RefCell<Option<Denial>>,
    declared: bool,
    pub(in crate::domain_computation::primary_graph) synchronous: Option<crate::domain_computation::primary_graph::application_contribution::producer::synchronous_outputs::CapturedSynchronousOutputs>,
}

impl DeclaredCoupledOutputDemands {
    pub(in crate::domain_computation::primary_graph) fn failed(cause: Denial) -> Self {
        Self {
            failure: std::cell::RefCell::new(Some(cause)),
            declared: true,
            ..Self::default()
        }
    }
    pub(in crate::domain_computation::primary_graph) fn is_declared(&self) -> bool {
        self.declared || self.synchronous.is_some()
    }
    pub(in crate::domain_computation::primary_graph) fn take_failure(&self) -> Option<Denial> {
        self.failure.borrow_mut().take()
    }
    fn finish(result: Result<Self, Denial>) -> Self {
        result.unwrap_or_else(Self::failed)
    }
}
pub(in crate::domain_computation::primary_graph) fn capture_correction<Schema, Correction>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    input: &<Correction::Source as ApplicationMutationBinding<Schema>>::Input,
    preimage: &dyn worth_query_declaration::facade::application_aftermath::ApplicationCorrectionPreImage<Schema>,
) -> DeclaredCoupledOutputDemands
where
    Schema: ApplicationSchema,
    Correction:
        worth_query_declaration::facade::application_aftermath::ApplicationCorrectionBinding<
            Schema,
        >,
{
    let mut visitor = Capture::<Schema, Correction::Source> {
        execution: *execution,
        input,
        preimage,
        demands: Ok(DeclaredCoupledOutputDemands::default()),
    };
    Correction::visit_coupled_outputs(&mut visitor);
    DeclaredCoupledOutputDemands::finish(visitor.demands)
}

pub(super) fn capture<Schema, Binding>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    input: &Binding::Input,
    preimage: &dyn worth_query_declaration::facade::application_aftermath::ApplicationCorrectionPreImage<Schema>,
) -> Result<DeclaredCoupledOutputDemands, Denial>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    let mut visitor = Capture::<Schema, Binding> {
        execution: *execution,
        input,
        preimage,
        demands: Ok(DeclaredCoupledOutputDemands::default()),
    };
    Binding::visit_coupled_outputs(&mut visitor);
    Ok(DeclaredCoupledOutputDemands::finish(visitor.demands))
}
struct Capture<'a, 'request, 'authority, Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>> {
    execution: worth_execution::ExecutionRequest<'request, 'authority>,
    input: &'a Binding::Input,
    preimage: &'a dyn worth_query_declaration::facade::application_aftermath::ApplicationCorrectionPreImage<Schema>,
    demands: Result<DeclaredCoupledOutputDemands, Denial>,
}
impl<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>>
    ApplicationCoupledOutputVisitor<Schema, Binding> for Capture<'_, '_, '_, Schema, Binding>
{
    fn required<Connection>(&mut self)
    where Connection: WorthQueryApplicationRequiredOutputConnection<Schema>, Connection::Demand: Send, Binding: WorthQueryApplicationRequiredOutputSource<Schema, Connection>,
        <source_arguments::Source<Schema, Connection> as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::Input: worth_query_declaration::facade::application_query::ApplicationQueryIntent<Schema, Binding = source_arguments::Source<Schema, Connection>>,
        <source_arguments::Source<Schema, Connection> as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::ScopeBinding: worth_query_declaration::facade::application_query::ApplicationQueryScopeResolution<Schema, <source_arguments::Source<Schema, Connection> as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::PrincipalIdentity>,
{
        type Family<S, C> = <<C as WorthQueryApplicationRequiredOutputConnection<S>>::Demand as WorthQueryApplicationOutputDemand<S>>::OutputFamily;
        type Source<S, C> = <Family<S, C> as WorthQueryProducerOutputFamily<S>>::Source;
        let Ok(demands) = &mut self.demands else {
            return;
        };
        demands.declared = true;
        let result = (|| {
            self.execution.run_native_with_checkpoint(|context| checkpoint(context, (Connection::IDENTITY.len() + Family::<Schema, Connection>::IDENTITY.len() + 1) as u64))
                .map_err(|stop| Denial::relational_read_stopped(stop.into(), Connection::IDENTITY))?.0
                .map_err(|stop| Denial::relational_read_stopped(worth_relational::facade::transactions::RelationalExecutionDenialCause::from(worth_execution::WorkCeilingDenial::Stopped(stop)), Connection::IDENTITY))?;
            let memory = self.execution.allocation_policy().reserve_memory((2 * size_of::<DeclaredCoupledOutputDemand>() + size_of::<<Source<Schema, Connection> as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::Input>()) as u64)
                .map_err(|cause| Denial::allocation_denied(Connection::IDENTITY, cause))?;
            let demand = Binding::demand_from_preimage(self.input, self.preimage)
                .map_err(|cause| Denial::new(Kind::CorrectionCaptureDenied, cause.subject()))?;
            demands.values.push(DeclaredCoupledOutputDemand {
                connection: Connection::IDENTITY,
                family: Family::<Schema, Connection>::IDENTITY,
                family_type: EqualityTypeId::of::<Family<Schema, Connection>>(),
                source_binding_type: EqualityTypeId::of::<Source<Schema, Connection>>(),
                demand: Box::new(demand),
                lower_source: source_arguments::lower::<Schema, Connection>,
            });
            demands._memory.extend(memory);
            Ok(())
        })();
        if let Err(cause) = result {
            self.demands = Err(cause);
        }
    }
}
