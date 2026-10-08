mod affinity;
mod pending;
use affinity::parameter_denial;
pub use pending::WorthQueryPendingSourceExpectation;

use super::*;
use crate::domain_computation::authorization::WorthQueryOperationAdmissionIdentity;
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQueryApplicationObservedFact,
    WorthQueryPrimaryGraphLayout,
};

struct CheckedSourceAffinity<'a> {
    layout: &'a WorthQueryPrimaryGraphLayout,
    selected_product: crate::basis::WorthQueryProductBranchReadIdentity,
    expected_query_identifier: &'static str,
}

/// Source facts prepared for exactly one admitted operation before its handler
/// or stable-output publication can consume them.
pub(in crate::domain_computation::primary_graph) struct PreparedObservedSourceExpectation {
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
        admission.bind_source_facts(self.facts);
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
    /// Moves only the checked source suffix into a caller-prepared fact buffer.
    /// The caller retains its sealed cutoff and owns the final admission.
    pub(in crate::domain_computation::primary_graph) fn append_into_stable_facts<
        Schema,
        Operation,
        Input,
        Scope,
    >(
        mut self,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        target: &mut Vec<WorthQueryApplicationObservedFact>,
    ) -> Result<WorthQueryBoundSourceExpectation, WorthQuerySourceExpectationDenial> {
        if !self.matches_operation(admission) {
            return Err(WorthQuerySourceExpectationDenial::new(
                WorthQuerySourceExpectationDenialKind::ForeignOperation,
                "prepared source expectation",
            ));
        }
        if target.capacity().saturating_sub(target.len()) < self.facts.len() {
            return Err(WorthQuerySourceExpectationDenial::new(
                WorthQuerySourceExpectationDenialKind::PreparationMemoryExceeded,
                "prepared source expectation",
            ));
        }
        target.append(&mut self.facts);
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

impl<Schema>
    crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: worth_query_installation::facade::ApplicationSchema,
{
    pub fn bind_application_source_expectation<Binding, Scope>(
        &self,
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
        self.bind_checked_source_expectation::<Binding, Scope>(admission, source, identity, input)
    }

    pub fn bind_application_result_set_expectation<Binding, Scope>(
        &self,
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
            admission,
            result_set.source,
            identity,
            input,
        )
    }

    /// The installed producer calls this before handler contact. Both fresh
    /// execution and stable output use its one metered fact construction.
    pub(in crate::domain_computation::primary_graph) fn prepare_application_source_expectation<
        Binding,
        Scope,
    >(
        &self,
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
        request_admission: &mut InvalidationEditAdmission,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<PreparedObservedSourceExpectation, WorthQuerySourceExpectationDenial>
    where
        Binding: worth_query_declaration::facade::application_operation::ApplicationMutationBinding<
            Schema,
        >,
    {
        let identity = source.idempotency_identity().bytes();
        let checked = self.checked_source_affinity::<Binding, Scope>(
            admission,
            &source,
            input,
            |parameters, expected| {
                parameters
                    .matches_expected_with_preflight(expected, |work, bytes| {
                        request_admission.charge_external_work(work).map_err(|_| {
                            WorthQuerySourceExpectationDenial::new(
                                WorthQuerySourceExpectationDenialKind::WorkBudgetExceeded,
                                Binding::IDENTITY,
                            )
                        })?;
                        request_admission.admit_read_scratch(bytes).map_err(|_| {
                            WorthQuerySourceExpectationDenial::new(
                                WorthQuerySourceExpectationDenialKind::PreparationMemoryExceeded,
                                Binding::IDENTITY,
                            )
                        })
                    })?
                    .map_err(|denial| parameter_denial(Binding::IDENTITY, denial))
            },
        )?;
        let prepared_facts = source.admit_fact_materialization(
            checked.layout,
            request_admission,
            Binding::IDENTITY,
        )?;
        let partition_identity = source.partition_identity();
        let allocation_control = crate::domain_computation::primary_graph::request_allocation_control::RequestAllocationControl::new(admission.publication_request(), allocation_policy);
        let facts = prepared_facts.into_facts(crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl::new(allocation_control.policy(), Some(admission.publication_request())))?;
        let super::WorthQueryApplicationBasisSelectionIdentity::Product(observed_product) =
            source.selection
        else {
            unreachable!("validated product source retains its selected observation")
        };
        Ok(PreparedObservedSourceExpectation {
            admission_identity: admission.admission_identity(),
            selected_product: checked.selected_product,
            observed_product,
            bound: WorthQueryBoundSourceExpectation {
                identity,
                partition_identity,
            },
            facts,
        })
    }

    fn bind_checked_source_expectation<Binding, Scope>(
        &self,
        admission: &mut WorthQueryAdmittedApplicationOperation<Schema, Binding::Operation, Binding::Input, Scope>,
        source: WorthQueryObservedSource<<Binding::SourceExpectation as worth_query_declaration::facade::application_operation::ApplicationMutationSourceExpectation<Schema>>::Query>,
        identity: [u8; 32],
        input: &Binding::Input,
    ) -> Result<WorthQueryPendingSourceExpectation<<Binding::SourceExpectation as worth_query_declaration::facade::application_operation::ApplicationMutationSourceExpectation<Schema>>::Query>, WorthQuerySourceExpectationDenial>
    where Binding: worth_query_declaration::facade::application_operation::ApplicationMutationBinding<Schema>,
    {
        let checked = self.checked_source_affinity::<Binding, Scope>(
            admission,
            &source,
            input,
            |parameters, expected| {
                parameters
                    .matches_expected(expected)
                    .map_err(|denial| parameter_denial(Binding::IDENTITY, denial))
            },
        )?;
        source.validate_fact_retention(
            checked.layout,
            checked.expected_query_identifier,
            Some(admission.publication_request()),
        )?;
        let bound = WorthQueryBoundSourceExpectation {
            identity,
            partition_identity: source.partition_identity(),
        };
        admission.bind_source_partition(bound.partition_identity);
        Ok(WorthQueryPendingSourceExpectation {
            admission_identity: admission.admission_identity(),
            selected_product: checked.selected_product,
            expected_query_identifier: checked.expected_query_identifier,
            bound,
            source,
        })
    }
}
