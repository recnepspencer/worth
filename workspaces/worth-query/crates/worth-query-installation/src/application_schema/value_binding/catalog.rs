use std::collections::BTreeMap;

use worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus;

use super::WorthQueryInstalledApplicationValueBinding;

/// Immutable installed bindings indexed by their exact field locus.
#[derive(Clone, Debug, Default)]
pub struct WorthQueryInstalledApplicationValueBindingCatalog {
    bindings: BTreeMap<ApplicationFieldBindingLocus, WorthQueryInstalledApplicationValueBinding>,
}

pub(crate) enum AdmittedFieldBindingStop<E> {
    Admission(E),
    AccountingOverflow,
}

impl WorthQueryInstalledApplicationValueBindingCatalog {
    pub(crate) fn new(
        bindings: BTreeMap<
            ApplicationFieldBindingLocus,
            WorthQueryInstalledApplicationValueBinding,
        >,
    ) -> Self {
        Self { bindings }
    }

    pub fn field(
        &self,
        entity: &str,
        aspect: &str,
        field: &str,
    ) -> Option<&WorthQueryInstalledApplicationValueBinding> {
        self.bindings
            .get(&ApplicationFieldBindingLocus::new(entity, aspect, field))
    }

    pub(crate) fn field_admitted<E>(
        &self,
        entity: &str,
        aspect: &str,
        field: &str,
        prepare: &mut impl FnMut(u64, u64) -> Result<(), E>,
    ) -> Result<Option<&WorthQueryInstalledApplicationValueBinding>, AdmittedFieldBindingStop<E>>
    {
        let input = entity
            .len()
            .checked_add(aspect.len())
            .and_then(|n| n.checked_add(field.len()))
            .ok_or(AdmittedFieldBindingStop::AccountingOverflow)?;
        let per_key = input
            .checked_add(3)
            .ok_or(AdmittedFieldBindingStop::AccountingOverflow)?;
        let count = self.bindings.len();
        let levels = usize::BITS.saturating_sub(count.max(1).leading_zeros()) as usize;
        let comparisons = count
            .min(11)
            .checked_mul(levels)
            .ok_or(AdmittedFieldBindingStop::AccountingOverflow)?;
        let work = comparisons
            .checked_mul(per_key)
            .and_then(|n| n.checked_add(input))
            .and_then(|n| n.checked_add(2))
            .and_then(|n| u64::try_from(n).ok())
            .ok_or(AdmittedFieldBindingStop::AccountingOverflow)?;
        let bytes =
            u64::try_from(input).map_err(|_| AdmittedFieldBindingStop::AccountingOverflow)?;
        prepare(work, bytes).map_err(AdmittedFieldBindingStop::Admission)?;
        Ok(self.field(entity, aspect, field))
    }

    pub fn bindings(
        &self,
    ) -> impl ExactSizeIterator<Item = &WorthQueryInstalledApplicationValueBinding> {
        self.bindings.values()
    }

    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }
}
