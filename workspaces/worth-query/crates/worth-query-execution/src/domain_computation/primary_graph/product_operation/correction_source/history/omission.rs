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
    WorthQueryApplicationQueryAdmissionDenialKind as Admit,
    WorthQueryEntityResolutionDenialKind as Resolve,
    WorthQueryEntityResolutionRefusalKind as Refused,
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
        D::Admission(cause) => matches!(
            cause.kind(),
            Admit::Authorization(_) | Admit::StalePrincipal
        ),
        D::Read(cause) => matches!(cause.kind(), Read::Authorization(_) | Read::Projection(_)),
        _ => false,
    }
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
}
