//! One decode's reservations move onto the exact shared schema text cells.
use super::{SchemaChangeText, SchemaDiffDetail};
use crate::schema::data::SchemaTransitionArtifact;
use worth_execution::ExecutionMemoryReservation;

pub(crate) struct SchemaTextReadmissionMemory {
    pub(crate) length: usize,
    pub(crate) memory: Option<ExecutionMemoryReservation>,
}
pub(crate) struct SchemaTransitionReadmissionMemory {
    pub(crate) texts: Vec<SchemaTextReadmissionMemory>,
    pub(crate) _tree_memory: Option<ExecutionMemoryReservation>,
}
impl SchemaTransitionReadmissionMemory {
    pub(crate) fn attach(
        &mut self,
        actual: Option<&SchemaTransitionArtifact>,
    ) -> Result<(), &'static str> {
        let mut texts = self.texts.iter_mut();
        if let Some(actual) = actual {
            for atom in &actual.diff_atoms {
                install(&mut texts, &atom.element.element_name)?;
                match &atom.detail {
                    SchemaDiffDetail::AddedField {
                        default_expression, ..
                    } => {
                        if let Some(text) = default_expression {
                            install(&mut texts, text)?;
                        }
                    }
                    SchemaDiffDetail::RemovedField { .. } => {}
                    SchemaDiffDetail::TypeChanged {
                        from_type, to_type, ..
                    } => {
                        install(&mut texts, from_type)?;
                        install(&mut texts, to_type)?;
                    }
                    SchemaDiffDetail::EnumDomainExpanded { added_variants, .. } => {
                        for text in added_variants {
                            install(&mut texts, text)?;
                        }
                    }
                    SchemaDiffDetail::InvariantContractChanged { contract_name }
                    | SchemaDiffDetail::SubscriberContractChanged { contract_name } => {
                        install(&mut texts, contract_name)?
                    }
                    SchemaDiffDetail::ProjectionContractChanged { projection_name } => {
                        install(&mut texts, projection_name)?
                    }
                    SchemaDiffDetail::FreeText { detail, .. } => install(&mut texts, detail)?,
                }
            }
        }
        if texts.next().is_some() {
            return Err("unmatched schema text admission");
        }
        Ok(())
    }
}
fn install<'a>(
    texts: &mut impl Iterator<Item = &'a mut SchemaTextReadmissionMemory>,
    actual: &SchemaChangeText,
) -> Result<(), &'static str> {
    let text = texts.next().ok_or("missing schema text admission")?;
    if actual.len() != text.length {
        return Err("schema text admission length mismatch");
    }
    if let Err(unused) = actual.retain_decoded_memory(text.memory.take()) {
        drop(unused);
    }
    Ok(())
}
