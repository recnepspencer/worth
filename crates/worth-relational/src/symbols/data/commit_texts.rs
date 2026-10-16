//! Exact referenced native symbol meanings, preserving each stored custody kind.
use super::{StringInterner, Symbol, SymbolTableSnapshot};
use crate::transactions::data::*;
use std::collections::BTreeSet;
use worth_foundational::facade::{
    AspectValue, ContractValidationInput, InternedString, PortableAspectPatchOperation,
    PortableRecordAspectPatch,
};
impl StringInterner {
    pub(crate) fn commit_texts(
        &self,
        plan: &MergedCommitPlan,
        patch: &crate::publication::patch::data::CanonicalAuthoritativePatch,
    ) -> Result<SymbolTableSnapshot, &'static str> {
        let entries = required_commit_symbols(plan, patch)
            .into_iter()
            .map(|symbol| {
                self.retained_text(symbol)
                    .map(|text| (symbol, text.clone()))
                    .ok_or("canonical payload references an unresolved symbol")
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(SymbolTableSnapshot { entries })
    }
}
pub(crate) fn required_commit_symbols(
    plan: &MergedCommitPlan,
    patch: &crate::publication::patch::data::CanonicalAuthoritativePatch,
) -> BTreeSet<Symbol> {
    use crate::publication::patch::data::{
        PublishedAuthoritativePatchOperation as Operation,
        PublishedAuthoritativePatchValue as Payload,
    };
    let mut symbols = BTreeSet::new();
    for intent in &plan.merged_intents {
        visit_intent(intent, &mut symbols);
    }
    for record in &patch.authoritative_record_patches {
        for operation in record.authoritative_patch.full_grammar_operations() {
            match operation {
                Operation::WholeAspectSet { value: input, .. } => match input {
                    Payload::Scalar(input) => value(input, &mut symbols),
                    Payload::Struct(input) => {
                        for (_, field) in input.fields() {
                            value(field, &mut symbols);
                        }
                    }
                },
                Operation::FieldLevelPatch { field_sets, .. } => {
                    for field in field_sets {
                        value(&field.value, &mut symbols);
                    }
                }
                Operation::WholeAspectClear { .. } => {}
            }
        }
    }
    symbols
}

fn value(value: &AspectValue, symbols: &mut BTreeSet<Symbol>) {
    if let AspectValue::String(InternedString::Symbol(symbol)) = value {
        symbols.insert(Symbol(symbol.0));
    }
}
fn client_key(key: &super::ClientKey, symbols: &mut BTreeSet<Symbol>) {
    if let Some(symbol) = key.as_symbol() {
        symbols.insert(symbol);
    }
}
fn fields(patch: &AspectFieldPatch, symbols: &mut BTreeSet<Symbol>) {
    for (_, field) in patch.iter() {
        value(field, symbols);
    }
}
fn portable(patch: &PortableRecordAspectPatch, symbols: &mut BTreeSet<Symbol>) {
    for operation in patch.operations() {
        match operation {
            PortableAspectPatchOperation::SetWhole { value: input, .. } => match input {
                ContractValidationInput::Scalar(scalar) => value(scalar, symbols),
                ContractValidationInput::Struct(structure) => {
                    for (_, field) in structure.fields() {
                        value(field, symbols);
                    }
                }
            },
            PortableAspectPatchOperation::PatchFields { field_sets, .. } => {
                for field in field_sets {
                    value(field.value(), symbols);
                }
            }
            PortableAspectPatchOperation::ClearWhole { .. } => {}
        }
    }
}
fn visit_intent(intent: &MutationIntent, symbols: &mut BTreeSet<Symbol>) {
    match intent {
        MutationIntent::Create(create) => match create {
            CreateIntent::Entity(entity) => {
                client_key(&entity.client_key, symbols);
                fields(&entity.fields, symbols);
            }
            CreateIntent::Relation(relation) => {
                client_key(&relation.client_key, symbols);
                fields(&relation.fields, symbols);
            }
            CreateIntent::BulkEntities(bulk) => {
                for key in &bulk.client_keys {
                    client_key(key, symbols);
                }
                for patch in &bulk.field_patches {
                    fields(patch, symbols);
                }
            }
            CreateIntent::BulkRelations(bulk) => {
                for key in &bulk.client_keys {
                    client_key(key, symbols);
                }
                for patch in &bulk.field_patches {
                    fields(patch, symbols);
                }
            }
            CreateIntent::EntityAspects(entity) => {
                client_key(&entity.client_key, symbols);
                portable(&entity.aspect_patch, symbols);
            }
            CreateIntent::RelationAspects(relation) => {
                client_key(&relation.client_key, symbols);
                portable(&relation.aspect_patch, symbols);
            }
        },
        MutationIntent::Entity(entity) => match entity {
            EntityMutationIntent::UpdateFields(update) => fields(&update.fields, symbols),
            EntityMutationIntent::ApplyAspectPatch(update) => {
                portable(&update.aspect_patch, symbols)
            }
            EntityMutationIntent::Replace(update) => {
                client_key(&update.replacement.client_key, symbols);
                fields(&update.replacement.fields, symbols);
            }
            EntityMutationIntent::Delete(_) | EntityMutationIntent::Revalidate(_) => {}
        },
        MutationIntent::Relation(relation) => match relation {
            RelationMutationIntent::ApplyAspectPatch(update) => {
                portable(&update.aspect_patch, symbols)
            }
            RelationMutationIntent::UpdateEndpoints(_) | RelationMutationIntent::Delete(_) => {}
        },
        MutationIntent::Materialization(materialization) => match materialization {
            MaterializationMutationIntent::RematerializeEntity(entity) => {
                fields(&entity.fields, symbols)
            }
            MaterializationMutationIntent::RematerializeRelation(relation) => {
                fields(&relation.fields, symbols)
            }
            MaterializationMutationIntent::SuspendEntity(_)
            | MaterializationMutationIntent::SuspendRelation(_) => {}
        },
    }
}
