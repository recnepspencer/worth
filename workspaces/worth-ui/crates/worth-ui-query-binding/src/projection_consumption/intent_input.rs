use std::sync::Arc;

use worth_query::facade::runtime::WorthQueryEvidenceIdentity;

use super::{
    UiApplicationScalarProjectionFactReceipt, UiCollectionCompleteness,
    UiCollectionProjectionChange, UiCollectionProjectionDelivery,
    UiCollectionProjectionFactReceipt, UiCollectionProjectionRowReference, UiProjectionFactReceipt,
    UiProjectionFactStopKind, UiProjectionRetainedActivityKind, UiProjectionUnavailableKind,
    UiScalarProjectionFactReceipt,
};
use worth_query_host::facade::publication::domain_computation::WorthQueryApplicationQueryPublicationReceipt;

#[path = "intent_input/collection_catalog.rs"]
mod collection_catalog;
#[path = "intent_input/collection_reference.rs"]
mod collection_reference;
#[path = "intent_input/collection_transition.rs"]
mod collection_transition;
#[path = "intent_input/scalar_input.rs"]
mod scalar_input;
#[path = "intent_input/transition_work.rs"]
mod transition_work;

use collection_catalog::UiProjectionInputCollectionCatalog;
pub use collection_reference::{
    UiProjectionInputCollectionRow, UiProjectionOptionReference, UiProjectionOptionStableKey,
};
pub use collection_transition::UiProjectionInputFactTransition;
use scalar_input::scalar_input;
pub use transition_work::UiProjectionInputTransitionWork;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UiProjectionInputSlot(u32);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiProjectionInputRevision {
    inner: Arc<UiProjectionInputRevisionInner>,
}

#[derive(Debug, Eq, PartialEq)]
struct UiProjectionInputRevisionInner {
    slot: UiProjectionInputSlot,
    projection: crate::WorthUiQueryViewIdentity,
    observation_order: u64,
    authority: UiProjectionInputAuthority,
}

#[derive(Debug, Eq, PartialEq)]
enum UiProjectionInputAuthority {
    Legacy {
        query_world: WorthQueryEvidenceIdentity,
        binding: WorthQueryEvidenceIdentity,
        source_generation: WorthQueryEvidenceIdentity,
        result_generation: WorthQueryEvidenceIdentity,
    },
    Application(WorthQueryApplicationQueryPublicationReceipt),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiProjectionInputPosture {
    Current,
    RetainedStale(UiProjectionRetainedActivityKind),
    Unavailable(UiProjectionUnavailableKind),
    Stopped(UiProjectionFactStopKind),
    TransitionStopped(UiProjectionInputTransitionStopKind),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiProjectionInputTransitionStopKind {
    MissingPredecessor,
    ProjectionChanged,
    WrongShape,
    PredecessorNotCurrent,
    MalformedPatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiProjectionInputFactReference {
    Scalar(Arc<UiScalarProjectionInputFact>),
    Collection(Arc<UiCollectionProjectionInputFact>),
}

#[derive(Debug, Eq, PartialEq)]
pub struct UiScalarProjectionInputFact {
    revision: UiProjectionInputRevision,
    posture: UiProjectionInputPosture,
    value: Option<Arc<str>>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct UiCollectionProjectionInputFact {
    revision: UiProjectionInputRevision,
    posture: UiProjectionInputPosture,
    completeness: Option<UiCollectionCompleteness>,
    catalog: Option<UiProjectionInputCollectionCatalog>,
    transition_work: UiProjectionInputTransitionWork,
}

impl UiProjectionInputRevision {
    fn from_fact(slot: UiProjectionInputSlot, fact: &UiProjectionFactReceipt) -> Self {
        Self {
            inner: Arc::new(UiProjectionInputRevisionInner {
                slot,
                projection: fact.projection_identity().clone(),
                observation_order: fact.observation_order(),
                authority: UiProjectionInputAuthority::Legacy {
                    query_world: fact.query_world_identity().clone(),
                    binding: fact.binding_identity().clone(),
                    source_generation: fact.source_generation_identity().clone(),
                    result_generation: fact.result_generation_identity().clone(),
                },
            }),
        }
    }

    fn from_application_fact(
        slot: UiProjectionInputSlot,
        fact: &UiApplicationScalarProjectionFactReceipt,
    ) -> Self {
        Self {
            inner: Arc::new(UiProjectionInputRevisionInner {
                slot,
                projection: fact.projection_identity().clone(),
                observation_order: fact.owner_order(),
                authority: UiProjectionInputAuthority::Application(fact.query_receipt().clone()),
            }),
        }
    }

    pub fn slot(&self) -> UiProjectionInputSlot {
        self.inner.slot
    }

    pub fn projection_identity(&self) -> &crate::WorthUiQueryViewIdentity {
        &self.inner.projection
    }

    pub fn observation_order(&self) -> u64 {
        self.inner.observation_order
    }

    pub fn query_world_identity(&self) -> Option<&WorthQueryEvidenceIdentity> {
        match &self.inner.authority {
            UiProjectionInputAuthority::Legacy { query_world, .. } => Some(query_world),
            UiProjectionInputAuthority::Application(_) => None,
        }
    }

    pub fn binding_identity(&self) -> Option<&WorthQueryEvidenceIdentity> {
        match &self.inner.authority {
            UiProjectionInputAuthority::Legacy { binding, .. } => Some(binding),
            UiProjectionInputAuthority::Application(_) => None,
        }
    }

    pub fn source_generation_identity(&self) -> Option<&WorthQueryEvidenceIdentity> {
        match &self.inner.authority {
            UiProjectionInputAuthority::Legacy {
                source_generation, ..
            } => Some(source_generation),
            UiProjectionInputAuthority::Application(_) => None,
        }
    }

    pub fn result_generation_identity(&self) -> Option<&WorthQueryEvidenceIdentity> {
        match &self.inner.authority {
            UiProjectionInputAuthority::Legacy {
                result_generation, ..
            } => Some(result_generation),
            UiProjectionInputAuthority::Application(_) => None,
        }
    }

    pub fn has_same_authority(&self, other: &Self) -> bool {
        match (&self.inner.authority, &other.inner.authority) {
            (
                UiProjectionInputAuthority::Legacy {
                    query_world: left_world,
                    binding: left_binding,
                    ..
                },
                UiProjectionInputAuthority::Legacy {
                    query_world: right_world,
                    binding: right_binding,
                    ..
                },
            ) => left_world == right_world && left_binding == right_binding,
            (
                UiProjectionInputAuthority::Application(left),
                UiProjectionInputAuthority::Application(right),
            ) => {
                let left = left.inspect();
                let right = right.inspect();
                left.query_identity() == right.query_identity()
                    && left.parameter_binding_identity() == right.parameter_binding_identity()
                    && left.basis().runtime_instance() == right.basis().runtime_instance()
                    && left.basis().branch() == right.basis().branch()
            }
            _ => false,
        }
    }

    pub(super) fn has_same_projection_owner(&self, other: &Self) -> bool {
        self.inner.slot == other.inner.slot
            && self.inner.projection == other.inner.projection
            && self.has_same_authority(other)
            && self.inner.observation_order < other.inner.observation_order
    }
}

impl UiProjectionInputFactReference {
    pub fn revision(&self) -> &UiProjectionInputRevision {
        match self {
            Self::Scalar(fact) => fact.revision(),
            Self::Collection(fact) => fact.revision(),
        }
    }

    pub fn posture(&self) -> UiProjectionInputPosture {
        match self {
            Self::Scalar(fact) => fact.posture(),
            Self::Collection(fact) => fact.posture(),
        }
    }
}

impl UiScalarProjectionInputFact {
    pub fn revision(&self) -> &UiProjectionInputRevision {
        &self.revision
    }

    pub fn posture(&self) -> UiProjectionInputPosture {
        self.posture
    }

    pub fn value(&self) -> Option<&str> {
        self.value.as_deref()
    }

    pub fn value_reference(&self) -> Option<Arc<str>> {
        self.value.as_ref().map(Arc::clone)
    }
}

impl UiCollectionProjectionInputFact {
    pub fn revision(&self) -> &UiProjectionInputRevision {
        &self.revision
    }

    pub fn posture(&self) -> UiProjectionInputPosture {
        self.posture
    }

    pub fn completeness(&self) -> Option<UiCollectionCompleteness> {
        self.completeness
    }

    pub fn row_count(&self) -> usize {
        self.catalog.as_ref().map_or(0, |catalog| catalog.len())
    }

    pub fn current_option(
        &self,
        row: &UiCollectionProjectionRowReference,
    ) -> Option<UiProjectionOptionReference> {
        if self.posture != UiProjectionInputPosture::Current {
            return None;
        }
        let catalog = self.catalog.as_ref()?;
        let (retained, _) = catalog.row(row.query_identity());
        retained.map(|retained| {
            UiProjectionOptionReference::query_issued(
                self.revision.clone(),
                retained.row().query_identity().clone(),
                retained.application_item_key(),
            )
        })
    }

    pub fn current_option_keys(&self) -> Option<Box<[UiProjectionOptionStableKey]>> {
        (self.posture == UiProjectionInputPosture::Current)
            .then(|| {
                self.catalog
                    .as_ref()
                    .map(|catalog| catalog.ordered_stable_keys())
            })
            .flatten()
    }

    pub fn current_application_item_keys(&self) -> Option<Box<[core::num::NonZeroU64]>> {
        (self.posture == UiProjectionInputPosture::Current)
            .then(|| {
                self.catalog
                    .as_ref()
                    .and_then(|catalog| catalog.ordered_application_item_keys())
            })
            .flatten()
    }

    pub fn transition_work(&self) -> UiProjectionInputTransitionWork {
        self.transition_work
    }
}

impl UiScalarProjectionFactReceipt {
    pub fn intent_input_transition(
        &self,
        slot: UiProjectionInputSlot,
    ) -> UiProjectionInputFactTransition {
        let revision = UiProjectionInputRevision::from_fact(slot, self.core());
        let (posture, value) = scalar_input(self.availability());
        UiProjectionInputFactTransition::replace(UiProjectionInputFactReference::Scalar(Arc::new(
            UiScalarProjectionInputFact {
                revision,
                posture,
                value,
            },
        )))
    }
}

impl UiApplicationScalarProjectionFactReceipt {
    pub fn intent_input_transition(
        &self,
        slot: UiProjectionInputSlot,
    ) -> UiProjectionInputFactTransition {
        let revision = UiProjectionInputRevision::from_application_fact(slot, self);
        let (posture, value) = if self.value().revision == 0 {
            (
                UiProjectionInputPosture::Unavailable(UiProjectionUnavailableKind::Pending),
                None,
            )
        } else {
            (
                UiProjectionInputPosture::Current,
                Some(Arc::from(self.value().status.as_str())),
            )
        };
        UiProjectionInputFactTransition::replace(UiProjectionInputFactReference::Scalar(Arc::new(
            UiScalarProjectionInputFact {
                revision,
                posture,
                value,
            },
        )))
    }
}

impl UiCollectionProjectionFactReceipt {
    pub fn intent_input_transition(
        &self,
        slot: UiProjectionInputSlot,
    ) -> UiProjectionInputFactTransition {
        collection_transition::from_fact(self, slot)
    }
}

impl UiProjectionInputSlot {
    pub(crate) fn from_index(index: usize) -> Option<Self> {
        u32::try_from(index).ok().map(Self)
    }

    pub const fn index(self) -> usize {
        self.0 as usize
    }

    #[cfg(any(test, feature = "certification-construction"))]
    pub const fn for_certification(index: u32) -> Self {
        Self(index)
    }
}
