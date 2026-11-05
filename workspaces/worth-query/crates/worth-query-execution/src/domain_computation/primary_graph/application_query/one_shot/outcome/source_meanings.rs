//! Complete concrete source meanings before the caller assembles typed observations.
use std::sync::Arc;
use worth_execution::{ExecutionArray, ExecutionRequest, ExecutionRequestArrayBuilder};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryRequestInterruption, WorthQueryRequestScope,
};
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
use {
    crate::domain_computation::primary_graph::application_query::observed_source::source_identity::WorthQueryObservedSourceMeaning,
    crate::domain_computation::primary_graph::application_query::observed_source::source_identity::WorthQueryObservedSourceMeaningRegistry,
    crate::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSourceFootprint,
    crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation,
    crate::domain_computation::primary_graph::source_read_equivalence::SourceReadEquivalence,
    crate::domain_computation::vocabulary::primary_graph::application_query_basis_registry::WorthQueryApplicationBasisSelectionIdentity,
};

use {
    super::admit_request, super::denial, super::source_storage,
    crate::domain_computation::primary_graph::vocabulary::application_query::one_shot_denial::WorthQueryApplicationOneShotDenial as Denial,
    crate::domain_computation::primary_graph::vocabulary::application_query::one_shot_denial::WorthQueryApplicationOneShotDenialKind as DenialKind,
};

pub(super) struct SourceMeanings<'a, 'request, 'authority> {
    pub(super) execution: &'a ExecutionRequest<'request, 'authority>,
    pub(super) registry: &'a WorthQueryObservedSourceMeaningRegistry,
    pub(super) query: &'a [u8; 32],
    pub(super) parameters: &'a [u8; 32],
    pub(super) selection: &'a WorthQueryApplicationBasisSelectionIdentity,
    pub(super) allocation: Option<&'a Arc<SourceFactsAllocation>>,
    pub(super) request: &'a WorthQueryRequestScope,
    pub(super) query_name: &'a str,
}

impl SourceMeanings<'_, '_, '_> {
    pub(super) fn rows(
        &self,
        footprints: Vec<WorthQueryObservedSourceFootprint>,
        comparisons: &mut SourceReadEquivalence,
    ) -> Result<ExecutionArray<Arc<WorthQueryObservedSourceMeaning>>, Denial> {
        let mut meanings =
            ExecutionRequestArrayBuilder::allocate(*self.execution, footprints.len())
                .map_err(|cause| source_storage::denied(cause, self.query_name))?;
        let mut equal = |left: &_, right: &_| self.equal(left, right, comparisons);
        for footprint in footprints {
            admit_request(self.request, self.query_name)?;
            let selection = self.selection.clone();
            let meaning = if let Some(allocation) = self.allocation {
                self.registry.intern_material_compared(
                    self.query,
                    self.parameters,
                    footprint,
                    &selection,
                    &mut equal,
                    self.execution,
                    allocation,
                )
            } else {
                self.registry.intern_compared(
                    self.query,
                    self.parameters,
                    footprint,
                    &selection,
                    &mut equal,
                )
            }
            .map_err(|cause| denied(cause, self.query_name))?
            .ok_or_else(|| self.exhausted())?;
            meanings
                .push(meaning)
                .map_err(|cause| source_storage::denied(cause, self.query_name))?;
        }
        meanings
            .seal()
            .map_err(|cause| source_storage::denied(cause, self.query_name))
    }

    pub(super) fn result_set(
        &self,
        footprint: WorthQueryObservedSourceFootprint,
        comparisons: &mut SourceReadEquivalence,
    ) -> Result<Arc<WorthQueryObservedSourceMeaning>, Denial> {
        let mut equal = |left: &_, right: &_| self.equal(left, right, comparisons);
        let meaning = if let Some(allocation) = self.allocation {
            self.registry.intern_material_result_set_compared(
                self.query,
                self.parameters,
                footprint,
                self.selection,
                &mut equal,
                self.execution,
                allocation,
            )
        } else {
            self.registry.intern_result_set_compared(
                self.query,
                self.parameters,
                footprint,
                self.selection,
                &mut equal,
            )
        };
        meaning
            .map_err(|cause| denied(cause, self.query_name))?
            .ok_or_else(|| self.exhausted())
    }

    fn equal(
        &self,
        left: &WorthQueryObservedSourceFootprint,
        right: &WorthQueryObservedSourceFootprint,
        comparisons: &mut SourceReadEquivalence,
    ) -> Result<bool, Cause> {
        super::super::super::observed_source::comparison::equal(
            left,
            right,
            comparisons,
            self.execution,
            &|| match self.request.interruption() {
                None => Ok(()),
                Some(WorthQueryRequestInterruption::Cancelled) => Err(Cause::Cancelled),
                Some(WorthQueryRequestInterruption::DeadlineExceeded) => {
                    Err(Cause::DeadlineElapsed)
                }
            },
        )
    }

    fn exhausted(&self) -> Denial {
        denial(
            DenialKind::SourceIdentityExhausted,
            self.query_name,
            self.query_name,
        )
    }
}

pub(super) fn denied(cause: Cause, query: &str) -> Denial {
    crate::domain_computation::primary_graph::vocabulary::application_query::one_shot::map_authorized_read_denial(
        crate::domain_computation::primary_graph::vocabulary::application_query::authorized_read::WorthQueryAuthorizedApplicationReadDenial::Read(
            super::super::super::read_execution::relational_read_denial(cause, query),
        ),
        query,
    )
}
