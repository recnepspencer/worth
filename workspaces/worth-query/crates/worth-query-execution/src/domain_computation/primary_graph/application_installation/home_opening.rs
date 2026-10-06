//! How one open found its home: started empty, resumed, or adopted.

use worth_query_declaration::facade::application_program::ApplicationProgramRevision;

use super::open_refusal::OpenFailure;
use super::{WorthQueryApplicationOpenDenial, WorthQueryOpenAdoptionPredecessor};
use crate::domain_computation::primary_graph::WorthQueryApplicationCheckpoint;

/// How the home started for the runtime this open returned.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryHomeOpening {
    /// The home was empty. The declared initial state ran, and the installed
    /// revision is this program's.
    Started,
    /// The image's activation named a revision in the admitted roster.
    Resumed {
        installed: ApplicationProgramRevision,
    },
    /// The image's activation named the adoption predecessor and the adoption ran.
    Adopted {
        from: WorthQueryOpenAdoptionPredecessor,
        installed: ApplicationProgramRevision,
    },
}

/// The start one open performed, with the successor image an adoption left.
pub(in crate::domain_computation::primary_graph) enum HomeStarted {
    Empty,
    DeclarationResumed,
    ProgramResumed {
        installed: ApplicationProgramRevision,
    },
    Adopted {
        from: WorthQueryOpenAdoptionPredecessor,
        installed: ApplicationProgramRevision,
        successor: WorthQueryApplicationCheckpoint,
    },
}

impl HomeStarted {
    /// The failure for a step refused after this start. An acknowledged
    /// adoption leaves the successor; every other start leaves the home unchanged.
    pub(in crate::domain_computation::primary_graph) fn refused(
        self,
        denial: WorthQueryApplicationOpenDenial,
    ) -> OpenFailure {
        match self {
            Self::Adopted { successor, .. } => OpenFailure::successor(successor, denial),
            Self::Empty | Self::DeclarationResumed | Self::ProgramResumed { .. } => denial.into(),
        }
    }

    /// The opening a program runtime reports. A declaration resume carries no
    /// installed revision, so it has no program opening.
    pub(in crate::domain_computation::primary_graph) fn program_opening(
        &self,
    ) -> Option<WorthQueryHomeOpening> {
        match self {
            Self::Empty => Some(WorthQueryHomeOpening::Started),
            Self::DeclarationResumed => None,
            Self::ProgramResumed { installed } => Some(WorthQueryHomeOpening::Resumed {
                installed: *installed,
            }),
            Self::Adopted {
                from, installed, ..
            } => Some(WorthQueryHomeOpening::Adopted {
                from: from.clone(),
                installed: *installed,
            }),
        }
    }
}
