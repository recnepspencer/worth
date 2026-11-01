//! Canonical patch combination moves owned rows and preserves right-hand sets.
use crate::aspects::{
    AspectMask, AuthoritativePatchConstructionDenial as Denial,
    AuthoritativeRecordAspectPatch as Patch, FieldLevelAspectPatch, MutationMask,
};
use std::cmp::Ordering;
use worth_proof::TransitionOutcome;

impl Patch {
    pub fn combine(left: Self, right: Self) -> TransitionOutcome<Self, Denial> {
        match combine_patches(left, right) {
            Ok(patch) => TransitionOutcome::success(patch),
            Err(denial) => TransitionOutcome::denied(denial),
        }
    }

    /// Portable admission already rejected duplicate operation keys in original
    /// order. Move its disjoint single-operation owners exactly once.
    pub(crate) fn from_disjoint_patches(patches: Vec<Self>) -> Self {
        let mut sets = Vec::with_capacity(
            patches
                .iter()
                .map(|patch| patch.whole_aspect_sets.len())
                .sum(),
        );
        let mut clears = Vec::with_capacity(
            patches
                .iter()
                .map(|patch| patch.whole_aspect_clears.len())
                .sum(),
        );
        let mut fields =
            Vec::with_capacity(patches.iter().map(|patch| patch.field_patches.len()).sum());
        for patch in patches {
            sets.extend(patch.whole_aspect_sets.into_vec());
            clears.extend(patch.whole_aspect_clears.into_vec());
            fields.extend(patch.field_patches.into_vec());
        }
        sets.sort_unstable_by(|left, right| left.key().cmp(right.key()));
        clears.sort_unstable_by(|left, right| left.key().cmp(right.key()));
        fields.sort_unstable_by(|left, right| left.key().cmp(right.key()));
        Self {
            whole_aspect_sets: sets.into_boxed_slice(),
            whole_aspect_clears: clears.into_boxed_slice(),
            field_patches: fields.into_boxed_slice(),
        }
    }
}

fn combine_patches(left: Patch, right: Patch) -> Result<Patch, Denial> {
    for key in right
        .whole_aspect_sets()
        .map(|(key, _)| key)
        .chain(right.whole_aspect_clears())
    {
        if left
            .field_patches
            .binary_search_by(|patch| patch.key().cmp(key))
            .is_ok()
        {
            return Err(Denial::AmbiguousWholeAndFieldPatch(key.clone()));
        }
    }
    for (key, _) in right.field_patches() {
        if left
            .whole_aspect_sets
            .binary_search_by(|value| value.key().cmp(key))
            .is_ok()
            || left
                .whole_aspect_clears
                .binary_search_by(|contract| contract.key().cmp(key))
                .is_ok()
        {
            return Err(Denial::AmbiguousWholeAndFieldPatch(key.clone()));
        }
    }
    let sets = merge_rows(
        left.whole_aspect_sets,
        right.whole_aspect_sets,
        |left, right| left.key().cmp(right.key()),
    );
    let clears = merge_rows(
        left.whole_aspect_clears,
        right.whole_aspect_clears,
        |left, right| left.key().cmp(right.key()),
    );
    let mut canonical_clears = Vec::with_capacity(clears.len());
    let mut set_keys = sets.iter().peekable();
    for contract in clears {
        while set_keys
            .peek()
            .is_some_and(|set| set.key() < contract.key())
        {
            set_keys.next();
        }
        if !set_keys
            .peek()
            .is_some_and(|set| set.key() == contract.key())
        {
            canonical_clears.push(contract);
        }
    }
    let mut fields = Vec::with_capacity(left.field_patches.len() + right.field_patches.len());
    let mut left = left.field_patches.into_vec().into_iter().peekable();
    let mut right = right.field_patches.into_vec().into_iter().peekable();
    while let (Some(a), Some(b)) = (left.peek(), right.peek()) {
        match a.key().cmp(b.key()) {
            Ordering::Less => fields.push(left.next().unwrap()),
            Ordering::Greater => fields.push(right.next().unwrap()),
            Ordering::Equal => {
                fields.push(combine_fields(left.next().unwrap(), right.next().unwrap())?)
            }
        }
    }
    fields.extend(left);
    fields.extend(right);
    Ok(Patch {
        whole_aspect_sets: sets,
        whole_aspect_clears: canonical_clears.into_boxed_slice(),
        field_patches: fields.into_boxed_slice(),
    })
}

fn combine_fields(
    mut left: FieldLevelAspectPatch,
    right: FieldLevelAspectPatch,
) -> Result<FieldLevelAspectPatch, Denial> {
    let overlap = right
        .field_sets()
        .map(|(key, _)| key)
        .chain(right.field_clears())
        .any(|key| {
            left.field_sets
                .binary_search_by(|(field, _)| field.cmp(key))
                .is_ok()
                || left.field_clears.binary_search(key).is_ok()
        });
    if left.contract != right.contract || overlap {
        return Err(Denial::DuplicateFieldPatch(left.key().clone()));
    }
    left.mask = AspectMask::<MutationMask>::new(
        left.mask.paths().iter().chain(right.mask.paths()).cloned(),
    );
    left.field_sets = merge_rows(left.field_sets, right.field_sets, |left, right| {
        left.0.cmp(&right.0)
    });
    left.field_clears = merge_rows(left.field_clears, right.field_clears, Ord::cmp);
    Ok(left)
}

/// Private movement mechanic for this patch owner's canonical row families.
/// Meaning decisions and overlap refusals remain at their concrete callers.
fn merge_rows<T>(left: Box<[T]>, right: Box<[T]>, compare: fn(&T, &T) -> Ordering) -> Box<[T]> {
    let mut rows = Vec::with_capacity(left.len() + right.len());
    let mut left = left.into_vec().into_iter().peekable();
    let mut right = right.into_vec().into_iter().peekable();
    while let (Some(a), Some(b)) = (left.peek(), right.peek()) {
        match compare(a, b) {
            Ordering::Less => rows.push(left.next().unwrap()),
            Ordering::Greater => rows.push(right.next().unwrap()),
            Ordering::Equal => {
                left.next();
                rows.push(right.next().unwrap());
            }
        }
    }
    rows.extend(left);
    rows.extend(right);
    rows.into_boxed_slice()
}
