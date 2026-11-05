use std::collections::BTreeMap;
use std::sync::Arc;

use worth_query_declaration::facade::application_schema::{
    ApplicationFieldBindingLocus, ApplicationFieldBindingOwnerKind,
};

use super::WorthQueryInstalledApplicationValueBinding;

/// Immutable installed bindings indexed by their exact field locus.
#[derive(Clone, Debug, Default)]
pub struct WorthQueryInstalledApplicationValueBindingCatalog {
    fragments: Arc<worth_query_declaration::facade::application_schema::fragment::FragmentCatalog>,
    content_formats: Arc<super::content::Formats>,
    bindings: Arc<
        BTreeMap<ApplicationFieldBindingLocus, Arc<WorthQueryInstalledApplicationValueBinding>>,
    >,
}

#[derive(Debug)]
pub enum AdmittedFieldBindingStop<E> {
    Admission(E),
    AccountingOverflow,
}

impl WorthQueryInstalledApplicationValueBindingCatalog {
    /// Whether both catalogues retain the same immutable installed backing.
    ///
    /// Cloning retains this identity; independently installing equal bindings
    /// does not. This comparison grants no request, snapshot or publication
    /// authority.
    pub fn same_installation_as(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.bindings, &other.bindings)
            && Arc::ptr_eq(&self.content_formats, &other.content_formats)
            && Arc::ptr_eq(&self.fragments, &other.fragments)
    }

    pub(crate) fn new(
        content_formats: super::content::Formats,
        bindings: BTreeMap<
            ApplicationFieldBindingLocus,
            WorthQueryInstalledApplicationValueBinding,
        >,
        fragments: worth_query_declaration::facade::application_schema::fragment::FragmentCatalog,
    ) -> Self {
        Self {
            fragments: Arc::new(fragments),
            content_formats: Arc::new(content_formats),
            bindings: Arc::new(
                bindings
                    .into_iter()
                    .map(|(key, value)| (key, Arc::new(value)))
                    .collect(),
            ),
        }
    }

    pub fn fragment<F: worth_query_declaration::facade::application_schema::fragment::Fragment>(
        &self,
        marker: F,
    ) -> Option<&worth_query_declaration::facade::application_schema::fragment::FragmentDefinition>
    {
        self.fragments.fragment(marker)
    }

    pub fn field(
        &self,
        entity: &str,
        aspect: &str,
        field: &str,
    ) -> Option<&WorthQueryInstalledApplicationValueBinding> {
        self.field_for_owner(
            ApplicationFieldBindingOwnerKind::Entity,
            entity,
            aspect,
            field,
        )
    }

    pub fn relation_field(
        &self,
        relation: &str,
        aspect: &str,
        field: &str,
    ) -> Option<&WorthQueryInstalledApplicationValueBinding> {
        self.field_for_owner(
            ApplicationFieldBindingOwnerKind::Relation,
            relation,
            aspect,
            field,
        )
    }

    fn field_for_owner(
        &self,
        owner: ApplicationFieldBindingOwnerKind,
        name: &str,
        aspect: &str,
        field: &str,
    ) -> Option<&WorthQueryInstalledApplicationValueBinding> {
        let locus = match owner {
            ApplicationFieldBindingOwnerKind::Entity => {
                ApplicationFieldBindingLocus::new(name, aspect, field)
            }
            ApplicationFieldBindingOwnerKind::Relation => {
                ApplicationFieldBindingLocus::for_relation(name, aspect, field)
            }
        };
        self.bindings.get(&locus).map(Arc::as_ref)
    }

    /// Inspect the existing immutable catalogue after funding its lookup and
    /// temporary key storage through the caller's request authority.
    pub fn field_admitted<E>(
        &self,
        entity: &str,
        aspect: &str,
        field: &str,
        prepare: &mut impl FnMut(u64, u64) -> Result<(), E>,
    ) -> Result<Option<&WorthQueryInstalledApplicationValueBinding>, AdmittedFieldBindingStop<E>>
    {
        self.field_for_owner_admitted(
            ApplicationFieldBindingOwnerKind::Entity,
            entity,
            aspect,
            field,
            prepare,
        )
    }

    pub fn relation_field_admitted<E>(
        &self,
        relation: &str,
        aspect: &str,
        field: &str,
        prepare: &mut impl FnMut(u64, u64) -> Result<(), E>,
    ) -> Result<Option<&WorthQueryInstalledApplicationValueBinding>, AdmittedFieldBindingStop<E>>
    {
        self.field_for_owner_admitted(
            ApplicationFieldBindingOwnerKind::Relation,
            relation,
            aspect,
            field,
            prepare,
        )
    }

    /// Retain the actual immutable entry after the same admitted field lookup.
    pub fn shared_field_admitted<E>(
        &self,
        entity: &str,
        aspect: &str,
        field: &str,
        prepare: &mut impl FnMut(u64, u64) -> Result<(), E>,
    ) -> Result<Option<Arc<WorthQueryInstalledApplicationValueBinding>>, AdmittedFieldBindingStop<E>>
    {
        self.entry_for_owner_admitted(
            ApplicationFieldBindingOwnerKind::Entity,
            entity,
            aspect,
            field,
            prepare,
        )
        .map(|entry| entry.cloned())
    }

    fn field_for_owner_admitted<E>(
        &self,
        owner: ApplicationFieldBindingOwnerKind,
        entity: &str,
        aspect: &str,
        field: &str,
        prepare: &mut impl FnMut(u64, u64) -> Result<(), E>,
    ) -> Result<Option<&WorthQueryInstalledApplicationValueBinding>, AdmittedFieldBindingStop<E>>
    {
        self.entry_for_owner_admitted(owner, entity, aspect, field, prepare)
            .map(|entry| entry.map(Arc::as_ref))
    }

    fn entry_for_owner_admitted<E>(
        &self,
        owner: ApplicationFieldBindingOwnerKind,
        entity: &str,
        aspect: &str,
        field: &str,
        prepare: &mut impl FnMut(u64, u64) -> Result<(), E>,
    ) -> Result<Option<&Arc<WorthQueryInstalledApplicationValueBinding>>, AdmittedFieldBindingStop<E>>
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
        let locus = match owner {
            ApplicationFieldBindingOwnerKind::Entity => {
                ApplicationFieldBindingLocus::new(entity, aspect, field)
            }
            ApplicationFieldBindingOwnerKind::Relation => {
                ApplicationFieldBindingLocus::for_relation(entity, aspect, field)
            }
        };
        Ok(self.bindings.get(&locus))
    }

    pub fn content_formats(&self) -> impl ExactSizeIterator<Item = &worth_query_declaration::facade::application_schema::ApplicationContentBindingRecipe>{
        self.content_formats.values().map(Arc::as_ref)
    }
    pub fn bindings(
        &self,
    ) -> impl ExactSizeIterator<Item = &WorthQueryInstalledApplicationValueBinding> {
        self.bindings.values().map(Arc::as_ref)
    }

    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }
}
