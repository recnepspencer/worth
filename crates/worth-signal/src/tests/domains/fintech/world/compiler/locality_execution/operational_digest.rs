use std::collections::BTreeMap;

use crate::data::error::SignalError;
use crate::tests::domains::fintech::world::compiler::locality_execution::CompiledFinancialLocalityWorld;
use sha2::{Digest, Sha256};
use worth_foundational::facade::{
    canonicalization, prepare_canonical_basis_sequence, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisValue, CanonicalDigestAlgorithmId,
    CanonicalDigestId, CanonicalizationRuleVersion,
};
use worth_proof::TransitionOutcome;

const DOMAIN: CanonicalBasisDomain =
    CanonicalBasisDomain::Future("WORTH.signal.m10.operational-authority");

impl CompiledFinancialLocalityWorld {
    pub(crate) fn operational_digest_without_observation_work(
        &self,
    ) -> Result<CanonicalDigestId, SignalError> {
        self.operational_digest_with_work(&BTreeMap::new())
    }

    pub(super) fn operational_digest_with_work(
        &self,
        performed_bindings: &super::FinancialPerformedCanonicalWork,
    ) -> Result<CanonicalDigestId, SignalError> {
        let graph = self.runtime.graph();
        let mut financial_values = self
            .committed_financial_values()?
            .into_iter()
            .map(|(output, value)| format!("value:{}={value}", output.ordinal()))
            .collect::<Vec<_>>();
        financial_values.sort();
        let mut values = vec![format!("financial-values={financial_values:?}")];
        values.push(format!(
            "readiness_epoch={:?}",
            graph.current_invalidation_readiness_epoch()
        ));
        values.push(format!(
            "published_commit_order={:?}",
            graph.published_output_commit_order_for_test()
        ));
        let current_branch = self.runtime.current_branch();
        values.push(format!(
            "branch-head={:?};known-branches={:?}",
            current_branch,
            self.runtime.known_branches()
        ));
        let mut node_rows = Vec::new();
        for (output, node) in &self.handles {
            let snapshot = graph.get_dep_snapshot(*node)?;
            let invalidation = graph.node_invalidation_input(*node)?;
            let operational_summary = graph.node_runtime_artifact_operational_summary(*node)?;
            let reuse_boundary = graph.node_runtime_artifact_reuse_boundary_snapshot(*node)?;
            let output_identity = graph
                .node_runtime_artifact_warm(*node)?
                .and_then(|warm| warm.output_identity.as_ref());
            node_rows.push(format!(
                "node:{output:?}:ordinal={};state={:?};revision={:?};version={:?};identity={output_identity:?};snapshot={snapshot:?};invalidation={invalidation:?};reuse={operational_summary:?};boundary={reuse_boundary:?}",
                output.ordinal(),
                graph.get_state(*node)?,
                graph.dependency_revision(*node)?,
                graph.node_aspect_version(*node)?,
            ));
        }
        node_rows.sort();
        values.push(format!(
            "node-ledger-digest={}",
            digest_rows("node", &node_rows)
        ));
        let mut performed_work = Vec::new();
        for (identity, count) in performed_bindings {
            performed_work.push(format!(
                "performed-work:target={:?};revision={};origin={:?};epoch={};stage={};count={count}",
                identity.axes().1,
                identity.axes().2,
                identity.axes().3,
                identity.axes().4,
                identity.axes().5,
            ));
        }
        // Keep every normalized work identity in the digest while representing
        // the work ledger as one canonical entry. A long locality run may
        // legitimately contain more than the canonicalizer's 4,096-entry
        // admission budget; the work identities themselves remain complete
        // and sorted inside this single bounded entry.
        performed_work.sort();
        values.push(format!(
            "performed-work-ledger-digest={}",
            digest_rows("performed-work", &performed_work)
        ));
        values.sort();
        let basis = values.into_iter().enumerate().map(|(ordinal, value)| {
            CanonicalBasisEntry::new(
                DOMAIN,
                CanonicalBasisLocus::Named(format!("operational.{ordinal}").into()),
                CanonicalBasisEntryKind::Identity,
                CanonicalBasisValue::ExactText(value.into()),
            )
        });
        let ready = match prepare_canonical_basis_sequence(
            CanonicalizationRuleVersion::new("1").expect("valid M10 digest rule"),
            DOMAIN,
            basis,
        ) {
            TransitionOutcome::Success(ready) => ready,
            denied => {
                return Err(SignalError::internal(format!(
                    "operational digest basis denied: {denied:?}"
                )))
            }
        };
        let digest_ready = match canonicalization().digest().for_sequence_with_budget(
            ready,
            CanonicalDigestAlgorithmId::sha256(),
            self.resources.digest_budget,
        ) {
            TransitionOutcome::Success(ready) => ready,
            denied => {
                return Err(SignalError::internal(format!(
                    "operational digest derivation denied: {denied:?}"
                )))
            }
        };
        let digest = canonicalization().digest().derive(digest_ready);
        Ok(CanonicalDigestId::new(*digest.value().bytes()))
    }
}

fn digest_rows(label: &str, rows: &[String]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(label.as_bytes());
    for row in rows {
        hasher.update((row.len() as u64).to_le_bytes());
        hasher.update(row.as_bytes());
    }
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
