use serde::{Deserialize, Serialize};
use worth_foundational::facade::{
    AspectBinding, AspectContractRevision, AspectIdentity, AspectKey, AspectValue, FieldKey,
    StructAspectValue,
};

use super::ordered_aspect_keys;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PublishedAuthoritativePatch {
    operations: Vec<PublishedAuthoritativePatchOperation>,
}

impl PublishedAuthoritativePatch {
    pub fn empty() -> Self {
        Self::default()
    }

    pub(crate) fn new(operations: Vec<PublishedAuthoritativePatchOperation>) -> Self {
        Self { operations }.canonicalized()
    }

    pub fn canonicalized(&self) -> Self {
        let mut operations = self.operations.clone();
        operations.sort_by(published_operation_order);
        Self { operations }
    }

    pub fn changed_aspects(&self) -> Vec<AspectKey> {
        ordered_aspect_keys(self.changed_aspect_keys().cloned())
    }

    pub fn changed_aspect_keys(&self) -> impl Iterator<Item = &AspectKey> {
        self.operations
            .iter()
            .map(PublishedAuthoritativePatchOperation::aspect_key)
    }

    pub(crate) fn full_grammar_operations(&self) -> &[PublishedAuthoritativePatchOperation] {
        &self.operations
    }

    /// The number of operations in the full patch grammar.
    pub fn full_grammar_operation_count(&self) -> usize {
        self.operations.len()
    }

    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        self.operations.iter().fold(
            (self.operations.capacity()
                * std::mem::size_of::<PublishedAuthoritativePatchOperation>()) as u64,
            |bytes, operation| bytes.saturating_add(operation.owned_allocation_capacity_bytes()),
        )
    }

    pub fn scalar_set_for(&self, aspect_key: &AspectKey) -> Option<&AspectValue> {
        self.operations
            .iter()
            .find_map(|operation| match operation {
                PublishedAuthoritativePatchOperation::WholeAspectSet {
                    aspect_key: operation_aspect_key,
                    value: PublishedAuthoritativePatchValue::Scalar(value),
                    ..
                } if operation_aspect_key == aspect_key => Some(value),
                _ => None,
            })
    }

    pub fn struct_set_for(&self, aspect_key: &AspectKey) -> Option<&StructAspectValue> {
        self.operations
            .iter()
            .find_map(|operation| match operation {
                PublishedAuthoritativePatchOperation::WholeAspectSet {
                    aspect_key: operation_aspect_key,
                    value: PublishedAuthoritativePatchValue::Struct(value),
                    ..
                } if operation_aspect_key == aspect_key => Some(value),
                _ => None,
            })
    }

    pub fn field_sets_for<'a>(
        &'a self,
        aspect_key: &'a AspectKey,
    ) -> impl Iterator<Item = &'a PublishedAuthoritativeFieldSet> + 'a {
        self.operations
            .iter()
            .filter_map(move |operation| match operation {
                PublishedAuthoritativePatchOperation::FieldLevelPatch {
                    aspect_key: operation_aspect_key,
                    field_sets,
                    ..
                } if operation_aspect_key == aspect_key => Some(field_sets.as_slice()),
                _ => None,
            })
            .flatten()
    }

    pub fn field_clears_for<'a>(
        &'a self,
        aspect_key: &'a AspectKey,
    ) -> impl Iterator<Item = &'a FieldKey> + 'a {
        self.operations
            .iter()
            .filter_map(move |operation| match operation {
                PublishedAuthoritativePatchOperation::FieldLevelPatch {
                    aspect_key: operation_aspect_key,
                    field_clears,
                    ..
                } if operation_aspect_key == aspect_key => Some(field_clears.as_slice()),
                _ => None,
            })
            .flatten()
    }

    pub fn whole_clear_for(&self, aspect_key: &AspectKey) -> bool {
        self.operations.iter().any(|operation| {
            matches!(
                operation,
                PublishedAuthoritativePatchOperation::WholeAspectClear {
                    aspect_key: operation_aspect_key,
                    ..
                } if operation_aspect_key == aspect_key
            )
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum PublishedAuthoritativePatchOperation {
    WholeAspectSet {
        aspect_key: AspectKey,
        aspect_identity: AspectIdentity,
        contract_revision: AspectContractRevision,
        binding: AspectBinding,
        value: PublishedAuthoritativePatchValue,
    },
    WholeAspectClear {
        aspect_key: AspectKey,
        aspect_identity: AspectIdentity,
        contract_revision: AspectContractRevision,
        binding: AspectBinding,
    },
    FieldLevelPatch {
        aspect_key: AspectKey,
        aspect_identity: AspectIdentity,
        contract_revision: AspectContractRevision,
        binding: AspectBinding,
        field_sets: Vec<PublishedAuthoritativeFieldSet>,
        field_clears: Vec<FieldKey>,
    },
}

impl PublishedAuthoritativePatchOperation {
    pub fn aspect_key(&self) -> &AspectKey {
        match self {
            Self::WholeAspectSet { aspect_key, .. }
            | Self::WholeAspectClear { aspect_key, .. }
            | Self::FieldLevelPatch { aspect_key, .. } => aspect_key,
        }
    }

    fn owned_allocation_capacity_bytes(&self) -> u64 {
        let aspect_and_binding = match self {
            Self::WholeAspectSet {
                aspect_key,
                binding,
                ..
            }
            | Self::WholeAspectClear {
                aspect_key,
                binding,
                ..
            }
            | Self::FieldLevelPatch {
                aspect_key,
                binding,
                ..
            } => {
                (aspect_key.owned_allocation_capacity_bytes()
                    + binding.owned_allocation_capacity_bytes()) as u64
            }
        };
        match self {
            Self::WholeAspectSet { value, .. } => {
                aspect_and_binding.saturating_add(value.owned_allocation_capacity_bytes())
            }
            Self::WholeAspectClear { .. } => aspect_and_binding,
            Self::FieldLevelPatch {
                field_sets,
                field_clears,
                ..
            } => aspect_and_binding
                .saturating_add(
                    (field_sets.capacity() * std::mem::size_of::<PublishedAuthoritativeFieldSet>())
                        as u64,
                )
                .saturating_add(
                    field_sets
                        .iter()
                        .map(|field| {
                            (field.field.owned_allocation_capacity_bytes()
                                + field.value.owned_allocation_capacity_bytes())
                                as u64
                        })
                        .sum(),
                )
                .saturating_add((field_clears.capacity() * std::mem::size_of::<FieldKey>()) as u64)
                .saturating_add(
                    field_clears
                        .iter()
                        .map(|field| field.owned_allocation_capacity_bytes() as u64)
                        .sum(),
                ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum PublishedAuthoritativePatchValue {
    Scalar(AspectValue),
    Struct(StructAspectValue),
}

impl PublishedAuthoritativePatchValue {
    fn owned_allocation_capacity_bytes(&self) -> u64 {
        match self {
            Self::Scalar(value) => value.owned_allocation_capacity_bytes() as u64,
            Self::Struct(value) => value.owned_allocation_capacity_bytes() as u64,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PublishedAuthoritativeFieldSet {
    pub field: FieldKey,
    pub value: AspectValue,
}

fn published_operation_order(
    left: &PublishedAuthoritativePatchOperation,
    right: &PublishedAuthoritativePatchOperation,
) -> std::cmp::Ordering {
    operation_rank(left)
        .cmp(&operation_rank(right))
        .then_with(|| left.aspect_key().cmp(right.aspect_key()))
}

fn operation_rank(operation: &PublishedAuthoritativePatchOperation) -> u8 {
    match operation {
        PublishedAuthoritativePatchOperation::WholeAspectSet { .. } => 0,
        PublishedAuthoritativePatchOperation::WholeAspectClear { .. } => 1,
        PublishedAuthoritativePatchOperation::FieldLevelPatch { .. } => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalar_set_query_ignores_field_level_patch_values_for_same_aspect() {
        let aspect_key = AspectKey::new("counter").unwrap();
        let field = FieldKey::new("value").unwrap();
        let patch = PublishedAuthoritativePatch::new(vec![
            PublishedAuthoritativePatchOperation::FieldLevelPatch {
                aspect_key: aspect_key.clone(),
                aspect_identity: AspectIdentity(1),
                contract_revision: AspectContractRevision(1),
                binding: AspectBinding::EntityField {
                    field: FieldKey::new("counter").unwrap(),
                },
                field_sets: vec![PublishedAuthoritativeFieldSet {
                    field,
                    value: AspectValue::Int64(99),
                }],
                field_clears: Vec::new(),
            },
        ]);

        assert_eq!(patch.scalar_set_for(&aspect_key), None);
        let field_sets = patch.field_sets_for(&aspect_key).collect::<Vec<_>>();
        assert_eq!(field_sets[0].value, AspectValue::Int64(99));
    }

    #[test]
    fn shape_specific_patch_queries_keep_operation_lanes_distinct() {
        let scalar_key = AspectKey::new("counter").unwrap();
        let struct_key = AspectKey::new("profile").unwrap();
        let clear_key = AspectKey::new("retired").unwrap();
        let patch = PublishedAuthoritativePatch::new(vec![
            PublishedAuthoritativePatchOperation::WholeAspectSet {
                aspect_key: scalar_key.clone(),
                aspect_identity: AspectIdentity(1),
                contract_revision: AspectContractRevision(1),
                binding: AspectBinding::EntityField {
                    field: FieldKey::new("counter").unwrap(),
                },
                value: PublishedAuthoritativePatchValue::Scalar(AspectValue::Int64(7)),
            },
            PublishedAuthoritativePatchOperation::WholeAspectSet {
                aspect_key: struct_key.clone(),
                aspect_identity: AspectIdentity(2),
                contract_revision: AspectContractRevision(1),
                binding: AspectBinding::EntityField {
                    field: FieldKey::new("profile").unwrap(),
                },
                value: PublishedAuthoritativePatchValue::Struct(
                    StructAspectValue::new(Vec::<(FieldKey, AspectValue)>::new()).unwrap(),
                ),
            },
            PublishedAuthoritativePatchOperation::WholeAspectClear {
                aspect_key: clear_key.clone(),
                aspect_identity: AspectIdentity(3),
                contract_revision: AspectContractRevision(1),
                binding: AspectBinding::EntityField {
                    field: FieldKey::new("retired").unwrap(),
                },
            },
        ]);

        assert_eq!(
            patch.scalar_set_for(&scalar_key),
            Some(&AspectValue::Int64(7))
        );
        assert!(patch.struct_set_for(&struct_key).is_some());
        assert!(patch.whole_clear_for(&clear_key));
        assert_eq!(patch.field_sets_for(&scalar_key).count(), 0);
    }

    #[test]
    fn field_patch_queries_cover_every_operation_for_requested_aspect() {
        let aspect_key = AspectKey::new("summary").unwrap();
        let sibling_key = AspectKey::new("other").unwrap();
        let title = FieldKey::new("title").unwrap();
        let status = FieldKey::new("status").unwrap();
        let stale = FieldKey::new("stale").unwrap();
        let patch = PublishedAuthoritativePatch::new(vec![
            PublishedAuthoritativePatchOperation::FieldLevelPatch {
                aspect_key: aspect_key.clone(),
                aspect_identity: AspectIdentity(1),
                contract_revision: AspectContractRevision(1),
                binding: AspectBinding::EntityField {
                    field: FieldKey::new("summary").unwrap(),
                },
                field_sets: vec![PublishedAuthoritativeFieldSet {
                    field: title.clone(),
                    value: AspectValue::String("draft".into()),
                }],
                field_clears: vec![],
            },
            PublishedAuthoritativePatchOperation::FieldLevelPatch {
                aspect_key: sibling_key,
                aspect_identity: AspectIdentity(2),
                contract_revision: AspectContractRevision(1),
                binding: AspectBinding::EntityField {
                    field: FieldKey::new("other").unwrap(),
                },
                field_sets: vec![PublishedAuthoritativeFieldSet {
                    field: FieldKey::new("ignored").unwrap(),
                    value: AspectValue::String("ignored".into()),
                }],
                field_clears: vec![FieldKey::new("ignored_clear").unwrap()],
            },
            PublishedAuthoritativePatchOperation::FieldLevelPatch {
                aspect_key: aspect_key.clone(),
                aspect_identity: AspectIdentity(1),
                contract_revision: AspectContractRevision(1),
                binding: AspectBinding::EntityField {
                    field: FieldKey::new("summary").unwrap(),
                },
                field_sets: vec![PublishedAuthoritativeFieldSet {
                    field: status.clone(),
                    value: AspectValue::String("ready".into()),
                }],
                field_clears: vec![stale.clone()],
            },
        ]);

        let field_sets = patch
            .field_sets_for(&aspect_key)
            .map(|field_set| (&field_set.field, &field_set.value))
            .collect::<Vec<_>>();
        let field_clears = patch.field_clears_for(&aspect_key).collect::<Vec<_>>();

        assert_eq!(
            field_sets,
            vec![
                (&title, &AspectValue::String("draft".into())),
                (&status, &AspectValue::String("ready".into()))
            ]
        );
        assert_eq!(field_clears, vec![&stale]);
    }
}
