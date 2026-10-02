use crate::canonicalization::basis::{
    CanonicalBasisEntry, CanonicalBasisLocus, CanonicalBasisValue,
};

#[derive(Default)]
pub(super) struct EntryComparisonWidth {
    pub entry: usize,
    pub domain: usize,
    pub locus: usize,
    pub duplicate_clone_work: usize,
    pub duplicate_clone_bytes: usize,
}

pub(super) fn measure<Stop>(
    entries: &[CanonicalBasisEntry],
    admit: &mut impl FnMut(usize, usize) -> Result<(), Stop>,
) -> Result<EntryComparisonWidth, super::CanonicalBasisPreparationStop<Stop>> {
    use super::CanonicalBasisPreparationStop::{AccountingOverflow, Resource};
    let mut width = EntryComparisonWidth::default();
    for entry in entries {
        // Pay each measurement visit before inspecting the entry. String
        // lengths read initialized headers; they do not scan string contents.
        admit(8, 0).map_err(Resource)?;
        let domain = match entry.domain() {
            crate::canonicalization::basis::CanonicalBasisDomain::Future(name) => name.len(),
            _ => 0,
        };
        let (locus, clone_bytes, clone_work) = match entry.locus() {
            CanonicalBasisLocus::Root | CanonicalBasisLocus::EntryOrdinal(_) => (1, 0, 1),
            CanonicalBasisLocus::Aspect(aspect) => {
                (aspect.as_str().len(), aspect.as_str().len(), 1)
            }
            CanonicalBasisLocus::Named(name) => {
                let (comparison, allocation) = interned_width(name);
                (comparison, allocation, 1)
            }
            CanonicalBasisLocus::AspectField { aspect, path } => {
                let mut bytes = aspect.as_str().len();
                let fields = path.fields().len();
                admit(fields.checked_mul(2).ok_or(AccountingOverflow)?, 0).map_err(Resource)?;
                for field in path.fields() {
                    bytes = bytes
                        .checked_add(field.as_str().len())
                        .ok_or(AccountingOverflow)?;
                }
                let backing = fields
                    .checked_mul(size_of::<crate::aspects::FieldKey>())
                    .and_then(|backing| backing.checked_add(bytes))
                    .ok_or(AccountingOverflow)?;
                (
                    bytes.checked_add(fields).ok_or(AccountingOverflow)?,
                    backing,
                    fields.checked_add(1).ok_or(AccountingOverflow)?,
                )
            }
        };
        let value = match entry.value() {
            CanonicalBasisValue::ExactText(text)
            | CanonicalBasisValue::DecimalText(text)
            | CanonicalBasisValue::BigIntText(text) => interned_width(text).0,
            CanonicalBasisValue::RationalText {
                numerator,
                denominator,
            } => interned_width(numerator)
                .0
                .checked_add(interned_width(denominator).0)
                .ok_or(AccountingOverflow)?,
            // Fixed scalar, digest, UUID and enum fields are all bounded here.
            _ => size_of::<CanonicalBasisValue>(),
        };
        let comparison = domain
            .checked_add(locus)
            .and_then(|sum| sum.checked_add(value))
            .and_then(|sum| sum.checked_add(8))
            .ok_or(AccountingOverflow)?;
        width.entry = width.entry.max(comparison);
        width.domain = width.domain.max(domain);
        width.locus = width.locus.max(locus);
        width.duplicate_clone_bytes = width.duplicate_clone_bytes.max(clone_bytes);
        width.duplicate_clone_work = width.duplicate_clone_work.max(
            clone_bytes
                .checked_add(clone_work)
                .ok_or(AccountingOverflow)?,
        );
    }
    Ok(width)
}

fn interned_width(value: &crate::values::InternedString) -> (usize, usize) {
    match value {
        crate::values::InternedString::Raw(text) => (text.len(), text.len()),
        crate::values::InternedString::Symbol(_) => (size_of::<u32>(), 0),
    }
}
