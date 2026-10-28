//! Original-request admission for borrowed installed field-index definitions.
use super::*;

impl IndexDefinitionCatalog {
    /// Prepay both actual definition-map descents before borrowing the matching
    /// roster. The callback keeps original request custody and cannot mutate it.
    pub(crate) fn with_field_admitted<R, E: From<worth_execution::MapKernelStop>>(
        &self,
        aspect: &AspectKey,
        field: &FieldKey,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
        read: impl FnOnce(
            &[Arc<DerivedIndexDefinition>],
            &mut worth_execution::MapKernelContext<'_, '_>,
        ) -> Result<R, E>,
    ) -> Result<R, E> {
        navigation(context, self.field_definitions.len(), aspect.as_str().len())?;
        let Some(fields) = self.field_definitions.get(aspect) else {
            return read(&[], context);
        };
        navigation(context, fields.len(), field.as_str().len())?;
        read(fields.get(field).map_or(&[], Vec::as_slice), context)
    }
}

/// Portable BTree upper bound: at most every occupied key plus absence.
pub(super) fn navigation(
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    entries: usize,
    width: usize,
) -> Result<(), worth_execution::MapKernelStop> {
    let work = (entries as u64)
        .checked_add(1)
        .and_then(|n| n.checked_mul((width as u64).checked_add(1)?))
        .ok_or(worth_execution::MapKernelStop::WorkCounterOverflow)?;
    if context.has_accounting() {
        context.checkpoint(work)?;
    } else {
        context.poll()?;
    }
    Ok(())
}
