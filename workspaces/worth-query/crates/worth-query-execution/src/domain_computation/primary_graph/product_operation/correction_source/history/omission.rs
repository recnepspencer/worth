//! Which selection refusals describe one entry rather than the listing.
//!
//! Only per-entry non-applicability omits an entry: it is not an ancestor
//! here, its retained history is gone, it is not declared correctable, or this
//! principal may not see it. Everything else (cancellation, deadlines, work,
//! memory, storage, publication in flight, retired branches, unavailable
//! owners, stale incarnations, foreign phases) propagates, so a listing never
//! reports a stopped read as an empty or shorter history.
use super::super::*;
use crate::facade::correction_material::QueryCorrectionMaterialDenial as Material;
use crate::facade::primary_graph::{
    WorthQueryApplicationOneShotDenialKind as Read,
    WorthQueryApplicationProjectionDenialKind as Projection,
    WorthQueryApplicationQueryAdmissionDenialKind as Admit,
    WorthQueryEntityResolutionDenialKind as Resolve,
    WorthQueryEntityResolutionRefusalKind as Refused,
    WorthQueryOperationAuthorizationDenialKind as Authorization,
    WorthQueryOperationAuthorizationRefusalKind as Policy,
};
use worth_runtime_world::facade::{
    CompositeAncestorSelectionDenial as History, ProductExactHistoryDenial,
};

pub(super) fn omits_entry(cause: &WorthQueryCorrectionSelectionDenial) -> bool {
    use WorthQueryCorrectionSelectionDenial as D;
    match cause {
        D::HistoryLimit
        | D::SourceUnavailable
        | D::SourceNotCorrectable
        | D::SourcePublicationMismatch
        | D::SourceMemberUnavailable => true,
        D::Material(cause) => material_omits(cause),
        D::ExactHistory(RuntimeWorldServiceDenial::Denied(ProductExactHistoryDenial::History(
            cause,
        ))) => history_omits(cause),
        D::Ancestry(RuntimeWorldServiceDenial::Denied(
            ProductBranchAncestorSelectionDenial::History(cause),
        )) => history_omits(cause),
        D::Disclosure(cause) => disclosure_omits(cause),
        D::ExactHistory(_)
        | D::Ancestry(_)
        | D::StaleIncarnation
        | D::HistorySource(_)
        | D::ForeignPhase
        | D::ForeignRuntime => false,
    }
}

fn history_omits(cause: &History) -> bool {
    match cause {
        History::NotAncestor
        | History::SourceUnavailable
        | History::SelectionLimitReached
        | History::RetiredHistoryGap => true,
        History::TargetUnavailable
        | History::Allocation(_)
        | History::Request(_)
        | History::Catalog(_) => false,
    }
}

/// Expired, unavailable or not-yet-ready material of this one writer.
fn material_omits(cause: &Material) -> bool {
    matches!(
        cause,
        Material::RetainedMaterialExpired
            | Material::Unavailable
            | Material::MemberUnavailable
            | Material::MaterialNotReady
            | Material::MaterialContentUnavailable { .. }
            | Material::NativeSourceUnavailable { .. }
            | Material::OriginalBindingUnavailable { .. }
            | Material::ProgramUnavailable { .. }
            | Material::PreparationDischarged { .. }
    )
}

/// Disclosure refusals that mean this principal may not see this one entry.
fn disclosure_omits(cause: &WorthQueryCorrectionDerivationDenial) -> bool {
    use WorthQueryCorrectionDerivationDenial as D;
    match cause {
        D::SubjectNotDisclosed
        | D::OriginalSubjectChanged
        | D::UnsupportedSourceMeaning
        | D::MissingPreImage
        | D::SemanticMaterialUnavailable => true,
        D::Selection(cause) => omits_entry(cause),
        D::Material(cause) => material_omits(cause),
        D::Scope(cause) => cause.kind() == Resolve::Refused(Refused::UnknownEntity),
        D::Admission(cause) => admission_omits(cause.kind()),
        D::Read(cause) => read_omits(cause.kind()),
        _ => false,
    }
}

fn admission_omits(kind: Admit) -> bool {
    matches!(kind, Admit::Authorization(Authorization::Refused(refusal)) if policy_omits(refusal))
}

fn read_omits(kind: Read) -> bool {
    match kind {
        Read::Authorization(Authorization::Refused(refusal)) => policy_omits(refusal),
        Read::Projection(projection) => matches!(
            projection,
            Projection::FieldOmitted
                | Projection::RelationOmitted
                | Projection::DomainProjectionRejected
        ),
        _ => false,
    }
}

/// Installed policy refused this principal for this one entry. Stops and
/// runtime, staleness or capacity refusals are never per-entry.
fn policy_omits(refusal: Policy) -> bool {
    matches!(
        refusal,
        Policy::CapabilityGrantMissing
            | Policy::CapabilityAuthorizationMissing
            | Policy::CapabilityRequired
            | Policy::CapabilityExpired
            | Policy::PurposeMismatch
            | Policy::ExplicitDenyRuleMatched
            | Policy::ConflictRuleMatched
            | Policy::SeparationOfDutyRuleMatched
            | Policy::DistinctActorRuleMatched
            | Policy::ElevationRequired
            | Policy::MandatoryReviewRejected
            | Policy::DelegationRejected
    )
}

#[cfg(test)]
mod tests {
    use super::omits_entry;
    use crate::domain_computation::primary_graph::product_operation::correction_source::{
        WorthQueryCorrectionDerivationDenial as Derivation,
        WorthQueryCorrectionSelectionDenial as Selection,
    };

    #[test]
    fn only_per_entry_non_applicability_is_omitted_and_stops_propagate() {
        for omitted in [
            Selection::SourceNotCorrectable,
            Selection::HistoryLimit,
            Selection::Disclosure(Derivation::SubjectNotDisclosed),
            Selection::Disclosure(Derivation::Selection(Box::new(
                Selection::SourceUnavailable,
            ))),
        ] {
            assert!(omits_entry(&omitted), "{omitted:?}");
        }
        for propagated in [
            Selection::StaleIncarnation,
            Selection::ForeignPhase,
            Selection::ForeignRuntime,
            Selection::Disclosure(Derivation::TargetIncarnationChanged),
            Selection::Disclosure(Derivation::ForeignPhase),
            Selection::Disclosure(Derivation::Selection(Box::new(Selection::ForeignRuntime))),
        ] {
            assert!(!omits_entry(&propagated), "{propagated:?}");
        }
    }

    #[test]
    fn stopped_authorization_and_decode_never_omit_an_entry() {
        use super::{admission_omits, read_omits};
        use crate::facade::primary_graph::{
            WorthQueryApplicationOneShotDenialKind as Read,
            WorthQueryApplicationProjectionDenialKind as Projection,
            WorthQueryApplicationQueryAdmissionDenialKind as Admit,
            WorthQueryOperationAuthorizationDenialKind as Authorization,
            WorthQueryOperationAuthorizationRefusalKind as Policy,
            WorthQueryProviderSessionControlStopKind as Control, WorthQueryValueDecodeDenial,
        };
        assert!(!admission_omits(Admit::StalePrincipal));
        assert!(!admission_omits(Admit::Authorization(
            Authorization::Stopped(Err(Control::Cancelled))
        )));
        assert!(!admission_omits(Admit::Authorization(
            Authorization::Refused(Policy::ForeignRuntime)
        )));
        assert!(!read_omits(Read::Authorization(Authorization::Stopped(
            Err(Control::TimedOut)
        ))));
        assert!(!read_omits(Read::Projection(Projection::ValueDecode(
            WorthQueryValueDecodeDenial::Checkpoint(worth_execution::MapKernelStop::Cancelled)
        ))));
        assert!(!read_omits(Read::Projection(
            Projection::PersistentNamesUnavailable
        )));
        assert!(admission_omits(Admit::Authorization(
            Authorization::Refused(Policy::CapabilityGrantMissing)
        )));
        assert!(read_omits(Read::Projection(Projection::FieldOmitted)));
    }
}
