//! Interpret canonical writes only at an actually installed Native field binding.
use super::{
    PublishedAuthoritativePatch, PublishedAuthoritativePatchOperation as Operation,
    PublishedAuthoritativePatchValue as Value,
};
use worth_foundational::facade::{AspectBinding, AspectContract, AspectShape, FieldKey};

/// Descriptive canonical effect, never an observed revision or a read permission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PublishedAuthoritativeFieldEffect {
    Present,
    FieldCleared,
    AspectCleared,
    Unchanged,
    Unsupported,
    Ambiguous,
}
use PublishedAuthoritativeFieldEffect as Effect;
#[derive(Clone, Copy)]
enum FieldStorage {
    Scalar,
    Struct,
}

impl PublishedAuthoritativePatch {
    /// O(operations + selected field entries + declared struct width), without allocation.
    /// Original contract identity/revision and the physical binding must match.
    /// Multiple relevant writes refuse instead of guessing their final order.
    pub fn field_effect<Stop>(
        &self,
        contract: &AspectContract,
        binding: &AspectBinding,
        field: &FieldKey,
        mut admit: impl FnMut(u64) -> Result<(), Stop>,
    ) -> Result<Effect, Stop> {
        let Some(storage) = installed_storage(contract, binding, field, &mut admit)? else {
            return Ok(Effect::Unsupported);
        };
        let mut result = Effect::Unchanged;
        for operation in &self.operations {
            admit((contract.key().as_str().len() + field.as_str().len()) as u64 + 1)?;
            if operation.aspect_key() != contract.key() {
                continue;
            }
            if !original_contract_matches(operation, contract, binding) {
                return Ok(Effect::Unsupported);
            }
            let effect = operation_effect(operation, storage, field, &mut admit)?;
            if matches!(effect, Effect::Unsupported | Effect::Ambiguous) {
                return Ok(effect);
            }
            if effect != Effect::Unchanged {
                if result != Effect::Unchanged {
                    return Ok(Effect::Ambiguous);
                }
                result = effect;
            }
        }
        Ok(result)
    }
}

/// Scalar physical fields are named by EntityField, never an aspect-name guess.
fn installed_storage<Stop>(
    contract: &AspectContract,
    binding: &AspectBinding,
    field: &FieldKey,
    admit: &mut impl FnMut(u64) -> Result<(), Stop>,
) -> Result<Option<FieldStorage>, Stop> {
    admit(1)?;
    let AspectBinding::EntityField { field: bound } = binding else {
        return Ok(None);
    };
    Ok(match contract.shape() {
        AspectShape::Scalar(_) if bound == field => Some(FieldStorage::Scalar),
        AspectShape::Struct(shape) => {
            admit(shape.fields().len() as u64 + 1)?;
            shape.field(field).map(|_| FieldStorage::Struct)
        }
        _ => None,
    })
}

fn original_contract_matches(
    operation: &Operation,
    contract: &AspectContract,
    installed: &AspectBinding,
) -> bool {
    let (identity, revision, binding) = match operation {
        Operation::WholeAspectSet {
            aspect_identity,
            contract_revision,
            binding,
            ..
        }
        | Operation::WholeAspectClear {
            aspect_identity,
            contract_revision,
            binding,
            ..
        }
        | Operation::FieldLevelPatch {
            aspect_identity,
            contract_revision,
            binding,
            ..
        } => (aspect_identity, contract_revision, binding),
    };
    *identity == contract.identity() && *revision == contract.revision() && binding == installed
}

fn operation_effect<Stop>(
    operation: &Operation,
    storage: FieldStorage,
    field: &FieldKey,
    admit: &mut impl FnMut(u64) -> Result<(), Stop>,
) -> Result<Effect, Stop> {
    Ok(match (operation, storage) {
        (
            Operation::WholeAspectSet {
                value: Value::Scalar(_),
                ..
            },
            FieldStorage::Scalar,
        ) => Effect::Present,
        (
            Operation::WholeAspectSet {
                value: Value::Struct(value),
                ..
            },
            FieldStorage::Struct,
        ) => {
            admit(value.len() as u64 + 1)?;
            if value.get(field).is_some() {
                Effect::Present
            } else {
                Effect::FieldCleared
            }
        }
        (Operation::WholeAspectSet { .. }, _) => Effect::Unsupported,
        (Operation::WholeAspectClear { .. }, _) => Effect::AspectCleared,
        (
            Operation::FieldLevelPatch {
                field_sets,
                field_clears,
                ..
            },
            _,
        ) => {
            admit((field_sets.len() + field_clears.len()) as u64 + 1)?;
            let sets = field_sets.iter().filter(|set| &set.field == field).count();
            let clears = field_clears.iter().filter(|key| *key == field).count();
            match (sets, clears) {
                (0, 0) => Effect::Unchanged,
                (1, 0) => Effect::Present,
                (0, 1) => match storage {
                    FieldStorage::Scalar => Effect::AspectCleared,
                    FieldStorage::Struct => Effect::FieldCleared,
                },
                _ => Effect::Ambiguous,
            }
        }
    })
}
