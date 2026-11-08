//! Installed definition meaning carries the backing that owns its allocation.
use super::data::DerivedIndexDefinition;
use std::sync::Arc;

#[derive(Debug)]
pub(crate) struct IndexDefinitionBacking {
    pub(crate) _prior: Option<Arc<Self>>,
    pub(crate) _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl Drop for IndexDefinitionBacking {
    fn drop(&mut self) {
        let mut prior = self._prior.take();
        while let Some(backing) = prior {
            match Arc::try_unwrap(backing) {
                Ok(mut exclusive) => prior = exclusive._prior.take(),
                Err(_) => break,
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct RetainedIndexDefinitionMeaning {
    definition: DerivedIndexDefinition,
    _backing: Option<Arc<IndexDefinitionBacking>>,
}

/// Shared immutable definition data. This grants neither index installation
/// nor access to a generation; the receiver still admits the selected index.
#[derive(Debug, Clone)]
pub struct RetainedDerivedIndexDefinition {
    meaning: Arc<RetainedIndexDefinitionMeaning>,
}
impl RetainedDerivedIndexDefinition {
    /// Retain caller-owned descriptive data without granting runtime authority.
    pub fn from_owned(definition: DerivedIndexDefinition) -> Self {
        Self::funded(definition, None)
    }
    pub(crate) fn funded(
        definition: DerivedIndexDefinition,
        backing: Option<Arc<IndexDefinitionBacking>>,
    ) -> Self {
        Self {
            meaning: Arc::new(RetainedIndexDefinitionMeaning {
                definition,
                _backing: backing,
            }),
        }
    }
}
impl std::ops::Deref for RetainedDerivedIndexDefinition {
    type Target = DerivedIndexDefinition;
    fn deref(&self) -> &Self::Target {
        &self.meaning.definition
    }
}
impl AsRef<DerivedIndexDefinition> for RetainedDerivedIndexDefinition {
    fn as_ref(&self) -> &DerivedIndexDefinition {
        &self.meaning.definition
    }
}
impl PartialEq for RetainedDerivedIndexDefinition {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref() == other.as_ref()
    }
}
impl Eq for RetainedDerivedIndexDefinition {}

impl serde::Serialize for RetainedDerivedIndexDefinition {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.as_ref().serialize(serializer)
    }
}
