use std::collections::BTreeMap;

use worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus;

use super::WorthQueryInstalledApplicationValueBinding;

/// Immutable installed bindings indexed by their exact field locus.
#[derive(Clone, Debug, Default)]
pub struct WorthQueryInstalledApplicationValueBindingCatalog {
    bindings: BTreeMap<ApplicationFieldBindingLocus, WorthQueryInstalledApplicationValueBinding>,
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
