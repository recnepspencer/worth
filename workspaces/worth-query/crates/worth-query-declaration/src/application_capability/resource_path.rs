//! Authority-free resource extents. Native admission must prove the complete anchored path.
use super::{
    ApplicationCapabilityContextEntitySlotBinding, ApplicationCapabilityContextEntitySlotRef,
    ApplicationCapabilityRelationBinding,
};
use crate::application_schema::{
    ApplicationAuthorizationTraversal as Traversal,
    ApplicationAuthorizationTraversalDirection as Direction, ApplicationRelationRef,
    WorthQueryPortableApplicationAuthorizationTraversalParts as TraversalParts,
};
use std::marker::PhantomData;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
/// Describes a direct grant resource or a context-anchored path to a descendant scope.
/// Neither this declaration nor its parent slot grants access; native admission proves the path.
pub struct ApplicationCapabilityResourcePath {
    head: ApplicationCapabilityRelationBinding,
    root: Option<ApplicationCapabilityContextEntitySlotBinding>,
    traversals: Vec<Traversal>,
}
impl From<ApplicationCapabilityRelationBinding> for ApplicationCapabilityResourcePath {
    fn from(head: ApplicationCapabilityRelationBinding) -> Self {
        Self {
            head,
            root: None,
            traversals: Vec::new(),
        }
    }
}
impl ApplicationCapabilityResourcePath {
    /// Begin at the indexed grant-resource relation, anchoring its reached resource to a context slot.
    pub fn root<Schema, Relation, Grant, Root, Context, Slot>(
        relation: ApplicationRelationRef<Schema, Relation, Grant, Root>,
        slot: ApplicationCapabilityContextEntitySlotRef<Schema, Context, Slot, Root>,
    ) -> ApplicationCapabilityResourcePathBuilder<Schema, Root> {
        ApplicationCapabilityResourcePathBuilder {
            path: Self {
                head: ApplicationCapabilityRelationBinding::from_reference(relation),
                root: Some(ApplicationCapabilityContextEntitySlotBinding::from_reference(slot)),
                traversals: Vec::new(),
            },
            marker: PhantomData,
        }
    }
    /// Descriptive reconstruction only. Cold schema admission validates closure and supported posture.
    pub fn from_untrusted_parts(
        head: ApplicationCapabilityRelationBinding,
        root: Option<ApplicationCapabilityContextEntitySlotBinding>,
        traversals: Vec<Traversal>,
    ) -> Self {
        Self {
            head,
            root,
            traversals,
        }
    }
    pub fn head(&self) -> &ApplicationCapabilityRelationBinding {
        &self.head
    }
    pub fn root_slot(&self) -> Option<&ApplicationCapabilityContextEntitySlotBinding> {
        self.root.as_ref()
    }
    pub fn traversals(&self) -> &[Traversal] {
        &self.traversals
    }
    pub fn is_direct(&self) -> bool {
        self.root.is_none() && self.traversals.is_empty()
    }
    pub fn scope_entity(&self) -> &str {
        self.traversals
            .last()
            .map_or(self.head.to(), |step| match step.direction() {
                Direction::Forward => step.to(),
                Direction::Reverse => step.from(),
            })
    }
}

pub struct ApplicationCapabilityResourcePathBuilder<Schema, Current> {
    path: ApplicationCapabilityResourcePath,
    marker: PhantomData<fn() -> (Schema, Current)>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplicationCapabilityResourcePathDenial {
    EmptyExtent,
}

pub(super) fn append_canonical(
    components: &mut Vec<super::ApplicationCapabilityCanonicalComponent>,
    path: &ApplicationCapabilityResourcePath,
) {
    use super::canonical_components::{structural_count, text};
    if path.is_direct() {
        return;
    }
    text(
        components,
        "target.resource-path.profile",
        "context-rooted-v1",
    );
    if let Some(slot) = path.root_slot() {
        for (name, value) in [
            ("context", slot.context()),
            ("context-type", slot.context_identity().as_str()),
            ("slot", slot.slot()),
            ("slot-type", slot.slot_identity().as_str()),
            ("entity", slot.entity()),
        ] {
            text(
                components,
                format!("target.resource-path.root.{name}"),
                value,
            );
        }
    }
    structural_count(
        components,
        "target.resource-path.step-count",
        path.traversals().len(),
    );
    for (index, step) in path.traversals().iter().enumerate() {
        for (name, value) in [
            ("relation", step.relation()),
            ("from", step.from()),
            ("to", step.to()),
            (
                "direction",
                match step.direction() {
                    Direction::Forward => "forward",
                    Direction::Reverse => "reverse",
                },
            ),
        ] {
            text(
                components,
                format!("target.resource-path.step.{index}.{name}"),
                value,
            );
        }
    }
}
impl<Schema, Current> ApplicationCapabilityResourcePathBuilder<Schema, Current> {
    pub fn forward<Relation, Next>(
        self,
        relation: ApplicationRelationRef<Schema, Relation, Current, Next>,
    ) -> ApplicationCapabilityResourcePathBuilder<Schema, Next> {
        self.step(
            relation.name(),
            relation.from(),
            relation.to(),
            Direction::Forward,
        )
    }
    pub fn reverse<Relation, Previous>(
        self,
        relation: ApplicationRelationRef<Schema, Relation, Previous, Current>,
    ) -> ApplicationCapabilityResourcePathBuilder<Schema, Previous> {
        self.step(
            relation.name(),
            relation.from(),
            relation.to(),
            Direction::Reverse,
        )
    }
    fn step<Next>(
        mut self,
        relation: &str,
        from: &str,
        to: &str,
        direction: Direction,
    ) -> ApplicationCapabilityResourcePathBuilder<Schema, Next> {
        self.path
            .traversals
            .push(Traversal::from_untrusted_parts(TraversalParts {
                relation: relation.into(),
                from: from.into(),
                to: to.into(),
                direction,
            }));
        ApplicationCapabilityResourcePathBuilder {
            path: self.path,
            marker: PhantomData,
        }
    }
    pub fn finish(
        self,
    ) -> Result<ApplicationCapabilityResourcePath, ApplicationCapabilityResourcePathDenial> {
        if self.path.traversals.is_empty() {
            return Err(ApplicationCapabilityResourcePathDenial::EmptyExtent);
        }
        Ok(self.path)
    }
}
