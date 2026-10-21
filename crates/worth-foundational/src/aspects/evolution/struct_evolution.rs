use std::collections::BTreeSet;

use super::{scalar_widens, AspectEvolutionKind, AspectEvolutionVerdict};
use crate::aspects::structs::{FieldDeclaration, FieldRequirement, StructAspectShape};

pub fn classify_struct_evolution(
    left: &StructAspectShape,
    right: &StructAspectShape,
) -> AspectEvolutionVerdict {
    if left.fields().iter().any(|field| {
        right
            .field(field.key())
            .is_some_and(|next| field.codec() != next.codec())
    }) {
        return AspectEvolutionVerdict::new(
            AspectEvolutionKind::Incompatible,
            "struct field codec meaning changed",
        );
    }
    let left_keys: BTreeSet<_> = left.fields().iter().map(FieldDeclaration::key).collect();
    let right_keys: BTreeSet<_> = right.fields().iter().map(FieldDeclaration::key).collect();

    if left_keys == right_keys {
        classify_equal_field_set_evolution(left, right, left_keys)
    } else if left_keys.is_subset(&right_keys) {
        classify_added_field_evolution(right, &left_keys)
    } else {
        AspectEvolutionVerdict::new(
            AspectEvolutionKind::Incompatible,
            "struct fields were removed or renamed",
        )
    }
}

fn classify_equal_field_set_evolution(
    left: &StructAspectShape,
    right: &StructAspectShape,
    shared_keys: BTreeSet<&crate::aspects::structs::FieldKey>,
) -> AspectEvolutionVerdict {
    for key in shared_keys {
        let left_field = left.field(key).expect("left key from left shape");
        let right_field = right.field(key).expect("right key from equal set");
        if left_field.value_type() != right_field.value_type() {
            return classify_field_type_change(left_field, right_field);
        }
        if left_field.requirement() != right_field.requirement()
            || left_field.absence() != right_field.absence()
        {
            return AspectEvolutionVerdict::new(
                AspectEvolutionKind::Incompatible,
                "struct field requirement or absence law changed",
            );
        }
    }
    AspectEvolutionVerdict::new(AspectEvolutionKind::Unchanged, "struct shape unchanged")
}

fn classify_added_field_evolution(
    right: &StructAspectShape,
    previous_keys: &BTreeSet<&crate::aspects::structs::FieldKey>,
) -> AspectEvolutionVerdict {
    let added_fields_are_backwards_readable = right
        .fields()
        .iter()
        .filter(|field| !previous_keys.contains(field.key()))
        .all(field_addition_is_backwards_readable);

    if added_fields_are_backwards_readable {
        AspectEvolutionVerdict::new(
            AspectEvolutionKind::Additive,
            "optional or defaulted struct fields were added",
        )
    } else {
        AspectEvolutionVerdict::new(
            AspectEvolutionKind::Incompatible,
            "required struct fields were added",
        )
    }
}

fn field_addition_is_backwards_readable(field: &FieldDeclaration) -> bool {
    !matches!(field.requirement(), FieldRequirement::Required)
}

fn classify_field_type_change(
    left_field: &FieldDeclaration,
    right_field: &FieldDeclaration,
) -> AspectEvolutionVerdict {
    if scalar_widens(left_field.value_type(), right_field.value_type()) {
        AspectEvolutionVerdict::new(AspectEvolutionKind::Widening, "struct field widened")
    } else {
        AspectEvolutionVerdict::new(
            AspectEvolutionKind::Narrowing,
            "struct field narrowed or changed incompatibly",
        )
    }
}
