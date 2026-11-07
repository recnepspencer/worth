use worth_runtime_bridge::facade::{
    BridgeCausalEnvelopeAssemblyRequest, BridgeCausalEvidenceFamily,
    BridgeCausalEvidenceReferenceIdentity, BridgeIdentityEvidence,
};

use super::super::super::*;
use super::materialization::support::*;

fn receipt_with_route(
    outcome: CausalObservationOutcome,
    route_identity: BridgeIdentityEvidence,
) -> QueryObservationReceipt {
    QueryObservationReceipt::fixture(
        outcome,
        vec![
            CausalObservationEvidenceIdentity::new(
                CausalEvidenceFamily::QueryInspection,
                crate::runtime::tests::causal_inspection::causal_test_reference_digest(format!(
                    "query-inspection:{}",
                    outcome.as_str()
                )),
            )
            .expect("fixture digest must fit"),
            CausalObservationEvidenceIdentity::new(
                CausalEvidenceFamily::BridgeRoute,
                route_identity,
            )
            .expect("fixture digest must fit"),
        ],
    )
    .expect("fixture digest must fit")
}

mod common_observation_paths;
mod support_postures;
mod temporal_async;
