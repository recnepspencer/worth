#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum WorthQueryMutationTargetClass {
    Collection,
    Entity,
}

impl WorthQueryMutationTargetClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Collection => "collection",
            Self::Entity => "entity",
        }
    }
}

impl std::fmt::Display for WorthQueryMutationTargetClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryMutationTargetDescriptor {
    target_class: WorthQueryMutationTargetClass,
    collection: Option<WorthQueryMutationTargetCollectionIdentity>,
    entity_identity: Option<WorthQueryEntityIdentity>,
}

impl WorthQueryMutationTargetDescriptor {
    pub(in crate::runtime) fn new(
        target_class: WorthQueryMutationTargetClass,
        collection: Option<WorthQueryMutationTargetCollectionIdentity>,
        entity_identity: Option<WorthQueryEntityIdentity>,
    ) -> Self {
        Self {
            target_class,
            collection,
            entity_identity,
        }
    }

    pub fn target_class(&self) -> WorthQueryMutationTargetClass {
        self.target_class
    }

    pub fn collection(&self) -> Option<&WorthQueryMutationTargetCollectionIdentity> {
        self.collection.as_ref()
    }

    pub fn entity_identity(&self) -> Option<&WorthQueryEntityIdentity> {
        self.entity_identity.as_ref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryMutationTargetEvidence {
    declared: WorthQueryMutationTargetDescriptor,
    resolved: WorthQueryMutationTargetDescriptor,
}

impl WorthQueryMutationTargetEvidence {
    pub(in crate::runtime) fn new(
        declared: WorthQueryMutationTargetDescriptor,
        resolved: WorthQueryMutationTargetDescriptor,
    ) -> Self {
        Self { declared, resolved }
    }

    pub fn declared(&self) -> &WorthQueryMutationTargetDescriptor {
        &self.declared
    }

    pub fn resolved(&self) -> &WorthQueryMutationTargetDescriptor {
        &self.resolved
    }

    #[cfg(test)]
    pub(crate) fn test_only(
        target_class: WorthQueryMutationTargetClass,
        collection: Option<&str>,
        entity_identity: Option<WorthQueryEntityIdentity>,
    ) -> Self {
        let descriptor = WorthQueryMutationTargetDescriptor {
            target_class,
            collection: collection.map(|collection| {
                WorthQueryMutationTargetCollectionIdentity::new("mutation-target", collection)
                    .expect("fixture digest must fit")
            }),
            entity_identity,
        };
        Self {
            declared: descriptor.clone(),
            resolved: descriptor,
        }
    }
}
use crate::memory_workspace::WorthQueryEntityIdentity;
use crate::runtime::WorthQueryMutationTargetCollectionIdentity;
