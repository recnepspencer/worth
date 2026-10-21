//! An abandoned read sheds only touched paths, without retaining a parent root.

use super::*;

#[derive(Clone, Debug)]
pub(super) enum QueryReadBacking {
    Live(Arc<QueryReadProjection>),
    Released {
        keys: OrdMap<Arc<FactPostingKey>, OrdSet<usize>>,
        // Original shared nodes keep their original direct allocation grant.
        // New removal paths are admitted with the owning native state edit.
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
    ) -> Result<Option<Self>, CompanionPreflightStop> {
        let mut keys = match self {
            Self::Live(projection) => projection.keys.clone(),
            Self::Released { keys } => keys.clone(),
        };
        if keys.remove(key).is_none() {
            return Err(CompanionPreflightStop::SelectedSourceMismatch);
        }

        Ok((!keys.is_empty()).then_some(Self::Released { keys }))
    }
}
