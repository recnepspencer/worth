use std::borrow::Cow;
mod layout_validation;
mod selector_comparison;
use layout_validation::validate_parameter_layout;
mod preparation;
mod source_request;
pub use preparation::{
    admit_application_query_parameters_admitted, WorthQueryParameterPreparationStop,
};
pub use source_request::WorthQueryCanonicalSourceRequest;
use worth_foundational::facade::{AspectValue, CanonicalDigestDerivationDenial, CanonicalDigestId};
use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;
use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;

use super::{
    parameter_canonical_basis::prepare_parameter_basis,
    WorthQueryApplicationParameterCanonicalArtifact,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationQueryParameterDenialKind {
    ParameterSetMismatch,
    ParameterTypeMismatch,
    CanonicalEntryBudgetExceeded,
    CanonicalEncodedByteBudgetExceeded,
    CanonicalDigestSlotRejected,
    /// Canonical preparation refused the actual compiler/target's sort qualification.
    UnsupportedSortImplementation(worth_foundational::facade::CanonicalSortImplementationDenial),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationQueryParameterDenial {
    kind: WorthQueryApplicationQueryParameterDenialKind,
    parameter: String,
}

impl WorthQueryApplicationQueryParameterDenial {
    fn new(
        kind: WorthQueryApplicationQueryParameterDenialKind,
        parameter: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            parameter: parameter.into(),
        }
    }

    pub const fn kind(&self) -> WorthQueryApplicationQueryParameterDenialKind {
        self.kind
    }

    pub fn parameter(&self) -> &str {
        &self.parameter
    }

    pub fn owned_allocation_capacity_bytes(&self) -> usize {
        self.parameter.capacity()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryAdmittedApplicationQueryParameters {
    canonical: WorthQueryApplicationParameterCanonicalArtifact,
    bindings: Vec<(Cow<'static, str>, AspectValue)>,
    budget: worth_foundational::facade::CanonicalDigestWorkBudget,
}

impl WorthQueryAdmittedApplicationQueryParameters {
    /// Exclusive backing retained when this admitted basis moves into an
    /// observed source. The caller prices the shared Arc header separately.
    pub fn retained_owned_capacity_bytes(&self) -> Option<usize> {
        let slots = self
            .bindings
            .capacity()
            .checked_mul(std::mem::size_of::<(Cow<'static, str>, AspectValue)>())?;
        self.bindings
            .iter()
            .try_fold(
                self.canonical
                    .basis()
                    .payload()
                    .owned_allocation_capacity_bytes(),
                |total, (name, value)| {
                    total
                        .checked_add(owned_name_capacity(name))?
                        .checked_add(value.owned_allocation_capacity_bytes())
                },
            )?
            .checked_add(slots)
    }

    /// The same immutable bindings and canonical rule are reissued under the
    /// current installation. Price the new basis, its digest-time clone and
    /// encoded material alongside the newly owned binding row before either
    /// canonical construction or value cloning begins.
    pub fn readmission_preparation_requirements(&self) -> Option<(u64, u64)> {
        let work = self.canonical.work();
        let basis_bytes = self
            .canonical
            .basis()
            .payload()
            .owned_allocation_capacity_bytes();
        let binding_slots = self
            .bindings
            .len()
            .checked_mul(std::mem::size_of::<(Cow<'static, str>, AspectValue)>())?;
        let binding_heap = self
            .bindings
            .iter()
            .try_fold(0usize, |total, (name, value)| {
                total
                    .checked_add(owned_name_capacity(name))?
                    .checked_add(value.owned_allocation_capacity_bytes())
            })?;
        let bytes = basis_bytes
            .checked_mul(3)?
            .checked_add(work.canonical_material_allocation_bytes())?
            .checked_add(binding_slots)?
            .checked_add(binding_heap)?;
        let copied_values = self
            .bindings
            .iter()
            .try_fold(0usize, |total, (name, value)| {
                total
                    .checked_add(name.len())?
                    .checked_add(value.semantic_byte_width())
            })?;
        // The owner row is sorted. An installed declaration can appear in any
        // order, so each layout check uses one bounded binary search.
        let lookup_steps = usize::BITS as usize - self.bindings.len().leading_zeros() as usize + 1;
        let longest_name = self
            .bindings
            .iter()
            .map(|(name, _)| name.len())
            .max()
            .unwrap_or(0);
        let layout_work = self
            .bindings
            .len()
            .checked_mul(lookup_steps)?
            .checked_mul(longest_name.checked_add(1)?)?;
        let units = usize::try_from(work.canonical_entries())
            .ok()?
            .checked_add(work.canonical_encoded_bytes())?
            .checked_add(work.sha256_input_bytes())?
            .checked_add(work.sha256_compression_blocks())?
            .checked_add(copied_values.checked_mul(2)?)?
            .checked_add(layout_work)?
            .checked_add(self.bindings.len())?;
        Some((u64::try_from(bytes).ok()?, u64::try_from(units).ok()?))
    }

    pub const fn identity(&self) -> &CanonicalDigestId {
        self.canonical.identity()
    }

    pub fn canonical_basis(&self) -> &WorthQueryApplicationParameterCanonicalArtifact {
        &self.canonical
    }

    pub fn bindings(&self) -> &[(Cow<'static, str>, AspectValue)] {
        &self.bindings
    }
}

pub fn admit_application_query_parameters<Schema, Query, Parameters, QueryResult, Scope>(
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    parameters: ApplicationQueryParameterSet<Query>,
) -> Result<WorthQueryAdmittedApplicationQueryParameters, WorthQueryApplicationQueryParameterDenial>
{
    let bindings = parameters
        .bindings()
        .iter()
        .map(|(name, value)| (Cow::Borrowed(*name), value.clone()))
        .collect();
    admit_bindings(query, bindings)
}

/// Re-admit captured parameter DATA. No prior digest, authority or application
/// projector participates; the current installed layout and policy decide.
pub fn readmit_recorded_application_query_parameters<
    Schema,
    Query,
    Parameters,
    QueryResult,
    Scope,
>(
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    recorded: Vec<(String, AspectValue)>,
) -> Result<WorthQueryAdmittedApplicationQueryParameters, WorthQueryApplicationQueryParameterDenial>
{
    let bindings = recorded
        .into_iter()
        .map(|(name, value)| (Cow::Owned(name), value))
        .collect();
    admit_bindings(query, bindings)
}

fn admit_bindings<Schema, Query, Parameters, QueryResult, Scope>(
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    mut bindings: Vec<(Cow<'static, str>, AspectValue)>,
) -> Result<WorthQueryAdmittedApplicationQueryParameters, WorthQueryApplicationQueryParameterDenial>
{
    bindings.sort_unstable_by(|(left, _), (right, _)| left.cmp(right));
    validate_parameter_layout(query, &bindings)?;
    let canonical = prepare_parameter_basis(&bindings, query.canonical_work_policy().parameters())
        .map_err(|denial| canonical_work_denial(query.name(), denial))?;
    Ok(WorthQueryAdmittedApplicationQueryParameters {
        canonical,
        bindings,
        budget: query.canonical_work_policy().parameters(),
    })
}

/// Re-admit retained descriptive values against the currently installed query.
/// The previous digest and canonical budget grant no authority to this read.
pub fn readmit_application_query_parameters<Schema, Query, Parameters, QueryResult, Scope>(
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    retained: &WorthQueryAdmittedApplicationQueryParameters,
) -> Result<WorthQueryAdmittedApplicationQueryParameters, WorthQueryApplicationQueryParameterDenial>
{
    validate_parameter_layout(query, &retained.bindings)?;
    let budget = query.canonical_work_policy().parameters();
    let canonical = prepare_parameter_basis(&retained.bindings, budget)
        .map_err(|denial| canonical_work_denial(query.name(), denial))?;
    // Canonical admission bounds the initialized value bytes; the installed
    // declaration bounds the number of slots copied into this fresh plan.
    Ok(WorthQueryAdmittedApplicationQueryParameters {
        canonical,
        bindings: retained.bindings.clone(),
        budget,
    })
}

fn canonical_work_denial(
    parameter: &str,
    denial: CanonicalDigestDerivationDenial,
) -> WorthQueryApplicationQueryParameterDenial {
    let kind = match denial {
        CanonicalDigestDerivationDenial::EntryLimitExceeded { .. } => {
            WorthQueryApplicationQueryParameterDenialKind::CanonicalEntryBudgetExceeded
        }
        CanonicalDigestDerivationDenial::EncodedByteLimitExceeded { .. } => {
            WorthQueryApplicationQueryParameterDenialKind::CanonicalEncodedByteBudgetExceeded
        }
        _ => WorthQueryApplicationQueryParameterDenialKind::CanonicalDigestSlotRejected,
    };
    WorthQueryApplicationQueryParameterDenial::new(kind, parameter)
}

fn owned_name_capacity(name: &Cow<'static, str>) -> usize {
    match name {
        Cow::Borrowed(_) => 0,
        Cow::Owned(name) => name.capacity(),
    }
}
