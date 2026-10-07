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
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryAdmittedApplicationQueryParameters {
    canonical: WorthQueryApplicationParameterCanonicalArtifact,
    bindings: Vec<(&'static str, AspectValue)>,
    budget: worth_foundational::facade::CanonicalDigestWorkBudget,
}

impl WorthQueryAdmittedApplicationQueryParameters {
    /// Exclusive backing retained when this admitted basis moves into an
    /// observed source. The caller prices the shared Arc header separately.
    pub fn retained_owned_capacity_bytes(&self) -> Option<usize> {
        let slots = self
            .bindings
            .capacity()
            .checked_mul(std::mem::size_of::<(&'static str, AspectValue)>())?;
        self.bindings
            .iter()
            .try_fold(
                self.canonical
                    .basis()
                    .payload()
                    .owned_allocation_capacity_bytes(),
                |total, (_, value)| total.checked_add(value.owned_allocation_capacity_bytes()),
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
            .checked_mul(std::mem::size_of::<(&'static str, AspectValue)>())?;
        let binding_heap = self.bindings.iter().try_fold(0usize, |total, (_, value)| {
            total.checked_add(value.owned_allocation_capacity_bytes())
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

    pub fn bindings(&self) -> &[(&'static str, AspectValue)] {
        &self.bindings
    }

    /// Compares a mutation's declared selectors with this admitted query's exact
    /// canonical basis, under the same installed canonicalization budget.
    pub fn matches_expected<Query>(
        &self,
        expected: ApplicationQueryParameterSet<Query>,
    ) -> Result<bool, WorthQueryApplicationQueryParameterDenial> {
        let mut bindings = expected.bindings().to_vec();
        bindings.sort_by_key(|(name, _)| *name);
        let canonical = prepare_parameter_basis(&bindings, self.budget)
            .map_err(|denial| canonical_work_denial("mutation-source", denial))?;
        Ok(self.canonical.is_equivalent_to(&canonical))
    }

    /// Admits the library-owned selector comparison before it clones bindings
    /// or constructs another canonical basis. The declaration callback which
    /// produces `expected` remains the declaration's own pre-effect work.
    pub fn matches_expected_with_preflight<Query, Stop>(
        &self,
        expected: ApplicationQueryParameterSet<Query>,
        mut admit: impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<Result<bool, WorthQueryApplicationQueryParameterDenial>, Stop> {
        let refuse_overflow = || {
            WorthQueryApplicationQueryParameterDenial::new(
                WorthQueryApplicationQueryParameterDenialKind::CanonicalEncodedByteBudgetExceeded,
                "mutation-source",
            )
        };
        let Some(walk) = self
            .bindings
            .len()
            .checked_add(expected.bindings().len())
            .and_then(|count| u64::try_from(count).ok())
        else {
            return Ok(Err(refuse_overflow()));
        };
        admit(walk, 0)?;
        let shape = |bindings: &[(&'static str, AspectValue)]| {
            bindings.iter().try_fold(
                (0usize, 0usize, 0usize),
                |(owned, semantic, longest), (name, value)| {
                    let owned = owned
                        .checked_add(value.owned_allocation_capacity_bytes())?
                        .checked_add(name.len())?
                        .checked_add(value.value_family().canonical_name().len())?;
                    let semantic = semantic
                        .checked_add(value.semantic_byte_width())?
                        .checked_add(name.len())?
                        .checked_add(value.value_family().canonical_name().len())?;
                    Some((owned, semantic, longest.max(name.len())))
                },
            )
        };
        let Some(prior) = shape(&self.bindings) else {
            return Ok(Err(refuse_overflow()));
        };
        let Some(next) = shape(expected.bindings()) else {
            return Ok(Err(refuse_overflow()));
        };
        let Some(comparison_work) = prior
            .1
            .checked_add(next.1)
            .and_then(|width| width.checked_add(self.bindings.len()))
            .and_then(|width| width.checked_add(expected.bindings().len()))
            .and_then(|width| u64::try_from(width).ok())
        else {
            return Ok(Err(refuse_overflow()));
        };
        admit(comparison_work, 0)?;
        let basis = self.canonical.basis().payload();
        let Some(basis_rows) = u64::try_from(basis.entries().len()).ok() else {
            return Ok(Err(refuse_overflow()));
        };
        admit(basis_rows, 0)?;
        let Some(basis_copy_and_compare) = basis_copy_and_comparison_visits(basis) else {
            return Ok(Err(refuse_overflow()));
        };
        // Equality classifies only the required scratch. The canonical
        // comparator below remains the sole acceptance authority.
        let same_initialized_bindings = self.bindings.as_slice() == expected.bindings();
        let requirements = if same_initialized_bindings {
            self.readmission_preparation_requirements()
        } else {
            let entries = expected
                .bindings()
                .len()
                .checked_mul(3)
                .and_then(|count| count.checked_add(1));
            let Some(entries) = entries else {
                return Ok(Err(refuse_overflow()));
            };
            if entries > self.budget.maximum_entry_count() as usize {
                return Ok(Err(WorthQueryApplicationQueryParameterDenial::new(
                    WorthQueryApplicationQueryParameterDenialKind::CanonicalEntryBudgetExceeded,
                    "mutation-source",
                )));
            }
            let encoded = self.budget.maximum_encoded_bytes();
            let prior_basis = self
                .canonical
                .basis()
                .payload()
                .owned_allocation_capacity_bytes();
            let entry_bytes = entries.checked_mul(std::mem::size_of::<
                worth_foundational::facade::CanonicalBasisEntry,
            >());
            let slot_bytes = expected
                .bindings()
                .len()
                .checked_mul(std::mem::size_of::<(&'static str, AspectValue)>());
            let bytes = encoded
                .checked_mul(4)
                .and_then(|n| n.checked_add(next.0.checked_mul(4)?))
                .and_then(|n| n.checked_add(prior_basis.checked_mul(2)?))
                .and_then(|n| n.checked_add(entry_bytes?.checked_mul(2)?))
                .and_then(|n| n.checked_add(slot_bytes?));
            let work = encoded
                .checked_mul(3)
                .and_then(|n| n.checked_add(next.1.checked_mul(2)?))
                .and_then(|n| n.checked_add(entries.checked_mul(8)?));
            bytes.zip(work).and_then(|(bytes, work)| {
                Some((u64::try_from(bytes).ok()?, u64::try_from(work).ok()?))
            })
        };
        let Some((bytes, work)) = requirements else {
            return Ok(Err(refuse_overflow()));
        };
        let count = expected.bindings().len();
        let sort_steps = (usize::BITS - count.max(1).leading_zeros()) as usize;
        let Some(sort_work) = count
            .checked_mul(sort_steps)
            .and_then(|steps| steps.checked_mul(next.2.checked_add(1)?))
            .and_then(|steps| u64::try_from(steps).ok())
        else {
            return Ok(Err(refuse_overflow()));
        };
        let Some(work) = work
            .checked_add(sort_work)
            .and_then(|work| work.checked_add(basis_copy_and_compare))
        else {
            return Ok(Err(refuse_overflow()));
        };
        admit(work, bytes)?;
        Ok(self.matches_expected(expected))
    }
}

pub fn admit_application_query_parameters<Schema, Query, Parameters, QueryResult, Scope>(
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    parameters: ApplicationQueryParameterSet<Query>,
) -> Result<WorthQueryAdmittedApplicationQueryParameters, WorthQueryApplicationQueryParameterDenial>
{
    let mut bindings = parameters.bindings().to_vec();
    bindings.sort_by_key(|(name, _)| *name);
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

fn basis_copy_and_comparison_visits(
    basis: &worth_foundational::facade::CanonicalBasisSequence,
) -> Option<u64> {
    use worth_foundational::facade::{CanonicalBasisLocus, CanonicalBasisValue, InternedString};
    let text = |value: &InternedString| match value {
        InternedString::Raw(value) => value.len(),
        InternedString::Symbol(_) => 1,
    };
    let mut initialized = basis.version().as_str().len();
    for entry in basis.entries() {
        initialized = initialized.checked_add(1)?;
        let CanonicalBasisLocus::Named(name) = entry.locus() else {
            unreachable!("application parameters use named canonical entries")
        };
        initialized = initialized.checked_add(text(name))?;
        let value_visits = match entry.value() {
            CanonicalBasisValue::ExactText(value)
            | CanonicalBasisValue::DecimalText(value)
            | CanonicalBasisValue::BigIntText(value) => text(value),
            CanonicalBasisValue::RationalText {
                numerator,
                denominator,
            } => text(numerator).checked_add(text(denominator))?,
            _ => 1,
        };
        initialized = initialized.checked_add(value_visits)?;
    }
    // The comparator clones both bases, then visits the selected entries.
    u64::try_from(initialized.checked_mul(3)?).ok()
}

fn validate_parameter_layout<Schema, Query, Parameters, QueryResult, Scope>(
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    bindings: &[(&'static str, AspectValue)],
) -> Result<(), WorthQueryApplicationQueryParameterDenial> {
    if bindings.len() != query.parameters().len() {
        return Err(WorthQueryApplicationQueryParameterDenial::new(
            WorthQueryApplicationQueryParameterDenialKind::ParameterSetMismatch,
            query.name(),
        ));
    }
    for declared in query.parameters() {
        let Some((_, value)) = bindings
            .binary_search_by(|(name, _)| name.cmp(&declared.name()))
            .ok()
            .and_then(|index| bindings.get(index))
        else {
            return Err(WorthQueryApplicationQueryParameterDenial::new(
                WorthQueryApplicationQueryParameterDenialKind::ParameterSetMismatch,
                declared.name(),
            ));
        };
        if value.value_family() != declared.scalar_family() {
            return Err(WorthQueryApplicationQueryParameterDenial::new(
                WorthQueryApplicationQueryParameterDenialKind::ParameterTypeMismatch,
                declared.name(),
            ));
        }
    }
    Ok(())
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
