//! One live Source audience per declared recipe in the actual schema registry.
use super::*;
use std::sync::{Mutex, Weak};
#[derive(Debug, Default)]
pub(crate) struct SourceBindingCache(Mutex<Weak<SourceProgram>>);
// Memoized handles do not change RelationalSchemaRegistry's descriptive
// equality. Its registered definitions remain the source of schema meaning.
impl PartialEq for SourceBindingCache {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}
impl Eq for SourceBindingCache {}
impl SourceBindingCache {
    pub(super) fn bind(
        &self,
        path: &DerivedRelationHandle,
        fields: Vec<ManagedIndexSourceField>,
        routes: Vec<ManagedIndexSourceProjectionRoute>,
    ) -> Result<ManagedIndexSourceProgram, Denial> {
        let mut bindings = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let identity = &path.parts().identity;
        if let Some(previous) = bindings.upgrade() {
            if previous.path != *path
                || previous.fields.as_ref() != fields.as_slice()
                || previous.routes.as_ref() != routes.as_slice()
            {
                return Err(Denial::DuplicateDefinition(identity.clone()));
            }
            return Ok(ManagedIndexSourceProgram(previous));
        }
        let issuer = ManagedIndexSourceAudienceIssuer::fresh();
        let authority = issuer.authority();
        let program = Arc::new(SourceProgram {
            path: path.clone(),
            fields: fields.into(),
            routes: routes.into(),
            issuer,
            authority,
        });
        // This path owns one weak cell. It neither pins a Source audience nor
        // accumulates names or weak buckets after independent schema cloning.
        *bindings = Arc::downgrade(&program);
        Ok(ManagedIndexSourceProgram(program))
    }
}

#[cfg(test)]
mod tests;
