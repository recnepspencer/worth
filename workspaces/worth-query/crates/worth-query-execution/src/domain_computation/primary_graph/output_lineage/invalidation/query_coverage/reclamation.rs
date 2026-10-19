//! An abandoned read sheds only touched paths, without retaining a parent root.
use super::super::admission::{PreparedTreeEdits, RetainedIndexAdmission};
use super::*;

#[derive(Clone, Debug)]
pub(super) enum QueryReadBacking {
    Live(Arc<QueryReadProjection>),
    Released {
        keys: OrdMap<Arc<FactPostingKey>, OrdSet<usize>>,
        // Original shared nodes keep their original direct allocation grant.
        // New removal paths are admitted with the owning native state edit.
        _capacity: Capacity,
    },
}
impl QueryReadBacking {
    pub(super) fn keys(&self) -> &OrdMap<Arc<FactPostingKey>, OrdSet<usize>> {
        match self {
            Self::Live(projection) => &projection.keys,
            Self::Released { keys, .. } => keys,
        }
    }
    pub(super) fn belongs_to(&self, original: &Arc<QueryReadProjection>) -> bool {
        matches!(self, Self::Live(projection) if Arc::ptr_eq(projection, original))
    }
    pub(super) fn detach(
        &self,
        key: &FactPostingKey,
        admission: &mut impl RetainedIndexAdmission,
    ) -> Result<Option<Self>, CompanionPreflightStop> {
        let (mut keys, capacity) = match self {
            Self::Live(projection) => (projection.keys.clone(), Arc::clone(&projection._capacity)),
            Self::Released { keys, _capacity } => (keys.clone(), Arc::clone(_capacity)),
        };
        let mut edits = PreparedTreeEdits::<Arc<FactPostingKey>, OrdSet<usize>>::new();
        edits.key_remove(key, keys.len(), admission)?;
        if keys.remove(key).is_none() {
            return Err(CompanionPreflightStop::SelectedSourceMismatch);
        }
        edits.retain(keys.len(), admission)?;
        Ok((!keys.is_empty()).then_some(Self::Released {
            keys,
            _capacity: capacity,
        }))
    }
}
