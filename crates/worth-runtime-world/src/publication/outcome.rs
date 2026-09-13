use crate::recovery::ProductUnpublishedOwnerEffects;

use super::{
    NoEffectCompositePublication, OwnerExecutionSettlement, PerformedCompositePublication,
};

/// Closed internal owner-execution outcome. Product publication is not
/// synthesized from a no-effect or recovery result.
#[derive(Debug)]
pub(crate) enum OwnerExecutionOutcome {
    Settled(OwnerExecutionSettlement),
    NoEffect(NoEffectCompositePublication),
    ProductUnpublished(ProductUnpublishedOwnerEffects),
}

/// The only public terminal vocabulary for coordinated publication.
#[must_use = "publication outcomes carry performed work or a retained terminal posture"]
#[derive(Debug)]
pub enum RuntimeWorldPublicationOutcome {
    Performed(PerformedCompositePublication),
    NoEffect(NoEffectCompositePublication),
    ProductUnpublished(ProductUnpublishedOwnerEffects),
}
