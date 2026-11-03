use crate::domain_computation::managed_run::tests::provisional_attempt_fixture::*;

pub fn proposed_change(
    action: &WorthQueryProvisionalEffectAction,
) -> (String, WorthQueryProposedFactOrigin, &'static str) {
    match action {
        WorthQueryProvisionalEffectAction::Create { symbolic_identity } => (
            symbolic_identity.to_string(),
            WorthQueryProposedFactOrigin::StagedCreation,
            "created",
        ),
        WorthQueryProvisionalEffectAction::Replace { target_identity } => (
            target_identity.to_string(),
            WorthQueryProposedFactOrigin::StagedReplacement,
            "replaced",
        ),
        WorthQueryProvisionalEffectAction::Retire { target_identity } => (
            target_identity.to_string(),
            WorthQueryProposedFactOrigin::StagedRetirement,
            "retired",
        ),
        WorthQueryProvisionalEffectAction::DeriveView { view_identity } => (
            view_identity.to_string(),
            WorthQueryProposedFactOrigin::DerivedProvisionalView,
            "derived",
        ),
    }
}
