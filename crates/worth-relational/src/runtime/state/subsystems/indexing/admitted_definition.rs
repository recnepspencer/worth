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

/// Cost: at most eleven comparisons per B-tree level, plus absence.
pub(super) fn navigation(
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    entries: usize,
    width: usize,
) -> Result<(), worth_execution::MapKernelStop> {
    let work = navigation_work(entries, width)?;
    if context.has_accounting() {
        context.checkpoint(work)?;
    } else {
        context.poll()?;
    }
    Ok(())
}

fn navigation_work(entries: usize, width: usize) -> Result<u64, worth_execution::MapKernelStop> {
    u64::from(usize::BITS - entries.leading_zeros())
        .checked_mul(11)
        .and_then(|n| n.checked_mul((width as u64).checked_add(1)?))
        .and_then(|n| n.checked_add(1))
        .ok_or(worth_execution::MapKernelStop::WorkCounterOverflow)
}

#[cfg(test)]
mod tests {
    use super::navigation_work;

    #[test]
    fn catalog_navigation_grows_logarithmically_with_unrelated_definitions() {
        let small = navigation_work(128, 8).unwrap();
        let large = navigation_work(256, 8).unwrap();
        assert!(large * 2 < small * 3);
        assert!(navigation_work(usize::MAX, usize::MAX).is_err());
    }
}
