use worth_proof::TransitionOutcome;

use crate::aspects::masks::{AspectMask, MutationMask};
use crate::aspects::patches::{
    AuthoritativePatchConstructionDenial, AuthoritativeRecordAspectPatch, FieldLevelAspectPatch,
};

impl AuthoritativeRecordAspectPatch {
    pub fn combine(
        left: Self,
        right: Self,
    ) -> TransitionOutcome<Self, AuthoritativePatchConstructionDenial> {
        match combine_patches(left, right) {
            Ok(patch) => TransitionOutcome::success(patch),
            Err(denial) => TransitionOutcome::denied(denial),
        }
    }
}

fn combine_patches(
    mut left: AuthoritativeRecordAspectPatch,
    right: AuthoritativeRecordAspectPatch,
) -> Result<AuthoritativeRecordAspectPatch, AuthoritativePatchConstructionDenial> {
    for key in right
        .whole_aspect_sets
        .keys()
        .chain(right.whole_aspect_clears.keys())
    {
        if left.field_patches.contains_key(key) {
            return Err(
                AuthoritativePatchConstructionDenial::AmbiguousWholeAndFieldPatch(key.clone()),
            );
        }
    }

    for key in right.field_patches.keys() {
        if left.whole_aspect_sets.contains_key(key) || left.whole_aspect_clears.contains_key(key) {
            return Err(
                AuthoritativePatchConstructionDenial::AmbiguousWholeAndFieldPatch(key.clone()),
            );
        }
    }

    for (key, contract) in right.whole_aspect_clears {
        if !left.whole_aspect_sets.contains_key(&key) {
            left.whole_aspect_clears.insert(key, contract);
        }
    }

    for (key, value) in right.whole_aspect_sets {
        left.whole_aspect_clears.remove(&key);
        left.whole_aspect_sets.insert(key, value);
    }

    for (key, right_patch) in right.field_patches {
        match left.field_patches.remove(&key) {
            Some(left_patch) => {
                left.field_patches
                    .insert(key.clone(), combine_field_patches(left_patch, right_patch)?);
            }
            None => {
                left.field_patches.insert(key, right_patch);
            }
        }
    }
    Ok(left)
}

fn combine_field_patches(
    mut left: FieldLevelAspectPatch,
    right: FieldLevelAspectPatch,
) -> Result<FieldLevelAspectPatch, AuthoritativePatchConstructionDenial> {
    let key = left.key().clone();
    let left_fields = left
        .field_sets
        .keys()
        .chain(left.field_clears.iter())
        .collect::<std::collections::BTreeSet<_>>();
    let overlaps = right
        .field_sets
        .keys()
        .chain(right.field_clears.iter())
        .any(|field| left_fields.contains(field));
    if left.contract != right.contract || overlaps {
        return Err(AuthoritativePatchConstructionDenial::DuplicateFieldPatch(
            key,
        ));
    }

    left.mask = AspectMask::<MutationMask>::new(
        left.mask.paths().iter().chain(right.mask.paths()).cloned(),
    );
    left.field_sets.extend(right.field_sets);
    left.field_clears.extend(right.field_clears);
    Ok(left)
}
