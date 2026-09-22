//! Exact field-mutation truth projected from one owner-validated mutation.

use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, CanonicalFieldPath, PortableAspectPatchOperation,
};

use super::ValidatedRelationalProposal;
use crate::transactions::data::{
    planned_aspect_field_locator, EntityMutationIntent, MutationIntent, RecordRef,
    RelationMutationIntent,
};

/// Operation-local work performed while projecting a validated mutation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ValidatedMutationFootprintWork {
    validated_intents_examined: usize,
    mutation_targets_materialized: usize,
}

impl ValidatedMutationFootprintWork {
    pub const fn validated_intents_examined(self) -> usize {
        self.validated_intents_examined
    }

    pub const fn mutation_targets_materialized(self) -> usize {
        self.mutation_targets_materialized
    }

    fn record_validated_intent(&mut self) {
        self.validated_intents_examined += 1;
    }

    fn record_materialized_targets(&mut self, count: usize) {
        self.mutation_targets_materialized += count;
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ValidatedFieldMutationTarget {
    ExactField {
        record: RecordRef,
        locator: AspectFieldLocator,
    },
    WholeAspect {
        record: RecordRef,
        aspect: AspectKey,
    },
    WholeRecord {
        record: RecordRef,
    },
}

/// Read-only exact-field description of one invariant-validated mutation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedMutationFootprint {
    targets: Vec<ValidatedFieldMutationTarget>,
    work: ValidatedMutationFootprintWork,
}

/// Evidence that no footprint work ran because its consumer did not request it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ValidatedMutationFootprintNotRequested {
    work: ValidatedMutationFootprintWork,
}

impl ValidatedMutationFootprintNotRequested {
    pub const fn work(self) -> ValidatedMutationFootprintWork {
        self.work
    }
}

/// Demand-gated result of projecting exact mutation truth.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ValidatedMutationFootprintProjection {
    NotRequested(ValidatedMutationFootprintNotRequested),
    Projected(ValidatedMutationFootprint),
}

impl ValidatedMutationFootprintProjection {
    pub const fn work(&self) -> ValidatedMutationFootprintWork {
        match self {
            Self::NotRequested(not_requested) => not_requested.work(),
            Self::Projected(footprint) => footprint.work(),
        }
    }

    pub const fn projected(&self) -> Option<&ValidatedMutationFootprint> {
        match self {
            Self::NotRequested(_) => None,
            Self::Projected(footprint) => Some(footprint),
        }
    }

    pub fn into_projected(self) -> Option<ValidatedMutationFootprint> {
        match self {
            Self::NotRequested(_) => None,
            Self::Projected(footprint) => Some(footprint),
        }
    }
}

impl ValidatedMutationFootprint {
    /// True only when the validated plan changes this exact prior field.
    pub fn mutates_field(&self, record: &RecordRef, locator: &AspectFieldLocator) -> bool {
        self.targets.iter().any(|target| match target {
            ValidatedFieldMutationTarget::ExactField {
                record: target_record,
                locator: target_locator,
            } => target_record == record && target_locator == locator,
            ValidatedFieldMutationTarget::WholeAspect {
                record: target_record,
                aspect,
            } => target_record == record && aspect == locator.aspect().aspect_key(),
            ValidatedFieldMutationTarget::WholeRecord {
                record: target_record,
            } => target_record == record,
        })
    }

    pub fn is_empty(&self) -> bool {
        self.targets.is_empty()
    }

    pub const fn work(&self) -> ValidatedMutationFootprintWork {
        self.work
    }
}

impl ValidatedRelationalProposal {
    /// Projects field mutation truth only when the consumer supplies demand.
    pub fn mutation_footprint<Demand: ?Sized>(
        &self,
        demand: Option<&Demand>,
    ) -> ValidatedMutationFootprintProjection {
        if demand.is_none() {
            return ValidatedMutationFootprintProjection::NotRequested(
                ValidatedMutationFootprintNotRequested {
                    work: ValidatedMutationFootprintWork::default(),
                },
            );
        }
        let validated_intents = &self.prepared.merged_plan.merged_intents;
        let mut targets = Vec::new();
        let mut work = ValidatedMutationFootprintWork::default();
        for intent in validated_intents {
            work.record_validated_intent();
            let materialized = targets_for_intent(intent);
            work.record_materialized_targets(materialized.len());
            targets.extend(materialized);
        }
        ValidatedMutationFootprintProjection::Projected(ValidatedMutationFootprint {
            targets,
            work,
        })
    }
}

fn targets_for_intent(intent: &MutationIntent) -> Vec<ValidatedFieldMutationTarget> {
    match intent {
        MutationIntent::Create(_) => Vec::new(),
        MutationIntent::Entity(intent) => entity_targets(intent),
        MutationIntent::Relation(intent) => relation_targets(intent),
        MutationIntent::Materialization(intent) => {
            vec![ValidatedFieldMutationTarget::WholeRecord {
                record: intent.record(),
            }]
        }
    }
}

fn entity_targets(intent: &EntityMutationIntent) -> Vec<ValidatedFieldMutationTarget> {
    match intent {
        EntityMutationIntent::UpdateFields(update) => update
            .fields
            .locators()
            .cloned()
            .map(|locator| ValidatedFieldMutationTarget::ExactField {
                record: RecordRef::Entity(update.entity_id),
                locator,
            })
            .collect(),
        EntityMutationIntent::ApplyAspectPatch(update) => {
            patch_targets(RecordRef::Entity(update.entity_id), &update.aspect_patch)
        }
        EntityMutationIntent::Replace(replace) => vec![ValidatedFieldMutationTarget::WholeRecord {
            record: RecordRef::Entity(replace.entity_id),
        }],
        EntityMutationIntent::Delete(delete) => vec![ValidatedFieldMutationTarget::WholeRecord {
            record: RecordRef::Entity(delete.entity_id),
        }],
        // A revalidation demand mutates no field, so it claims no mutation
        // target: the footprint a validator sees must not name a write the
        // candidate never makes.
        EntityMutationIntent::Revalidate(_) => Vec::new(),
    }
}

fn relation_targets(intent: &RelationMutationIntent) -> Vec<ValidatedFieldMutationTarget> {
    match intent {
        RelationMutationIntent::ApplyAspectPatch(update) => patch_targets(
            RecordRef::Relation(update.relation_id),
            &update.aspect_patch,
        ),
        RelationMutationIntent::UpdateEndpoints(_) => Vec::new(),
        RelationMutationIntent::Delete(delete) => vec![ValidatedFieldMutationTarget::WholeRecord {
            record: RecordRef::Relation(delete.relation_id),
        }],
    }
}

fn patch_targets(
    record: RecordRef,
    patch: &worth_foundational::facade::PortableRecordAspectPatch,
) -> Vec<ValidatedFieldMutationTarget> {
    patch
        .operations()
        .iter()
        .flat_map(|operation| match operation {
            PortableAspectPatchOperation::SetWhole { basis, .. }
            | PortableAspectPatchOperation::ClearWhole { basis } => {
                vec![ValidatedFieldMutationTarget::WholeAspect {
                    record: record.clone(),
                    aspect: basis.key().clone(),
                }]
            }
            PortableAspectPatchOperation::PatchFields {
                basis,
                selected_fields,
                ..
            } => selected_fields
                .iter()
                .cloned()
                .map(|field| ValidatedFieldMutationTarget::ExactField {
                    record: record.clone(),
                    locator: planned_aspect_field_locator(
                        basis.key().clone(),
                        CanonicalFieldPath::single(field),
                    ),
                })
                .collect(),
        })
        .collect()
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
