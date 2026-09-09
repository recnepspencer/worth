use std::sync::Arc;

use worth_runtime_bridge::facade::BridgeInstalledConditionalLowering;

use crate::recovery::ProductUnpublishedOwnerEffects;

use super::{NoEffectCompositePublication, PerformedCompositePublication};

/// Terminal result of publishing one Bridge conditional definition through
/// the product world.
pub enum RuntimeWorldConditionalDefinitionPublicationOutcome {
    Performed {
        publication: PerformedCompositePublication,
        lowering: Arc<BridgeInstalledConditionalLowering>,
    },
    NoEffect(NoEffectCompositePublication),
    ProductUnpublished(RuntimeWorldUnpublishedConditionalDefinition),
}

/// Product-unpublished owner evidence together with any Signal definition
/// custody created by the attempt.
pub struct RuntimeWorldUnpublishedConditionalDefinition {
    effects: ProductUnpublishedOwnerEffects,
}

impl RuntimeWorldUnpublishedConditionalDefinition {
    pub(crate) fn new(effects: ProductUnpublishedOwnerEffects) -> Self {
        Self { effects }
    }

    pub fn effects(&self) -> &ProductUnpublishedOwnerEffects {
        &self.effects
    }

    pub fn retains_signal_definition(&self) -> bool {
        self.effects.retains_signal_definition()
    }
}
