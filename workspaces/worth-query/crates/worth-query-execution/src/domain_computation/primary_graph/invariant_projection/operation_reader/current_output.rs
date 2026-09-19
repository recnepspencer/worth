use std::marker::PhantomData;

use super::super::WorthQueryInvariantEntityIdentity;

mod resolution;

pub struct WorthQueryCurrentOutputRole<Family, Entity> {
    name: &'static str,
    _marker: PhantomData<fn() -> (Family, Entity)>,
}

impl<Family, Entity> Copy for WorthQueryCurrentOutputRole<Family, Entity> {}

impl<Family, Entity> Clone for WorthQueryCurrentOutputRole<Family, Entity> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Family, Entity> WorthQueryCurrentOutputRole<Family, Entity> {
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            _marker: PhantomData,
        }
    }

    pub const fn name(self) -> &'static str {
        self.name
    }
}

pub enum WorthQueryCurrentOutputSelection<Schema, Entity> {
    Unique(WorthQueryInvariantEntityIdentity<Schema, Entity>),
    Missing,
    Ambiguous(Vec<WorthQueryInvariantEntityIdentity<Schema, Entity>>),
    ObsoleteSource,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryCurrentOutputDenialKind {
    FamilyUnavailable,
    StaleSource,
    EntityMismatch,
    OutputUnavailable,
    UndeclaredDecisionTarget,
    ForeignIdentity,
    WorkBudgetExceeded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryCurrentOutputDenial {
    kind: WorthQueryCurrentOutputDenialKind,
    subject: String,
}

impl WorthQueryCurrentOutputDenial {
    pub const fn kind(&self) -> WorthQueryCurrentOutputDenialKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }

    fn new(kind: WorthQueryCurrentOutputDenialKind, subject: impl Into<String>) -> Self {
        Self {
            kind,
            subject: subject.into(),
        }
    }
}

impl std::fmt::Display for WorthQueryCurrentOutputDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "current output denied: {:?} ({})",
            self.kind, self.subject
        )
    }
}

impl std::error::Error for WorthQueryCurrentOutputDenial {}
