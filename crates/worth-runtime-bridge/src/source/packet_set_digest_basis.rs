use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::snapshot::MaterializedTruthViewObservation;

use super::AdmittedSourceContract;

pub(crate) fn planned_packet_set_digest_from_observation(
    contract: &AdmittedSourceContract,
    observation: &MaterializedTruthViewObservation,
) -> Arc<str> {
    let canonical_basis = format!(
        "planned-source-read-packet-set|contract={}|validated={}|packets={}",
        contract.digest(),
        super::ValidatedSourceDeclaration::from_contract(contract).digest(),
        observation.planned().digest(),
    );
    let digest = Sha256::digest(canonical_basis.as_bytes());
    Arc::from(format!("planned-source-read-packet-set:sha256:{digest:x}"))
}

pub(crate) fn materialized_packet_set_digest_from_observation(
    planned_packet_set_digest: &str,
    observation: &MaterializedTruthViewObservation,
) -> Arc<str> {
    let canonical_basis = format!(
        "materialized-truth-view-packet-set|planned={}|observations={}|{}|{:?}|{}",
        planned_packet_set_digest,
        observation.planned().digest(),
        observation.snapshot_identity().canonical_label(),
        observation.materialization_path(),
        observation
            .snapshot_token()
            .snapshot_identity()
            .canonical_label(),
    );
    let digest = Sha256::digest(canonical_basis.as_bytes());
    Arc::from(format!(
        "materialized-truth-view-packet-set:sha256:{digest:x}"
    ))
}
