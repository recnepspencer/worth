//! A request's walk refusal names the work it interrupted.
use super::{InvalidationEditAdmission, OccurrenceSearchCharge, WorthQueryOutputDemandDenial};
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind;

#[derive(Clone, Copy)]
pub(in super::super) enum WalkKind {
    Rejoin,
    Supersession,
    TerminalCleanup,
    CacheReclaim,
    JoinedReadyRelease,
    RequiredMember,
    PrepaidPass,
}

impl WalkKind {
    pub(in super::super) fn denial(self) -> WorthQueryOutputDemandDenial {
        let subject = match self {
            Self::Rejoin => "refreshed output rejoin exceeds request work",
            Self::Supersession => "supersession occurrence walk exceeds request work",
            Self::TerminalCleanup => "terminal cleanup occurrence walk exceeds request work",
            Self::CacheReclaim => "cache reclaim occurrence walk exceeds request work",
            Self::JoinedReadyRelease => "joined Ready release occurrence walk exceeds request work",
            Self::RequiredMember => "required member occurrence walk exceeds request work",
            Self::PrepaidPass => "prepaid occurrence walk has no paid pass",
        };
        WorthQueryOutputDemandDenial::new(
            WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
            subject,
        )
    }
}

pub(in super::super) struct WalkAdmission<'a> {
    admission: &'a mut InvalidationEditAdmission,
    kind: WalkKind,
}

impl<'a> WalkAdmission<'a> {
    pub(in super::super) fn new(
        admission: &'a mut InvalidationEditAdmission,
        kind: WalkKind,
    ) -> Self {
        Self { admission, kind }
    }

    pub(in super::super) fn admission_mut(&mut self) -> &mut InvalidationEditAdmission {
        self.admission
    }
}

impl OccurrenceSearchCharge for WalkAdmission<'_> {
    type Refusal = WorthQueryOutputDemandDenial;

    fn occurrence_path(&mut self, records: usize) -> Result<(), Self::Refusal> {
        self.admission
            .occurrence_path(records)
            .map_err(|_| self.kind.denial())
    }

    fn inspect_row(&mut self) -> Result<(), Self::Refusal> {
        self.admission.inspect_row().map_err(|_| self.kind.denial())
    }
}
