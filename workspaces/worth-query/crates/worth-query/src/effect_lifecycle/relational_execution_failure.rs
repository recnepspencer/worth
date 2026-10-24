use super::execution::EffectExecutionDenialKind;

/// The durable medium could not say whether the effect's commit became
/// durable. Nothing was installed live, but the commit may be durable, so this
/// is neither a denial nor a performed effect; only reopening the Relational
/// owner resolves it.
#[derive(Debug)]
pub struct RelationalEffectDurabilityUnresolved {
    message: String,
    unresolved: worth_relational::facade::durability::RelationalDurabilityUnresolved,
}

impl RelationalEffectDurabilityUnresolved {
    pub fn message(&self) -> &str {
        &self.message
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        String,
        worth_relational::facade::durability::RelationalDurabilityUnresolved,
    ) {
        (self.message, self.unresolved)
    }
}

#[derive(Debug)]
pub enum RelationalEffectExecutionFailure {
    Deferred {
        kind: super::EffectExecutionDeferredKind,
        message: String,
    },
    Denied {
        kind: EffectExecutionDenialKind,
        message: String,
    },
    ControlStopped {
        kind: super::EffectExecutionControlStopKind,
        message: String,
    },
    DurabilityUnresolved(RelationalEffectDurabilityUnresolved),
}

impl RelationalEffectExecutionFailure {
    pub fn deferred(kind: super::EffectExecutionDeferredKind, message: impl Into<String>) -> Self {
        Self::Deferred {
            kind,
            message: message.into(),
        }
    }

    pub(crate) fn durability_unresolved(
        unresolved: worth_relational::facade::durability::RelationalDurabilityUnresolved,
    ) -> Self {
        Self::DurabilityUnresolved(RelationalEffectDurabilityUnresolved {
            message: format!(
                "durable publication of position {} is unresolved: {}",
                unresolved.position().0,
                unresolved.detail()
            ),
            unresolved,
        })
    }

    pub fn message(&self) -> &str {
        match self {
            Self::Deferred { message, .. }
            | Self::Denied { message, .. }
            | Self::ControlStopped { message, .. } => message,
            Self::DurabilityUnresolved(unresolved) => unresolved.message(),
        }
    }
}

impl From<(EffectExecutionDenialKind, String)> for RelationalEffectExecutionFailure {
    fn from((kind, message): (EffectExecutionDenialKind, String)) -> Self {
        Self::Denied { kind, message }
    }
}
