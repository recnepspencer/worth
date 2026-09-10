use worth_query_admission::facade::basis::{
    BasisFamily, BasisLifecyclePosture, NormalizedBasisIntent,
};
use worth_runtime_bridge::facade::BridgeAsyncRequestTruthViewBasisKind;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorthQueryManagedSemanticBasisDenial {
    Mismatch,
    Unsupported,
}

pub(crate) struct WorthQueryManagedSemanticBasisObservation<'a> {
    pub(crate) semantic: &'a NormalizedBasisIntent,
    pub(crate) bridge_kind: BridgeAsyncRequestTruthViewBasisKind,
    pub(crate) bridge_authority_basis_digest: &'a str,
    pub(crate) relational_current_at_admission: bool,
    pub(crate) product_observation:
        Option<&'a worth_runtime_world::facade::ProductBranchObservation>,
}

pub(crate) fn validate_managed_semantic_basis(
    observation: WorthQueryManagedSemanticBasisObservation<'_>,
) -> Result<(), WorthQueryManagedSemanticBasisDenial> {
    // Exact Product affinity is compared with the operation before lower
    // planning. The surviving observation keeps that World authority concrete.
    if observation.product_observation.is_some() {
        return Ok(());
    }
    match observation.semantic.family() {
        BasisFamily::CurrentHead => validate_current_head(observation),
        BasisFamily::BranchSnapshot => validate_branch_snapshot(observation),
        BasisFamily::RuntimeSnapshot => validate_runtime_snapshot(observation),
        BasisFamily::Preview
        | BasisFamily::PreviewDerived
        | BasisFamily::StoreBacked
        | BasisFamily::DurableReload => Err(WorthQueryManagedSemanticBasisDenial::Unsupported),
        _ => Err(WorthQueryManagedSemanticBasisDenial::Unsupported),
    }
}

fn validate_branch_snapshot(
    observation: WorthQueryManagedSemanticBasisObservation<'_>,
) -> Result<(), WorthQueryManagedSemanticBasisDenial> {
    if observation.semantic.lifecycle() == BasisLifecyclePosture::SnapshotPinned
        && observation
            .semantic
            .lower_runtime_binding_digest()
            .is_some()
        && observation.bridge_kind == BridgeAsyncRequestTruthViewBasisKind::BranchHead
    {
        Ok(())
    } else {
        Err(WorthQueryManagedSemanticBasisDenial::Mismatch)
    }
}

fn validate_current_head(
    observation: WorthQueryManagedSemanticBasisObservation<'_>,
) -> Result<(), WorthQueryManagedSemanticBasisDenial> {
    let current_kind = matches!(
        observation.bridge_kind,
        BridgeAsyncRequestTruthViewBasisKind::Authoritative
            | BridgeAsyncRequestTruthViewBasisKind::BranchHead
    );
    if observation.semantic.lifecycle() == BasisLifecyclePosture::Current
        && observation.relational_current_at_admission
        && current_kind
    {
        Ok(())
    } else {
        Err(WorthQueryManagedSemanticBasisDenial::Mismatch)
    }
}

fn validate_runtime_snapshot(
    observation: WorthQueryManagedSemanticBasisObservation<'_>,
) -> Result<(), WorthQueryManagedSemanticBasisDenial> {
    if observation.semantic.lifecycle() == BasisLifecyclePosture::SnapshotPinned
        && observation.semantic.lower_runtime_binding_digest()
            == Some(observation.bridge_authority_basis_digest)
    {
        Ok(())
    } else {
        Err(WorthQueryManagedSemanticBasisDenial::Mismatch)
    }
}

#[cfg(test)]
mod tests {
    use worth_query_admission::facade::basis::{
        normalize_raw_basis_intent, BasisOperationLane, ObservationLaneWitness, RawBasisIntent,
    };

    use super::*;

    #[test]
    fn current_head_requires_current_relational_and_nonhistorical_bridge_basis() {
        let semantic = normalized(RawBasisIntent::CurrentHead);
        assert_eq!(
            validate_managed_semantic_basis(observation(
                &semantic,
                BridgeAsyncRequestTruthViewBasisKind::BranchHead,
                true,
                "unused",
            )),
            Ok(())
        );
        assert_eq!(
            validate_managed_semantic_basis(observation(
                &semantic,
                BridgeAsyncRequestTruthViewBasisKind::Historical,
                true,
                "unused",
            )),
            Err(WorthQueryManagedSemanticBasisDenial::Mismatch)
        );
        assert_eq!(
            validate_managed_semantic_basis(observation(
                &semantic,
                BridgeAsyncRequestTruthViewBasisKind::BranchHead,
                false,
                "unused",
            )),
            Err(WorthQueryManagedSemanticBasisDenial::Mismatch)
        );
    }

    #[test]
    fn runtime_snapshot_requires_the_exact_bridge_authority_basis_digest() {
        let semantic = normalized(RawBasisIntent::RuntimeSnapshot {
            snapshot_identity: "snapshot-a".into(),
            lower_runtime_binding_digest: Some("bridge-basis-a".into()),
        });
        assert_eq!(
            validate_managed_semantic_basis(observation(
                &semantic,
                BridgeAsyncRequestTruthViewBasisKind::Authoritative,
                false,
                "bridge-basis-a",
            )),
            Ok(())
        );
        assert_eq!(
            validate_managed_semantic_basis(observation(
                &semantic,
                BridgeAsyncRequestTruthViewBasisKind::Authoritative,
                false,
                "bridge-basis-b",
            )),
            Err(WorthQueryManagedSemanticBasisDenial::Mismatch)
        );
    }

    #[test]
    fn pinned_branch_snapshot_does_not_require_current_head() {
        let semantic = normalized(RawBasisIntent::BranchSnapshot {
            branch_identity:
                crate::domain_computation::primary_graph::primary_relational_branch_id().0,
            snapshot_identity: "snapshot-v7".into(),
        });
        assert_eq!(
            validate_managed_semantic_basis(observation(
                &semantic,
                BridgeAsyncRequestTruthViewBasisKind::BranchHead,
                false,
                "already-joined-by-typed-lower-basis",
            )),
            Ok(())
        );
    }

    #[test]
    fn preview_and_store_families_cannot_enter_ordinary_managed_execution() {
        for semantic in [
            normalized(RawBasisIntent::Preview {
                preview_identity: "preview".into(),
                stale: false,
            }),
            normalized(RawBasisIntent::StoreBacked {
                store_basis_identity: "store".into(),
            }),
        ] {
            assert_eq!(
                validate_managed_semantic_basis(observation(
                    &semantic,
                    BridgeAsyncRequestTruthViewBasisKind::Preview,
                    false,
                    "unused",
                )),
                Err(WorthQueryManagedSemanticBasisDenial::Unsupported)
            );
        }
    }

    fn normalized(raw: RawBasisIntent) -> NormalizedBasisIntent {
        normalize_raw_basis_intent(raw, ObservationLaneWitness::lane_name())
            .expect("managed-run semantic fixture should normalize")
    }

    fn observation<'a>(
        semantic: &'a NormalizedBasisIntent,
        bridge_kind: BridgeAsyncRequestTruthViewBasisKind,
        relational_current_at_admission: bool,
        bridge_authority_basis_digest: &'a str,
    ) -> WorthQueryManagedSemanticBasisObservation<'a> {
        WorthQueryManagedSemanticBasisObservation {
            semantic,
            bridge_kind,
            bridge_authority_basis_digest,
            relational_current_at_admission,
            product_observation: None,
        }
    }
}
