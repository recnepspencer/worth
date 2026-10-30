use std::collections::BTreeSet;

use worth_foundational::{AspectShape, ScalarAspectType};

use crate::merge::data::{AspectMergePolicyDeclaration, AspectMergePolicyKind};
use crate::schema::data::{AspectBinding, DeclaredAspectContractBinding, SchemaRegistryError};

#[expect(
    clippy::disallowed_methods,
    reason = "Comparison uses declared semantic keys; type names label diagnostics only"
)]
pub(super) fn canonicalize_merge_policy_declarations(
    kind_id: crate::identity::data::KindId,
    mut declarations: Vec<AspectMergePolicyDeclaration>,
    aspects: &[DeclaredAspectContractBinding],
) -> Result<Vec<AspectMergePolicyDeclaration>, SchemaRegistryError> {
    declarations.sort_by(|left, right| left.aspect_key.cmp(&right.aspect_key));
    let aspect_keys = aspects
        .iter()
        .map(DeclaredAspectContractBinding::aspect_key)
        .collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();
    for declaration in &declarations {
        reject_policy_for_undeclared_aspect(kind_id, declaration, &aspect_keys)?;
        reject_duplicate_policy(kind_id, declaration, &mut seen)?;
        validate_policy_contract(kind_id, declaration, aspects)?;
    }
    Ok(declarations)
}

fn reject_policy_for_undeclared_aspect(
    kind_id: crate::identity::data::KindId,
    declaration: &AspectMergePolicyDeclaration,
    aspect_keys: &BTreeSet<worth_foundational::facade::AspectKey>,
) -> Result<(), SchemaRegistryError> {
    if aspect_keys.contains(&declaration.aspect_key) {
        return Ok(());
    }
    Err(SchemaRegistryError::invalid_aspect_declaration(
        kind_id,
        format!(
            "merge policy declaration references undeclared aspect key {:?}",
            declaration.aspect_key
        ),
    ))
}

fn reject_duplicate_policy(
    kind_id: crate::identity::data::KindId,
    declaration: &AspectMergePolicyDeclaration,
    seen: &mut BTreeSet<worth_foundational::facade::AspectKey>,
) -> Result<(), SchemaRegistryError> {
    if seen.insert(declaration.aspect_key.clone()) {
        return Ok(());
    }
    Err(SchemaRegistryError::invalid_aspect_declaration(
        kind_id,
        format!(
            "duplicate merge policy declaration for aspect key {:?}",
            declaration.aspect_key
        ),
    ))
}

fn validate_policy_contract(
    kind_id: crate::identity::data::KindId,
    declaration: &AspectMergePolicyDeclaration,
    aspects: &[DeclaredAspectContractBinding],
) -> Result<(), SchemaRegistryError> {
    let aspect = aspects
        .iter()
        .find(|aspect| aspect.aspect_key() == declaration.aspect_key)
        .expect(/* Proof: Schema admission resolves each merge-policy declaration against its declared aspect list before constructing this plan; that list is immutable here. */ "merge policy declaration references existing declared aspect");
    match declaration.policy {
        AspectMergePolicyKind::FailOnConflict | AspectMergePolicyKind::PreferRicher => Ok(()),
        AspectMergePolicyKind::LastWriterWins => {
            validate_last_writer_wins_contract(kind_id, aspect)
        }
        AspectMergePolicyKind::MonotonicCounter => {
            validate_monotonic_counter_contract(kind_id, aspect)
        }
        AspectMergePolicyKind::AdditiveSet => reject_additive_set_without_native_contract(kind_id),
        AspectMergePolicyKind::Custom(_) => reject_custom_policy(kind_id),
    }
}

fn validate_last_writer_wins_contract(
    kind_id: crate::identity::data::KindId,
    aspect: &DeclaredAspectContractBinding,
) -> Result<(), SchemaRegistryError> {
    if is_scalar_record_field_aspect(aspect) {
        return Ok(());
    }
    Err(SchemaRegistryError::invalid_aspect_declaration(
        kind_id,
        "LastWriterWins merge policy requires a scalar record-field foundational contract",
    ))
}

fn validate_monotonic_counter_contract(
    kind_id: crate::identity::data::KindId,
    aspect: &DeclaredAspectContractBinding,
) -> Result<(), SchemaRegistryError> {
    if is_integer_record_field_aspect(aspect) {
        return Ok(());
    }
    Err(SchemaRegistryError::invalid_aspect_declaration(
        kind_id,
        "MonotonicCounter merge policy requires an integer scalar record-field foundational contract",
    ))
}

fn reject_additive_set_without_native_contract(
    kind_id: crate::identity::data::KindId,
) -> Result<(), SchemaRegistryError> {
    Err(SchemaRegistryError::invalid_aspect_declaration(
        kind_id,
        "AdditiveSet merge policy requires a native foundational set contract, which is not available in this milestone",
    ))
}

fn reject_custom_policy(kind_id: crate::identity::data::KindId) -> Result<(), SchemaRegistryError> {
    Err(SchemaRegistryError::invalid_aspect_declaration(
        kind_id,
        "merge planning does not yet support custom merge policy declarations",
    ))
}

fn is_scalar_record_field_aspect(aspect: &DeclaredAspectContractBinding) -> bool {
    matches!(
        (&aspect.binding, aspect.contract.shape()),
        (
            AspectBinding::EntityField { .. } | AspectBinding::RelationField { .. },
            AspectShape::Scalar(_)
        )
    )
}

fn is_integer_record_field_aspect(aspect: &DeclaredAspectContractBinding) -> bool {
    matches!(
        (&aspect.binding, aspect.contract.shape()),
        (
            AspectBinding::EntityField { .. } | AspectBinding::RelationField { .. },
            AspectShape::Scalar(
                ScalarAspectType::Int8
                    | ScalarAspectType::Int16
                    | ScalarAspectType::Int32
                    | ScalarAspectType::Int64
            )
        )
    )
}
