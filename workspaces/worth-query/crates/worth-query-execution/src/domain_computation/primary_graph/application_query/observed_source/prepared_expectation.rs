use crate::domain_computation::primary_graph::application_query::observed_source::prepared_expectation::admission::PreparedExpectationAdmissionParentOps;
mod admission;
mod affinity;
mod pending;
pub(super) use affinity::parameter_denial;
pub use pending::WorthQueryPendingSourceExpectation;

use super::*;
use crate::domain_computation::vocabulary::authorization::operation_admission::WorthQueryOperationAdmissionIdentity;

use crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation;
use crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout;
use crate::domain_computation::primary_graph::vocabulary::application_query::observed_source_denial::WorthQuerySourceExpectationDenialKind;
use crate::domain_computation::vocabulary::primary_graph::observed_source::WorthQuerySourceExpectationKind;

struct CheckedSourceAffinity<'a> {
    layout: &'a WorthQueryPrimaryGraphLayout,
    selected_product: crate::basis::WorthQueryProductBranchReadIdentity,
    expected_query_identifier: &'static str,
}

/// Source facts prepared for exactly one admitted operation before its handler
/// or stable-output publication can consume them.
pub struct PreparedObservedSourceExpectation {
    context: Option<super::ObservedQueryExpressionContext>,
    admission_identity: WorthQueryOperationAdmissionIdentity,
    selected_product: crate::basis::WorthQueryProductBranchReadIdentity,
    observed_product: crate::basis::WorthQueryProductBranchReadIdentity,
    bound: WorthQueryBoundSourceExpectation,
    facts: Vec<WorthQueryApplicationObservedFact>,
}

/// Exact source facts for a stable output. The facts remain inside this owner
/// until the selected native read and input cutoff consume the whole binding.
pub(in crate::domain_computation::primary_graph) struct BoundStableObservedSourceFacts {
    admission_identity: WorthQueryOperationAdmissionIdentity,
    selected_product: crate::basis::WorthQueryProductBranchReadIdentity,
    observed_product: crate::basis::WorthQueryProductBranchReadIdentity,
    bound: WorthQueryBoundSourceExpectation,
    facts: Vec<WorthQueryApplicationObservedFact>,
}

impl PreparedObservedSourceExpectation {
    /// Descriptive partition coordinate for selecting the existing owner
    /// candidate before this move-only source is consumed.
    pub(in crate::domain_computation::primary_graph) const fn partition_identity(
        &self,
    ) -> [u8; 32] {
        self.bound.partition_identity
    }

    fn matches_admission<Schema, Operation, Input, Scope>(
        &self,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> bool {
        self.admission_identity == admission.admission_identity()
            && admission
                .graph_work()
                .mutation_product()
                .is_some_and(|product| {
                    self.selected_product
                        .matches_observation(product.observation())
                })
    }

    pub(in crate::domain_computation::primary_graph) fn consume_into<
        Schema,
        Operation,
        Input,
        Scope,
    >(
        self,
        admission: &mut WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<WorthQueryBoundSourceExpectation, WorthQuerySourceExpectationDenial> {
        if !self.matches_admission(admission) {
            return Err(WorthQuerySourceExpectationDenial::new(
                WorthQuerySourceExpectationDenialKind::ForeignOperation,
                "prepared source expectation",
            ));
        }
        admission.bind_source_partition(self.bound.partition_identity);
        admission.bind_source_expectation_kind(self.bound.kind);
        admission.bind_source_facts(self.facts, self.context);
        Ok(self.bound)
    }

    pub(in crate::domain_computation::primary_graph) fn consume_stable<
        Schema,
        Operation,
        Input,
        Scope,
    >(
        self,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<BoundStableObservedSourceFacts, WorthQuerySourceExpectationDenial> {
        // Stable producer cutoffs do not execute authored body requirements.
        // Never silently discard values required by that body.
        if admission
            .allowed_graph_contract()
            .body()
            .is_some_and(|body| body.requires_query_context())
        {
            return Err(WorthQuerySourceExpectationDenial::new(
                WorthQuerySourceExpectationDenialKind::SourceContractMismatch,
                "query-context body is not a stable producer cutoff",
            ));
        }
        if !self.matches_admission(admission)
            || !admission
                .graph_work()
                .mutation_product()
                .is_some_and(|product| {
                    self.observed_product
                        .matches_observation(product.observation())
                })
        {
            return Err(WorthQuerySourceExpectationDenial::new(
                WorthQuerySourceExpectationDenialKind::ForeignOperation,
                "prepared source expectation",
            ));
        }
        Ok(BoundStableObservedSourceFacts {
            admission_identity: self.admission_identity,
            selected_product: self.selected_product,
            observed_product: self.observed_product,
            bound: self.bound,
            facts: self.facts,
        })
    }
}

impl BoundStableObservedSourceFacts {
    /// Consume the exact operation binding after the publisher retained its
    /// newly funded suffix. No descriptive payload can issue this binding.
    pub(in crate::domain_computation::primary_graph) fn finish_stable<
        Schema,
        Operation,
        Input,
        Scope,
    >(
        self,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<WorthQueryBoundSourceExpectation, WorthQuerySourceExpectationDenial> {
        if !self.matches_operation(admission) {
            return Err(WorthQuerySourceExpectationDenial::new(
                WorthQuerySourceExpectationDenialKind::ForeignOperation,
                "prepared source expectation",
            ));
        }
        Ok(self.bound)
    }

    pub(in crate::domain_computation::primary_graph) fn matches_operation<
        Schema,
        Operation,
        Input,
        Scope,
    >(
        &self,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> bool {
        self.admission_identity == admission.admission_identity()
            && admission
                .graph_work()
                .mutation_product()
                .is_some_and(|product| {
                    self.selected_product
                        .matches_observation(product.observation())
                        && self
                            .observed_product
                            .matches_observation(product.observation())
                })
    }

    pub(in crate::domain_computation::primary_graph) const fn bound_source(
        &self,
    ) -> WorthQueryBoundSourceExpectation {
        self.bound
    }

    pub(in crate::domain_computation::primary_graph) fn source_facts(
        &self,
    ) -> &[WorthQueryApplicationObservedFact] {
        &self.facts
    }
}

pub trait PreparedExpectationOps<Schema>:
    crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<Schema>
where
    Schema: worth_query_installation::facade::ApplicationSchema,
{
    fn bind_application_source_expectation<Binding, Scope>(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        admission: &mut WorthQueryAdmittedApplicationOperation<
            Schema,
            Binding::Operation,
            Binding::Input,
            Scope,
        >,
        source: WorthQueryObservedSource<
            <Binding::SourceExpectation as worth_query_declaration::facade::application_operation::ApplicationMutationSourceExpectation<Schema>>::Query,
        >,
        input: &Binding::Input,
    ) -> Result<WorthQueryPendingSourceExpectation<<Binding::SourceExpectation as worth_query_declaration::facade::application_operation::ApplicationMutationSourceExpectation<Schema>>::Query>, WorthQuerySourceExpectationDenial>
    where
        Binding: worth_query_declaration::facade::application_operation::ApplicationMutationBinding<
            Schema,
        >,
{
        let identity = source.idempotency_identity().bytes();
        self.bind_checked_source_expectation::<Binding, Scope>(
            phase,
            admission,
            source,
            identity,
            input,
            WorthQuerySourceExpectationKind::Row,
        )
    }

fn bind_application_result_set_expectation<Binding, Scope>(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        admission: &mut WorthQueryAdmittedApplicationOperation<
            Schema,
            Binding::Operation,
            Binding::Input,
            Scope,
        >,
        result_set: WorthQueryObservedResultSet<
            <Binding::SourceExpectation as worth_query_declaration::facade::application_operation::ApplicationMutationSourceExpectation<Schema>>::Query,
        >,
        input: &Binding::Input,
    ) -> Result<WorthQueryPendingSourceExpectation<<Binding::SourceExpectation as worth_query_declaration::facade::application_operation::ApplicationMutationSourceExpectation<Schema>>::Query>, WorthQuerySourceExpectationDenial>
    where
        Binding: worth_query_declaration::facade::application_operation::ApplicationMutationBinding<
            Schema,
        >,
{
        let identity = result_set.idempotency_identity();
        self.bind_checked_source_expectation::<Binding, Scope>(
            phase,
            admission,
            result_set.source,
            identity,
            input,
            WorthQuerySourceExpectationKind::ResultSet,
        )
    }
}

impl<
        Schema,
        HostRuntime: crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<
                Schema,
            > + ?Sized,
    > PreparedExpectationOps<Schema> for HostRuntime
where
    Schema: worth_query_installation::facade::ApplicationSchema,
{
}

pub(in crate::domain_computation::primary_graph) trait PreparedExpectationScopedOps<Schema>:
    crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<Schema>
where
    Schema: worth_query_installation::facade::ApplicationSchema,
{
    fn prepare_application_source_expectation<Binding, Scope>(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        admission: &WorthQueryAdmittedApplicationOperation<
            Schema,
            Binding::Operation,
            Binding::Input,
            Scope,
        >,
        source: WorthQueryObservedSource<
            <Binding::SourceExpectation as worth_query_declaration::facade::application_operation::ApplicationMutationSourceExpectation<Schema>>::Query,
        >,
        input: &Binding::Input,
    ) -> Result<PreparedObservedSourceExpectation, WorthQuerySourceExpectationDenial>
    where
        Binding: worth_query_declaration::facade::application_operation::ApplicationMutationBinding<
            Schema,
        >,
    {
        self.validate_source_admission(admission, Binding::IDENTITY)?
            .prepare::<Binding>(phase, source, input)
    }
}

impl<
        Schema,
        HostRuntime: crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<
                Schema,
            > + ?Sized,
    > PreparedExpectationScopedOps<Schema> for HostRuntime
where
    Schema: worth_query_installation::facade::ApplicationSchema,
{
}

trait PreparedExpectationOwnOps<Schema>:
    crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<Schema>
where
    Schema: worth_query_installation::facade::ApplicationSchema,
{
    fn bind_checked_source_expectation<Binding, Scope>(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        admission: &mut WorthQueryAdmittedApplicationOperation<Schema, Binding::Operation, Binding::Input, Scope>,
        source: WorthQueryObservedSource<<Binding::SourceExpectation as worth_query_declaration::facade::application_operation::ApplicationMutationSourceExpectation<Schema>>::Query>,
        identity: [u8; 32],
        input: &Binding::Input,
        kind: WorthQuerySourceExpectationKind,
    ) -> Result<WorthQueryPendingSourceExpectation<<Binding::SourceExpectation as worth_query_declaration::facade::application_operation::ApplicationMutationSourceExpectation<Schema>>::Query>, WorthQuerySourceExpectationDenial>
    where Binding: worth_query_declaration::facade::application_operation::ApplicationMutationBinding<Schema>,
{
        self.readmit_mutable_source_admission(admission, Binding::IDENTITY)?
            .bind::<Binding>(phase, source, identity, input, kind)
    }
}

impl<
        Schema,
        HostRuntime: crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<
                Schema,
            > + ?Sized,
    > PreparedExpectationOwnOps<Schema> for HostRuntime
where
    Schema: worth_query_installation::facade::ApplicationSchema,
{
}

pub(crate) mod court_doors_for_test;
