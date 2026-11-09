//! One live Source audience per declared recipe in the actual schema registry.
use super::*;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, Weak};

static NEXT_PROGRAM_ORDER: AtomicU64 = AtomicU64::new(0);

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
        predecessor: Option<crate::identity::data::KindId>,
    ) -> Result<ManagedIndexSourceProgram, Denial> {
        let mut bindings = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let identity = &path.parts().identity;
        if let Some(previous) = bindings.upgrade() {
            if previous.predecessor != predecessor
                || previous.path != *path
                || previous.fields.as_ref() != fields.as_slice()
                || previous.routes.as_ref() != routes.as_slice()
            {
                return Err(Denial::DuplicateDefinition(identity.clone()));
            }
            return Ok(ManagedIndexSourceProgram(previous));
        }
        let order = mint_program_order(&NEXT_PROGRAM_ORDER)?;
        let issuer = ManagedIndexSourceAudienceIssuer::fresh();
        let authority = issuer.authority();
        let program = Arc::new(SourceProgram {
            order,
            path: path.clone(),
            predecessor,
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

fn mint_program_order(counter: &AtomicU64) -> Result<u64, Denial> {
    let mut current = counter.load(Ordering::Relaxed);
    loop {
        let next = current
            .checked_add(1)
            .ok_or(Denial::SourceProgramIdentityExhausted)?;
        match counter.compare_exchange_weak(current, next, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => return Ok(current),
            Err(observed) => current = observed,
        }
    }
}

#[cfg(test)]
mod tests;
