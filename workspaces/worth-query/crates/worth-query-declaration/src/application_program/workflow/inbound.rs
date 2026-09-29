use std::any::TypeId;

use crate::application_schema::{
    ApplicationEffectMarkerIdentity, ApplicationInboundOccurrenceBinding,
    ApplicationInboundOccurrenceLimits, ApplicationInboundOccurrenceProtocol,
};

use super::{ApplicationWorkflowNodeIdentity, ApplicationWorkflowSpec};

/// The declared contract selected by a wait. Installation must match every
/// field to the origin operation's one installed inbound effect.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationWorkflowInboundRef {
    effect: String,
    effect_type: TypeId,
    protocol: ApplicationInboundOccurrenceProtocol,
    source_identity: String,
    limits: ApplicationInboundOccurrenceLimits,
}

impl ApplicationWorkflowInboundRef {
    pub fn declared<Spec, Effect>(binding: ApplicationInboundOccurrenceBinding<Effect>) -> Self
    where
        Spec: ApplicationWorkflowSpec,
        Effect: ApplicationEffectMarkerIdentity<Spec::Schema> + 'static,
    {
        Self {
            effect: Effect::IDENTIFIER.to_owned(),
            effect_type: TypeId::of::<Effect>(),
            protocol: binding.protocol().clone(),
            source_identity: binding.source_identity().to_owned(),
            limits: binding.limits(),
        }
    }

    pub fn effect(&self) -> &str {
        &self.effect
    }
    pub const fn effect_type(&self) -> TypeId {
        self.effect_type
    }
    pub const fn protocol(&self) -> &ApplicationInboundOccurrenceProtocol {
        &self.protocol
    }
    pub fn source_identity(&self) -> &str {
        &self.source_identity
    }
    pub const fn limits(&self) -> ApplicationInboundOccurrenceLimits {
        self.limits
    }

    pub(crate) fn from_member_provenance(
        effect: String,
        effect_type: TypeId,
        protocol: ApplicationInboundOccurrenceProtocol,
        source_identity: String,
        limits: ApplicationInboundOccurrenceLimits,
    ) -> Self {
        Self {
            effect,
            effect_type,
            protocol,
            source_identity,
            limits,
        }
    }
}

/// The wait never owns a clock. The instance's admitted deadline governs it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationWorkflowInboundWait {
    UntilInstanceDeadline,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationWorkflowAwaitInbound {
    origin: ApplicationWorkflowNodeIdentity,
    inbound: ApplicationWorkflowInboundRef,
    wait: ApplicationWorkflowInboundWait,
}

impl ApplicationWorkflowAwaitInbound {
    #[doc(hidden)]
    pub fn new(
        origin: ApplicationWorkflowNodeIdentity,
        inbound: ApplicationWorkflowInboundRef,
        wait: ApplicationWorkflowInboundWait,
    ) -> Self {
        Self {
            origin,
            inbound,
            wait,
        }
    }
    pub fn origin(&self) -> &ApplicationWorkflowNodeIdentity {
        &self.origin
    }
    pub fn inbound(&self) -> &ApplicationWorkflowInboundRef {
        &self.inbound
    }
    pub const fn wait(&self) -> ApplicationWorkflowInboundWait {
        self.wait
    }
}
