//! Original complete query consumption, retained without exposing native facts.
use super::*;
use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind;
use crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation;
use crate::publication_boundary::WorthQueryProgramPublicationAccess;
use worth_execution::ExecutionArray;
use worth_execution::ExecutionArrayBuilder;
use worth_query_installation::facade::ApplicationSchema;

mod comparison;
mod coverage;
use coverage::RetainedQuerySource;

#[cfg(test)]
mod storage_court;

/// Move-only native ownership of all original set and row dependency descriptors.
/// Variable payloads share their original funded backing; this array owns its
/// separately funded descriptor slots. No consumer can construct a subset.
///
/// Capture borrows the complete native pairing and requires no row cloning:
/// ```no_run
/// use worth_query_execution::{publication_boundary::{WorthQueryProgramPublicationAccess,
///     discovery_currentness::{WorthQueryRetainedQueryDependencies, WorthQueryRequiredSetCurrentnessDenial}},
///     facade::primary_graph::{WorthQuerySelectedProductOperation, WorthQueryApplicationOutputDemandSource},
///     facade::application_contribution::WorthQueryAdvancementPhase};
/// use worth_query_installation::facade::ApplicationSchema;
/// fn retain<S: ApplicationSchema, Q, V>(selected: &WorthQuerySelectedProductOperation<'_, S>,
///     phase: &WorthQueryAdvancementPhase<'_>, access: &WorthQueryProgramPublicationAccess,
///     original: &WorthQueryApplicationOutputDemandSource<Q,V>)
///     -> Result<WorthQueryRetainedQueryDependencies, WorthQueryRequiredSetCurrentnessDenial> {
///     selected.retain_query_dependencies(phase, access, original)
/// }
/// ```
/// ```compile_fail,E0599
/// use worth_query_execution::publication_boundary::discovery_currentness::WorthQueryRetainedQueryDependencies;
/// fn duplicate(original: WorthQueryRetainedQueryDependencies) -> WorthQueryRetainedQueryDependencies {
///     original.clone()
/// }
/// ```
pub struct WorthQueryRetainedQueryDependencies {
    sources: ExecutionArray<RetainedQuerySource>,
}

/// Native causes remain distinct from a moved output or discovery fact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryRequiredSetCurrentnessDenial {
    Source(WorthQuerySourceExpectationDenial),
    Output(WorthQueryOutputDemandDenial),
}
impl From<WorthQuerySourceExpectationDenial> for WorthQueryRequiredSetCurrentnessDenial {
    fn from(value: WorthQuerySourceExpectationDenial) -> Self {
        Self::Source(value)
    }
}
impl From<WorthQueryOutputDemandDenial> for WorthQueryRequiredSetCurrentnessDenial {
    fn from(value: WorthQueryOutputDemandDenial) -> Self {
        Self::Output(value)
    }
}
fn storage(
    cause: worth_execution::ExecutionAllocationDenial,
) -> WorthQueryRequiredSetCurrentnessDenial {
    WorthQuerySourceExpectationDenial::source_retention_denied(
        "original discovery dependencies",
        cause.into(),
    )
    .into()
}
fn source_denial(
    kind: WorthQuerySourceExpectationDenialKind,
) -> WorthQueryRequiredSetCurrentnessDenial {
    WorthQuerySourceExpectationDenial::new(kind, "original discovery dependencies").into()
}
fn execution_denial(
    cause: crate::domain_computation::primary_graph::WorthQueryAdvancementDenial,
) -> WorthQueryRequiredSetCurrentnessDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::ExecutionRequest(cause),
        "original discovery dependencies",
    )
    .into()
}
impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema> {
    /// Retains the complete sealed pairing, never a caller-selected row subset.
    /// All original owners remain borrowed until allocation, pushes and seal succeed.
    pub fn retain_query_dependencies<Query, Value>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _: &WorthQueryProgramPublicationAccess,
        source: &WorthQueryApplicationOutputDemandSource<Query, Value>,
    ) -> Result<WorthQueryRetainedQueryDependencies, WorthQueryRequiredSetCurrentnessDenial> {
        let execution = phase
            .checked_execution_request_for(&self.application().product_runtime)
            .map_err(execution_denial)?;
        let set = source.result_set_observation().ok_or_else(|| {
            source_denial(WorthQuerySourceExpectationDenialKind::MissingExpectation)
        })?;
        if source.rows().len() != source.observed_sources().len()
            || !set.source().has_complete_output_dependencies()
        {
            return Err(source_denial(
                WorthQuerySourceExpectationDenialKind::IncompleteFootprint,
            ));
        }
        let count = source
            .observed_sources()
            .len()
            .checked_add(1)
            .ok_or_else(|| {
                source_denial(WorthQuerySourceExpectationDenialKind::PreparationMemoryExceeded)
            })?;

        for observed in std::iter::once(set.source()).chain(source.observed_sources()) {
            check_request(&execution)?;

            self.validate_dependency(observed, true)?;
            let original = set.source();
            if observed.query_identifier != original.query_identifier
                || observed.query_identity != original.query_identity
                || observed.parameter_binding_identity != original.parameter_binding_identity
                || observed.model_root != original.model_root
                || !std::sync::Arc::ptr_eq(&observed.parameters, &original.parameters)
                || !std::sync::Arc::ptr_eq(&observed.scope_selector, &original.scope_selector)
            {
                return Err(source_denial(
                    WorthQuerySourceExpectationDenialKind::SourceContractMismatch,
                ));
            }
        }
        self.application()
            .primary_provider
            .graph
            .with_runtime(&execution, |relational| {
                self.retain_original_dependencies(
                    &execution,
                    relational,
                    std::iter::once(set.source()).chain(source.observed_sources()),
                    count,
                )
            })
    }

    /// Joins retained complete proofs while every original remains owned by its
    /// continuation. Refusal drops only the attempted shared descriptors.
    pub fn combine_query_dependencies<'a, I>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _: &WorthQueryProgramPublicationAccess,
        parts: I,
    ) -> Result<WorthQueryRetainedQueryDependencies, WorthQueryRequiredSetCurrentnessDenial>
    where
        I: Iterator<Item = &'a WorthQueryRetainedQueryDependencies> + Clone,
    {
        let execution = phase
            .checked_execution_request_for(&self.application().product_runtime)
            .map_err(execution_denial)?;

        let count = parts.clone().try_fold(0usize, |count, part| {
            check_request(&execution)?;

            count.checked_add(part.sources.len()).ok_or_else(|| {
                source_denial(WorthQuerySourceExpectationDenialKind::PreparationMemoryExceeded)
            })
        })?;
        for part in parts.clone() {
            check_request(&execution)?;
            for source in part.sources.elements() {
                execution
                    .allocation_policy()
                    .check_live()
                    .map_err(storage)?;

                self.validate_dependency(&source.source, false)?;
            }
        }
        // Backing traversal checks the request at each part, like count and
        // validation, so a stop between groups never pulls the next group.
        retain_descriptors(parts.map(|part| part.sources.elements()), count, &execution)
    }

    fn validate_dependency<Query>(
        &self,
        source: &WorthQueryObservedSource<Query>,
        exact: bool,
    ) -> Result<(), WorthQueryRequiredSetCurrentnessDenial> {
        let runtime = self.application();
        let selected = crate::basis::WorthQueryProductBranchReadIdentity::from_observation(
            self.product().observation(),
        );
        // The sealed descriptor was issued under this immutable installation.
        // Capture checked the full pairing; merge/currentness carry that exact
        // query and parameter identity, including installed certification queries.
        source.validate_affinity(
            runtime.runtime.authority_identity().as_u64(),
            &runtime.installed_schema.binding_identity(),
            self.application_basis().identity().branch_id(),
            source.model_root,
            &selected,
            &source.query_identifier,
            &source.query_identity,
        )?;
        source.validate_completeness(&source.query_identifier)?;
        if exact
            && !matches!(&source.selection, WorthQueryApplicationBasisSelectionIdentity::Product(original) if original == &selected)
        {
            return Err(source_denial(
                WorthQuerySourceExpectationDenialKind::ForeignOperation,
            ));
        }
        Ok(())
    }
}

// Only validated native descriptors reach this storage owner. Original complete
// owners stay borrowed throughout the exact allocation and each fallible move.
fn retain_descriptors<'a, Part>(
    parts: impl IntoIterator<Item = Part>,
    count: usize,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<WorthQueryRetainedQueryDependencies, WorthQueryRequiredSetCurrentnessDenial>
where
    Part: IntoIterator<Item = &'a RetainedQuerySource>,
{
    check_request(execution)?;
    let policy = execution.allocation_policy();
    let mut array = ExecutionArrayBuilder::allocate(count, policy).map_err(storage)?;
    for part in parts {
        check_request(execution)?;
        for source in part {
            check_request(execution)?;
            array.push(source.shared()).map_err(storage)?;
        }
    }
    check_request(execution)?;
    Ok(WorthQueryRetainedQueryDependencies {
        sources: array.seal().map_err(storage)?,
    })
}

fn check_request(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<(), WorthQueryRequiredSetCurrentnessDenial> {
    use crate::domain_computation::primary_graph::application_contribution::execution_denial::advancement_checkpoint_stop;
    execution
        .check_live()
        .map_err(|cause| execution_denial(advancement_checkpoint_stop(cause)))
}
