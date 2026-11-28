use std::collections::BTreeMap;
use std::fmt;
use std::panic::{RefUnwindSafe, UnwindSafe};
use std::sync::Arc;

use worth_query_declaration::facade::application_schema::{
    fragment::FragmentCatalog, ApplicationFieldBindingLocus, ApplicationFieldBindingOwnerKind,
};

use super::exclusion_groups::WorthQueryInstalledAspectExclusionGroup;
use super::WorthQueryInstalledApplicationValueBinding;

type InstalledBindingsMap =
    BTreeMap<ApplicationFieldBindingLocus, Arc<WorthQueryInstalledApplicationValueBinding>>;

trait FragmentCatalogBacking: Send + Sync + UnwindSafe + RefUnwindSafe {
    fn as_catalog(&self) -> &FragmentCatalog;
}

impl FragmentCatalogBacking for FragmentCatalog {
    fn as_catalog(&self) -> &FragmentCatalog {
        self
    }
}

trait FormatsBacking: Send + Sync + UnwindSafe + RefUnwindSafe {
    fn as_formats(&self) -> &super::content::Formats;
}

impl FormatsBacking for super::content::Formats {
    fn as_formats(&self) -> &super::content::Formats {
        self
    }
}

trait InstalledBindingsBacking: Send + Sync + UnwindSafe + RefUnwindSafe {
    fn as_bindings(&self) -> &InstalledBindingsMap;
}

impl InstalledBindingsBacking for InstalledBindingsMap {
    fn as_bindings(&self) -> &InstalledBindingsMap {
        self
    }
}

/// Immutable installed bindings indexed by their exact field locus.
#[derive(Clone)]
pub struct WorthQueryInstalledApplicationValueBindingCatalog {
    fragments: Arc<dyn FragmentCatalogBacking>,
    exclusion_groups: Arc<BTreeMap<String, Vec<WorthQueryInstalledAspectExclusionGroup>>>,
    content_formats: Arc<dyn FormatsBacking>,
    bindings: Arc<dyn InstalledBindingsBacking>,
}

impl Default for WorthQueryInstalledApplicationValueBindingCatalog {
    fn default() -> Self {
        Self {
            fragments: Arc::new(FragmentCatalog::default()),
            exclusion_groups: Arc::default(),
            content_formats: Arc::new(super::content::Formats::default()),
            bindings: Arc::new(InstalledBindingsMap::default()),
        }
    }
}

impl fmt::Debug for WorthQueryInstalledApplicationValueBindingCatalog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WorthQueryInstalledApplicationValueBindingCatalog")
            .field("fragments", self.fragments_ref())
            .field("content_formats", self.content_formats_ref())
            .field("bindings", self.bindings_ref())
            .finish()
    }
}

#[derive(Debug)]
pub enum AdmittedFieldBindingStop<E> {
    Admission(E),
    AccountingOverflow,
}

impl WorthQueryInstalledApplicationValueBindingCatalog {
    fn fragments_ref(&self) -> &FragmentCatalog {
        FragmentCatalogBacking::as_catalog(&*self.fragments)
    }

    fn content_formats_ref(&self) -> &super::content::Formats {
        FormatsBacking::as_formats(&*self.content_formats)
    }

    fn bindings_ref(&self) -> &InstalledBindingsMap {
        InstalledBindingsBacking::as_bindings(&*self.bindings)
    }

    /// Whether both catalogues retain the same immutable installed backing.
    ///
    /// Cloning retains this identity; independently installing equal bindings
    /// does not. This comparison grants no request, snapshot or publication
    /// authority.
    pub fn same_installation_as(&self, other: &Self) -> bool {
        std::ptr::eq(self.bindings_ref(), other.bindings_ref())
            && std::ptr::eq(self.content_formats_ref(), other.content_formats_ref())
            && std::ptr::eq(self.fragments_ref(), other.fragments_ref())
    }

    pub(crate) fn new(
        content_formats: super::content::Formats,
        bindings: BTreeMap<
            ApplicationFieldBindingLocus,
            WorthQueryInstalledApplicationValueBinding,
        >,
        fragments: FragmentCatalog,
    ) -> Self {
        let bindings: InstalledBindingsMap = bindings
            .into_iter()
            .map(|(key, value)| (key, Arc::new(value)))
            .collect();
        Self {
            exclusion_groups: Arc::new(super::exclusion_groups::lower(&fragments)),
            fragments: Arc::new(fragments),
            content_formats: Arc::new(content_formats),
            bindings: Arc::new(bindings),
        }
    }

    pub fn fragment<F: worth_query_declaration::facade::application_schema::fragment::Fragment>(
        &self,
        marker: F,
    ) -> Option<&worth_query_declaration::facade::application_schema::fragment::FragmentDefinition>
    {
        self.fragments_ref().fragment(marker)
    }

    pub fn exclusion_groups<'a>(
        &'a self,
        entity: &str,
    ) -> impl Iterator<
        Item = (
            &'a str,
            &'a std::collections::BTreeSet<worth_foundational::facade::AspectKey>,
            bool,
        ),
    > + 'a {
        self.exclusion_groups
            .get(entity)
            .into_iter()
            .flatten()
            .map(|group| (group.group_id.as_str(), &group.aspects, group.required))
    }

    pub fn fragments(&self) -> &FragmentCatalog {
        self.fragments_ref()
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
        self.bindings_ref().get(&locus).map(Arc::as_ref)
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
        let count = self.bindings_ref().len();
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
        Ok(self.bindings_ref().get(&locus))
    }

    pub fn content_formats(&self) -> impl ExactSizeIterator<Item = &worth_query_declaration::facade::application_schema::ApplicationContentBindingRecipe>{
        self.content_formats_ref().values().map(Arc::as_ref)
    }
    pub fn bindings(
        &self,
    ) -> impl ExactSizeIterator<Item = &WorthQueryInstalledApplicationValueBinding> {
        self.bindings_ref().values().map(Arc::as_ref)
    }

    pub fn len(&self) -> usize {
        self.bindings_ref().len()
    }

    pub fn is_empty(&self) -> bool {
        self.bindings_ref().is_empty()
    }
}
