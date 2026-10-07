use std::str::FromStr;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum FocusGroup {
    EntryAdmission,
    EntryRelease,
    EntryCustody,
    EntryResidencyPolicy,
    EntryCapacity,
    EntryPool,
    EntryPending,
    EntrySuccessor,
    EntryPublication,
    EntryMediaDenials,
    EntryLimits,
    PhaseOracle,
    PhaseCheckpoint,
    PhaseAgreement,
    PhaseRecovery,
    PhaseMutation,
    PhaseSuccessor,
    PhaseObserver,
}

impl FocusGroup {
    pub(crate) const ALL: &'static [Self] = &[
        Self::EntryAdmission,
        Self::EntryRelease,
        Self::EntryCustody,
        Self::EntryResidencyPolicy,
        Self::EntryCapacity,
        Self::EntryPool,
        Self::EntryPending,
        Self::EntrySuccessor,
        Self::EntryPublication,
        Self::EntryMediaDenials,
        Self::EntryLimits,
        Self::PhaseOracle,
        Self::PhaseCheckpoint,
        Self::PhaseAgreement,
        Self::PhaseRecovery,
        Self::PhaseMutation,
        Self::PhaseSuccessor,
        Self::PhaseObserver,
    ];

    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::EntryAdmission => "entry-admission",
            Self::EntryRelease => "entry-release",
            Self::EntryCustody => "entry-custody",
            Self::EntryResidencyPolicy => "entry-residency-policy",
            Self::EntryCapacity => "entry-capacity",
            Self::EntryPool => "entry-pool",
            Self::EntryPending => "entry-pending",
            Self::EntrySuccessor => "entry-successor",
            Self::EntryPublication => "entry-publication",
            Self::EntryMediaDenials => "entry-media-denials",
            Self::EntryLimits => "entry-limits",
            Self::PhaseOracle => "phase-oracle",
            Self::PhaseCheckpoint => "phase-checkpoint",
            Self::PhaseAgreement => "phase-agreement",
            Self::PhaseRecovery => "phase-recovery",
            Self::PhaseMutation => "phase-mutation",
            Self::PhaseSuccessor => "phase-successor",
            Self::PhaseObserver => "phase-observer",
        }
    }
}

impl FromStr for FocusGroup {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .iter()
            .copied()
            .find(|group| group.as_str() == value)
            .ok_or_else(|| {
                let choices = Self::ALL
                    .iter()
                    .map(|group| group.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("unknown focus group `{value}`; expected {choices}")
            })
    }
}
