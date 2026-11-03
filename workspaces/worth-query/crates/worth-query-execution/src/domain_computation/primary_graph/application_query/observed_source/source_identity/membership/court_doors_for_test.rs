//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryObservedSourceSelection {
    #[doc(hidden)]
    pub fn new_for_test(runtime_authority: u64, schema_binding: ApplicationSchemaBindingIdentity, meaning: Arc<WorthQueryObservedSourceMeaning>, managed: Option<Arc<ManagedSourceRead>>) -> Self {
        Self { runtime_authority, schema_binding, meaning, managed }
    }
}
