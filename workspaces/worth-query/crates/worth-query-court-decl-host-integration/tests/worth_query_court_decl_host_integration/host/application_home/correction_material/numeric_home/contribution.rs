//! Directory-admissible NumericSchema contribution without MemoryOnly wake rules.
use super::model::{
    self, calculation, calculation_readiness,
    edit::{self},
    other_edit::{self},
    schema::*,
};
use worth_query_decl::facade::{application_operation::*, application_schema::*};
use worth_query_host::facade::{application_contribution::*, primary_graph::*};

struct DirectoryHandler<H>(H);
impl<B, H> OperationHandler<NumericSchema, B> for DirectoryHandler<H>
where
    B: ApplicationMutationBinding<NumericSchema>,
    H: OperationHandler<NumericSchema, B>,
{
    fn declared_durable_capabilities() -> Option<&'static [WorthQueryOperationDurableCapability]> {
        Some(&[])
    }
    fn decide(
        &self,
        input: &B::Input,
        reader: &mut DecisionReader<'_, '_, '_, NumericSchema, B>,
    ) -> HandlerResult<B::Decision, B::Denial> {
        self.0.decide(input, reader)
    }
    fn prepare_decision(
        &self,
        input: &B::Input,
        reader: &mut DecisionReader<'_, '_, '_, NumericSchema, B>,
    ) -> WorthQueryDecisionPlan<B::Decision, B::Denial> {
        self.0.prepare_decision(input, reader)
    }
    fn build_candidate(
        &self,
        input: &B::Input,
        decision: B::Decision,
        candidate: &mut CandidateWriter<'_, NumericSchema, B>,
    ) -> HandlerResult<B::Result, B::Denial> {
        self.0.build_candidate(input, decision, candidate)
    }
}

pub struct DirectoryNumericContribution;
impl ApplicationSchemaContribution<NumericSchema> for DirectoryNumericContribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        <NumericContribution as ApplicationSchemaContribution<NumericSchema>>::IDENTITY;
    type Members = <NumericContribution as ApplicationSchemaContribution<NumericSchema>>::Members;
}
impl WorthQueryApplicationContribution<NumericSchema> for DirectoryNumericContribution {
    type Configuration = ();
    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<NumericSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        contracts
            .producer::<model::causal_inspection::output_equivalence::calculation::Producer>()?;
        contracts
            .conditional::<model::causal_inspection::output_equivalence::readiness::Readiness>()?;
        contracts.producer::<model::causal_inspection::retired_outputs::calculation::Producer>()?;
        contracts
            .conditional::<model::causal_inspection::retired_outputs::readiness::Readiness>()?;
        contracts.producer::<calculation::Producer>()?;
        contracts.producer::<model::deferred_completion::reciprocal::Producer>()?;
        contracts.conditional::<model::deferred_completion::readiness::Readiness>()?;
        contracts.producer::<model::aspect_input::calculation::Producer>()?;
        contracts.producer::<model::scalar_input::calculation::Producer>()?;
        contracts.conditional::<model::scalar_input::readiness::Readiness>()?;
        contracts.conditional::<model::aspect_input::readiness::Readiness>()?;
        contracts.conditional::<calculation_readiness::Readiness>()?;
        Ok(())
    }
    fn configure(
        (): (),
        setup: &mut WorthQueryApplicationContributionSetup<'_, NumericSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.handler::<model::write_admission::Binding, _>(model::write_admission::Handler)?;
        setup.handler::<model::causal_inspection::distinct_subjects::grant_status::Binding, _>(
            model::causal_inspection::distinct_subjects::grant_status::Handler,
        )?;
        setup.handler::<model::causal_inspection::output_equivalence::calculation::Binding, _>(
            model::causal_inspection::output_equivalence::calculation::Handler,
        )?;
        setup.producer::<model::causal_inspection::output_equivalence::calculation::Producer>(
            model::causal_inspection::output_equivalence::calculation::Provider,
        )?;
        setup
            .conditional::<model::causal_inspection::output_equivalence::readiness::Readiness>(())?;
        setup.handler::<model::causal_inspection::retired_outputs::calculation::Binding, _>(
            model::causal_inspection::retired_outputs::calculation::Handler,
        )?;
        setup.producer::<model::causal_inspection::retired_outputs::calculation::Producer>(
            model::causal_inspection::retired_outputs::calculation::Provider,
        )?;
        setup.conditional::<model::causal_inspection::retired_outputs::readiness::Readiness>(())?;
        setup.handler::<model::deferred_completion::reciprocal::Binding, _>(
            model::deferred_completion::reciprocal::Handler,
        )?;
        setup.producer::<model::deferred_completion::reciprocal::Producer>(
            model::deferred_completion::reciprocal::Provider,
        )?;
        setup.conditional::<model::deferred_completion::readiness::Readiness>(())?;
        setup.handler::<calculation::Binding, _>(DirectoryHandler(calculation::Handler))?;
        setup.handler::<model::scalar_input::authoring::Binding, _>(
            model::scalar_input::authoring::Handler,
        )?;
        setup.handler::<model::scalar_input::calculation::Binding, _>(
            model::scalar_input::calculation::Handler,
        )?;
        setup.producer::<model::scalar_input::calculation::Producer>(
            model::scalar_input::calculation::Provider,
        )?;
        setup.conditional::<model::scalar_input::readiness::Readiness>(())?;
        setup.handler::<model::aspect_input::authoring::Binding, _>(
            model::aspect_input::authoring::Handler,
        )?;
        setup.handler::<model::aspect_input::calculation::Binding, _>(
            model::aspect_input::calculation::Handler,
        )?;
        setup.producer::<model::aspect_input::calculation::Producer>(
            model::aspect_input::calculation::Provider,
        )?;
        setup.conditional::<model::aspect_input::readiness::Readiness>(())?;
        setup.producer::<calculation::Producer>(calculation::Provider)?;
        setup.conditional::<calculation_readiness::Readiness>(())?;
        setup.handler::<model::composite_definition_model::Binding, _>(
            model::composite_definition_handler::Handler,
        )?;
        setup.handler::<edit::EditBinding, _>(DirectoryHandler(edit::Handler))?;
        setup.handler::<model::operand_edit::OperandBinding, _>(model::operand_edit::Handler)?;
        setup
            .handler::<model::observed_clone::Binding, _>(model::authored_identity_handlers::Handler)?;
        setup.handler::<model::adjacency_link::Binding, _>(model::adjacency_link::Handler)?;
        setup.handler::<model::authored_identity_model::CreateBinding, _>(
            model::authored_identity_handlers::Handler,
        )?;
        setup.handler::<model::authored_identity_model::RetireBinding, _>(
            model::authored_identity_handlers::Handler,
        )?;
        setup.handler::<model::authored_identity_model::RestoreBinding, _>(
            model::authored_identity_handlers::Handler,
        )?;
        setup.handler::<model::authored_identity_reference::Binding, _>(
            model::authored_identity_reference::Handler,
        )?;
        setup.handler::<model::source_output_model::IssueBinding, _>(
            model::source_output_handlers::IssueHandler,
        )?;
        setup.handler::<model::source_output_model::RecordBinding, _>(
            model::source_output_handlers::RecordHandler,
        )?;
        setup.handler::<model::source_output_model::RetireBinding, _>(
            model::source_output_handlers::RetireHandler,
        )?;
        setup.handler::<other_edit::OtherBinding, _>(DirectoryHandler(other_edit::Handler))?;
        setup.handler::<model::compensation_movement::CreditBinding, _>(
            model::compensation_movement::CreditHandler,
        )?;
        setup.handler::<model::compensation_movement::DebitBinding, _>(
            model::compensation_movement::DebitHandler,
        )?;
        setup.handler::<model::principal_status::Binding, _>(model::principal_status::Handler)
    }
}
