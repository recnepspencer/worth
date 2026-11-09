//! Exact mutation-selector comparison against admitted parameters.
use super::*;

impl WorthQueryAdmittedApplicationQueryParameters {
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
        let Some(prior) = binding_shape(self.bindings.iter().map(|(n, v)| (n.as_ref(), v))) else {
            return Ok(Err(refuse_overflow()));
        };
        let Some(next) = binding_shape(expected.bindings().iter().map(|(n, v)| (*n, v))) else {
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
        let same_initialized_bindings = self.bindings.len() == expected.bindings().len()
            && self.bindings.iter().zip(expected.bindings()).all(
                |((name, value), (expected_name, expected_value))| {
                    name.as_ref() == *expected_name && value == expected_value
                },
            );
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

fn binding_shape<'a>(
    mut bindings: impl Iterator<Item = (&'a str, &'a AspectValue)>,
) -> Option<(usize, usize, usize)> {
    bindings.try_fold(
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
