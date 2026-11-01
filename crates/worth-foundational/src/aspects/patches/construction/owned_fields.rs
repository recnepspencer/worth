//! Fully materialized field rosters preserve original validation order.
use super::field_level::{validate_contract_mask, validate_field_clear, validate_field_set};
use crate::aspects::{
    AspectContract, AspectMask, AuthoritativePatchConstructionDenial as Denial, FieldKey,
    FieldLevelAspectPatch, MutationMask,
};
use crate::values::AspectValue;

pub(super) struct ClearOrdinal {
    key: FieldKey,
    ordinal: usize,
}

impl FieldLevelAspectPatch {
    pub(crate) fn from_owned_fields(
        contract: &AspectContract,
        mask: &AspectMask<MutationMask>,
        mut sets: Vec<(FieldKey, AspectValue)>,
        clears: Vec<FieldKey>,
    ) -> Result<Self, Denial> {
        let shape = validate_contract_mask(contract, mask)?;
        let duplicate = first_duplicate(&sets);
        for (ordinal, (key, value)) in sets.iter().enumerate() {
            validate_field_set(shape, mask, key, value)?;
            if Some(ordinal) == duplicate {
                return Err(Denial::DuplicateFieldSet(key.clone()));
            }
        }
        sets.sort_unstable_by(|left, right| left.0.cmp(&right.0));
        let mut clear_rows = Vec::with_capacity(clears.len());
        for (ordinal, key) in clears.into_iter().enumerate() {
            validate_field_clear(shape, mask, &key)?;
            if sets.binary_search_by(|(set, _)| set.cmp(&key)).is_err() {
                clear_rows.push(ClearOrdinal { key, ordinal });
            }
        }
        clear_rows.sort_unstable_by(|left, right| {
            left.key
                .cmp(&right.key)
                .then(left.ordinal.cmp(&right.ordinal))
        });
        clear_rows.dedup_by(|right, left| right.key == left.key);
        let mut canonical_clears = Vec::with_capacity(clear_rows.len());
        canonical_clears.extend(clear_rows.into_iter().map(|row| row.key));
        if sets.is_empty() && canonical_clears.is_empty() {
            return Err(Denial::EmptyFieldPatch);
        }
        Ok(Self {
            contract: contract.clone(),
            mask: mask.clone(),
            field_sets: sets.into_boxed_slice(),
            field_clears: canonical_clears.into_boxed_slice(),
        })
    }
}

fn first_duplicate(sets: &[(FieldKey, AspectValue)]) -> Option<usize> {
    let mut keys = Vec::with_capacity(sets.len());
    keys.extend(
        sets.iter()
            .enumerate()
            .map(|(ordinal, (key, _))| (key, ordinal)),
    );
    keys.sort_unstable_by(|left, right| left.0.cmp(right.0).then(left.1.cmp(&right.1)));
    keys.windows(2)
        .filter(|pair| pair[0].0 == pair[1].0)
        .map(|pair| pair[1].1)
        .min()
}
