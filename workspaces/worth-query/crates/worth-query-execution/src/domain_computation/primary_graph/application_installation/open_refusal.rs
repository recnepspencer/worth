//! A refused open returns the home by the phase the refusal happened in.

use super::{WorthQueryApplicationOpenDenial, WorthQueryOpenAdoptionRecovery};
use crate::domain_computation::primary_graph::{ApplicationHome, WorthQueryApplicationCheckpoint};

/// One refused open: where the home is now, and why the open was refused.
#[derive(Debug)]
pub struct WorthQueryApplicationOpenRefusal {
    pub home: WorthQueryRefusedHome,
    pub denial: WorthQueryApplicationOpenDenial,
}

/// The home after a refused open.
#[derive(Debug)]
#[must_use = "a refused open still owns the application's home"]
pub enum WorthQueryRefusedHome {
    /// Refused before any effect. This is the home the open was given.
    Unchanged(ApplicationHome),
    /// The adoption was performed and acknowledged, then a later step refused.
    /// Reopening this home resumes at the successor.
    Successor(ApplicationHome),
    /// The adoption was performed but its settlement deferred or its capture
    /// stopped. There is no home until the capsule repairs into one.
    InRepair(WorthQueryOpenAdoptionRecovery),
}

impl std::fmt::Display for WorthQueryApplicationOpenRefusal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let phase = match &self.home {
            WorthQueryRefusedHome::Unchanged(_) => "home unchanged",
            WorthQueryRefusedHome::Successor(_) => "home at the adopted successor",
            WorthQueryRefusedHome::InRepair(_) => "home in adoption repair",
        };
        write!(formatter, "{} ({phase})", self.denial)
    }
}

impl std::error::Error for WorthQueryApplicationOpenRefusal {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.denial)
    }
}

/// A refusal raised inside open, before the caller's home is rejoined to it.
pub(in crate::domain_computation::primary_graph) struct OpenFailure {
    home: FailedHome,
    denial: WorthQueryApplicationOpenDenial,
}

enum FailedHome {
    Unchanged,
    Successor(WorthQueryApplicationCheckpoint),
    InRepair(WorthQueryOpenAdoptionRecovery),
}

impl From<WorthQueryApplicationOpenDenial> for OpenFailure {
    /// Every step before the adoption performs leaves the home as it was given.
    fn from(denial: WorthQueryApplicationOpenDenial) -> Self {
        Self {
            home: FailedHome::Unchanged,
            denial,
        }
    }
}

impl OpenFailure {
    pub(in crate::domain_computation::primary_graph) fn successor(
        image: WorthQueryApplicationCheckpoint,
        denial: WorthQueryApplicationOpenDenial,
    ) -> Self {
        Self {
            home: FailedHome::Successor(image),
            denial,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn in_repair(
        recovery: WorthQueryOpenAdoptionRecovery,
        denial: WorthQueryApplicationOpenDenial,
    ) -> Self {
        Self {
            home: FailedHome::InRepair(recovery),
            denial,
        }
    }

    /// Rejoins the failure to the home the open was given.
    pub(in crate::domain_computation::primary_graph) fn refuse(
        self,
        given: ApplicationHome,
    ) -> WorthQueryApplicationOpenRefusal {
        let home = match self.home {
            FailedHome::Unchanged => WorthQueryRefusedHome::Unchanged(given),
            FailedHome::Successor(image) => {
                WorthQueryRefusedHome::Successor(ApplicationHome::holding(image))
            }
            FailedHome::InRepair(recovery) => WorthQueryRefusedHome::InRepair(recovery),
        };
        WorthQueryApplicationOpenRefusal {
            home,
            denial: self.denial,
        }
    }
}

impl From<crate::facade::primary_graph::WorthQueryHandleDenial> for OpenFailure {
    fn from(denial: crate::facade::primary_graph::WorthQueryHandleDenial) -> Self {
        super::WorthQueryApplicationOpenDenial::Graph(denial.into()).into()
    }
}
