use crate::application::{
    WorthQueryDeclarationAdmissionError, WorthQueryDeclarationAdmissionOrLegalityError,
    WorthQueryDeclarationEntryProgressionError, WorthQueryDeclarationFamilyMarker,
    WorthQueryDeclarationInput, WorthQueryDeclarationLegalityDenial,
    WorthQueryDeclarationProgressionTerminalError, WorthQueryDomainEntryMarker,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryGroupedDeclarationStopKind {
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
    Deferred,
    Unsupported,
    InvalidContext,
    Canonicalization,
    Denied,
    Stale,
    RebindRequired,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGroupedDeclarationStop {
    member_index: usize,
    declaration_family_key: &'static str,
    stop_kind: WorthQueryGroupedDeclarationStopKind,
    reason: String,
}

impl WorthQueryGroupedDeclarationStop {
    pub(super) fn new(
        member_index: usize,
        declaration_family_key: &'static str,
        stop_kind: WorthQueryGroupedDeclarationStopKind,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            member_index,
            declaration_family_key,
            stop_kind,
            reason: reason.into(),
        }
    }

    pub fn member_index(&self) -> usize {
        self.member_index
    }

    pub fn declaration_family_key(&self) -> &'static str {
        self.declaration_family_key
    }

    pub fn stop_kind(&self) -> WorthQueryGroupedDeclarationStopKind {
        self.stop_kind
    }

    pub fn reason(&self) -> &str {
        &self.reason
    }
}

pub(super) fn grouped_declaration_stop<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
>(
    member_index: usize,
    error: &WorthQueryDeclarationEntryProgressionError<D, I>,
) -> WorthQueryGroupedDeclarationStop {
    match error {
        WorthQueryDeclarationEntryProgressionError::EvidenceIdentityDerivationDenied(cause) => {
            WorthQueryGroupedDeclarationStop::new(
                member_index,
                I::Family::semantic_family_key(),
                WorthQueryGroupedDeclarationStopKind::EvidenceIdentityDerivationDenied(*cause),
                "canonical evidence identity refused",
            )
        }
        WorthQueryDeclarationEntryProgressionError::Entry(entry) => {
            grouped_declaration_entry_stop(member_index, entry)
        }
        WorthQueryDeclarationEntryProgressionError::Progression(progression) => {
            grouped_progression_stop(member_index, progression)
        }
    }
}

fn grouped_declaration_entry_stop<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
>(
    member_index: usize,
    error: &WorthQueryDeclarationAdmissionOrLegalityError<D, I>,
) -> WorthQueryGroupedDeclarationStop {
    match error {
        WorthQueryDeclarationAdmissionOrLegalityError::EvidenceIdentityDerivationDenied(cause) => {
            WorthQueryGroupedDeclarationStop::new(
                member_index,
                I::Family::semantic_family_key(),
                WorthQueryGroupedDeclarationStopKind::EvidenceIdentityDerivationDenied(*cause),
                "canonical evidence identity refused",
            )
        }
        WorthQueryDeclarationAdmissionOrLegalityError::Admission(admission) => {
            grouped_admission_stop(member_index, admission)
        }
        WorthQueryDeclarationAdmissionOrLegalityError::Legality(legality) => {
            grouped_legality_stop(member_index, legality)
        }
    }
}

fn grouped_admission_stop<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>(
    member_index: usize,
    error: &WorthQueryDeclarationAdmissionError<D, I>,
) -> WorthQueryGroupedDeclarationStop {
    match error {
WorthQueryDeclarationAdmissionError::EvidenceIdentityDerivationDenied(cause) => WorthQueryGroupedDeclarationStop::new(member_index, I::Family::semantic_family_key(), WorthQueryGroupedDeclarationStopKind::EvidenceIdentityDerivationDenied(*cause), "canonical evidence identity refused"),
        WorthQueryDeclarationAdmissionError::Deferred(denial) => WorthQueryGroupedDeclarationStop::new(
            member_index,
            I::Family::semantic_family_key(),
            WorthQueryGroupedDeclarationStopKind::Deferred,
            format!(
                "member {member_index} declaration deferred with capability status {}",
                denial.capability_status().as_str()
            ),
        ),
        WorthQueryDeclarationAdmissionError::AsyncDeferred(denial) => WorthQueryGroupedDeclarationStop::new(
            member_index,
            I::Family::semantic_family_key(),
            WorthQueryGroupedDeclarationStopKind::Deferred,
            format!(
                "member {member_index} declaration deferred because async support is {}",
                denial.async_support().as_str()
            ),
        ),
        WorthQueryDeclarationAdmissionError::TemporalDeferred(denial) => WorthQueryGroupedDeclarationStop::new(
            member_index,
            I::Family::semantic_family_key(),
            WorthQueryGroupedDeclarationStopKind::Deferred,
            format!(
                "member {member_index} declaration deferred because temporal support is {}",
                denial.temporal_support().as_str()
            ),
        ),
        WorthQueryDeclarationAdmissionError::Unsupported(denial) => WorthQueryGroupedDeclarationStop::new(
            member_index,
            I::Family::semantic_family_key(),
            WorthQueryGroupedDeclarationStopKind::Unsupported,
            format!(
                "member {member_index} declaration unsupported with capability status {}",
                denial.capability_status().as_str()
            ),
        ),
        WorthQueryDeclarationAdmissionError::AsyncUnsupported(denial) => WorthQueryGroupedDeclarationStop::new(
            member_index,
            I::Family::semantic_family_key(),
            WorthQueryGroupedDeclarationStopKind::Unsupported,
            format!(
                "member {member_index} declaration unsupported because async support is {}",
                denial.async_support().as_str()
            ),
        ),
        WorthQueryDeclarationAdmissionError::TemporalUnsupported(denial) => WorthQueryGroupedDeclarationStop::new(
            member_index,
            I::Family::semantic_family_key(),
            WorthQueryGroupedDeclarationStopKind::Unsupported,
            format!(
                "member {member_index} declaration unsupported because temporal support is {}",
                denial.temporal_support().as_str()
            ),
        ),
        WorthQueryDeclarationAdmissionError::InvalidContext(denial) => WorthQueryGroupedDeclarationStop::new(
            member_index,
            I::Family::semantic_family_key(),
            WorthQueryGroupedDeclarationStopKind::InvalidContext,
            format!(
                "member {member_index} declaration invalid in the admitted context with capability status {}",
                denial.capability_status().as_str()
            ),
        ),
        WorthQueryDeclarationAdmissionError::Canonicalization(error) => WorthQueryGroupedDeclarationStop::new(
            member_index,
            I::Family::semantic_family_key(),
            WorthQueryGroupedDeclarationStopKind::Canonicalization,
            format!("member {member_index} canonicalization failed: {error:?}"),
        ),
    }
}

fn grouped_legality_stop<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>(
    member_index: usize,
    error: &WorthQueryDeclarationLegalityDenial<D, I>,
) -> WorthQueryGroupedDeclarationStop {
    match error {
        WorthQueryDeclarationLegalityDenial::DeferredByLegalityBoundary { .. } => {
            WorthQueryGroupedDeclarationStop::new(
                member_index,
                I::Family::semantic_family_key(),
                WorthQueryGroupedDeclarationStopKind::Deferred,
                format!("member {member_index} declaration deferred by legality boundary"),
            )
        }
        WorthQueryDeclarationLegalityDenial::UnsupportedLegalityClass { .. } => {
            WorthQueryGroupedDeclarationStop::new(
                member_index,
                I::Family::semantic_family_key(),
                WorthQueryGroupedDeclarationStopKind::Unsupported,
                format!("member {member_index} declaration uses an unsupported legality class"),
            )
        }
        WorthQueryDeclarationLegalityDenial::TemporalProjectionUnsupported { kind, .. }
            if kind.is_deferred() =>
        {
            WorthQueryGroupedDeclarationStop::new(
                member_index,
                I::Family::semantic_family_key(),
                WorthQueryGroupedDeclarationStopKind::Deferred,
                format!("member {member_index} declaration deferred by temporal legality boundary"),
            )
        }
        WorthQueryDeclarationLegalityDenial::AsyncProjectionUnsupported { kind, .. }
            if kind.is_deferred() =>
        {
            WorthQueryGroupedDeclarationStop::new(
                member_index,
                I::Family::semantic_family_key(),
                WorthQueryGroupedDeclarationStopKind::Deferred,
                format!("member {member_index} declaration deferred by async legality boundary"),
            )
        }
        WorthQueryDeclarationLegalityDenial::WrongAdmittedWorld { .. }
        | WorthQueryDeclarationLegalityDenial::IllegalRoleClaim { .. }
        | WorthQueryDeclarationLegalityDenial::IllegalSurfaceDisposition { .. }
        | WorthQueryDeclarationLegalityDenial::TemporalProjectionUnsupported { .. }
        | WorthQueryDeclarationLegalityDenial::AsyncProjectionUnsupported { .. } => {
            WorthQueryGroupedDeclarationStop::new(
                member_index,
                I::Family::semantic_family_key(),
                WorthQueryGroupedDeclarationStopKind::InvalidContext,
                format!("member {member_index} declaration failed legality review"),
            )
        }
    }
}

fn grouped_progression_stop<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>(
    member_index: usize,
    error: &WorthQueryDeclarationProgressionTerminalError<D, I>,
) -> WorthQueryGroupedDeclarationStop {
    let (kind, reason) = match error {
        WorthQueryDeclarationProgressionTerminalError::EvidenceIdentityDerivationDenied(cause) => (
            WorthQueryGroupedDeclarationStopKind::EvidenceIdentityDerivationDenied(*cause),
            "canonical evidence identity refused",
        ),
        WorthQueryDeclarationProgressionTerminalError::Deferred(_) => (
            WorthQueryGroupedDeclarationStopKind::Deferred,
            "declaration progression deferred",
        ),
        WorthQueryDeclarationProgressionTerminalError::Denied(_) => (
            WorthQueryGroupedDeclarationStopKind::Denied,
            "declaration progression denied",
        ),
        WorthQueryDeclarationProgressionTerminalError::Stale(_) => (
            WorthQueryGroupedDeclarationStopKind::Stale,
            "declaration progression went stale",
        ),
        WorthQueryDeclarationProgressionTerminalError::RebindRequired(_) => (
            WorthQueryGroupedDeclarationStopKind::RebindRequired,
            "declaration progression requires rebind",
        ),
        WorthQueryDeclarationProgressionTerminalError::Failed(_) => (
            WorthQueryGroupedDeclarationStopKind::Failed,
            "declaration progression failed",
        ),
    };
    WorthQueryGroupedDeclarationStop::new(
        member_index,
        I::Family::semantic_family_key(),
        kind,
        format!("member {member_index} {reason}"),
    )
}
