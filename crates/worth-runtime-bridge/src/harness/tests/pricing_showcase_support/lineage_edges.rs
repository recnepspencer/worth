use super::super::pricing_support::PricingWorkloadCertificationBundle;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(in crate::harness::tests) struct PricingLineageProvenanceEdge {
    pub(in crate::harness::tests) from: String,
    pub(in crate::harness::tests) to: String,
    pub(in crate::harness::tests) kind: &'static str,
    pub(in crate::harness::tests) surface: &'static str,
}

impl PricingWorkloadCertificationBundle {
    pub(in crate::harness::tests) fn lineage_provenance_edges(
        &self,
    ) -> Vec<PricingLineageProvenanceEdge> {
        vec![
            PricingLineageProvenanceEdge {
                from: self.matrix.reference.source_commit.as_str().to_owned(),
                to: self.matrix.reference.main_snapshot.as_str().to_owned(),
                kind: "commit_to_snapshot",
                surface: "reference",
            },
            PricingLineageProvenanceEdge {
                from: self.provenance.main_commit.as_str().to_owned(),
                to: self.provenance.main_snapshot.as_str().to_owned(),
                kind: "commit_to_snapshot",
                surface: "historical_provenance",
            },
            PricingLineageProvenanceEdge {
                from: self.provenance.shock_commit.as_str().to_owned(),
                to: self.provenance.shock_snapshot.as_str().to_owned(),
                kind: "commit_to_snapshot",
                surface: "historical_provenance",
            },
            PricingLineageProvenanceEdge {
                from: self.matrix.reference.main_snapshot.as_str().to_owned(),
                to: self
                    .matrix
                    .reference
                    .speculative_snapshot
                    .as_str()
                    .to_owned(),
                kind: "fork_basis_to_speculative_snapshot",
                surface: "branch_comparison",
            },
            PricingLineageProvenanceEdge {
                from: self.matrix.replay.source_commit.as_str().to_owned(),
                to: self.matrix.replay.route_identity.as_str().to_owned(),
                kind: "commit_to_route",
                surface: "replay",
            },
            PricingLineageProvenanceEdge {
                from: self.matrix.replay.route_identity.as_str().to_owned(),
                to: self.matrix.replay.invalidation_identity.as_str().to_owned(),
                kind: "route_to_invalidation",
                surface: "replay",
            },
            PricingLineageProvenanceEdge {
                from: self.aspect.source_commit.as_str().to_owned(),
                to: self.aspect.aspect_registration_id.as_str().to_owned(),
                kind: "commit_to_aspect_registration",
                surface: "aspect",
            },
            PricingLineageProvenanceEdge {
                from: self.aspect.aspect_registration_id.as_str().to_owned(),
                to: self.aspect.invalidation_target.clone(),
                kind: "aspect_to_target",
                surface: "aspect",
            },
            PricingLineageProvenanceEdge {
                from: self
                    .promotion
                    .promotion_session_identity
                    .as_str()
                    .to_owned(),
                to: self.promotion.authoritative_commit_boundary_digest.clone(),
                kind: "promotion_session_to_authoritative_boundary",
                surface: "speculation",
            },
            PricingLineageProvenanceEdge {
                from: self.promotion.authoritative_commit_boundary_digest.clone(),
                to: self.promotion.authoritative_artifact_digest.clone(),
                kind: "authoritative_boundary_to_artifact",
                surface: "promotion",
            },
            PricingLineageProvenanceEdge {
                from: self.fanout.second_source_commit.as_str().to_owned(),
                to: self.fanout.second_snapshot.as_str().to_owned(),
                kind: "commit_to_snapshot",
                surface: "fanout",
            },
            PricingLineageProvenanceEdge {
                from: self.restart_replay.source_commit.as_str().to_owned(),
                to: self.restart_replay.route_identity.as_str().to_owned(),
                kind: "commit_to_route",
                surface: "restart_replay",
            },
            PricingLineageProvenanceEdge {
                from: format!("{:?}", self.writeback.family_kind),
                to: self.writeback.commit_replay_semantic_digest.clone(),
                kind: "writeback_family_to_commit_replay_digest",
                surface: "writeback",
            },
            PricingLineageProvenanceEdge {
                from: self.hostile_failure.source_commit.as_str().to_owned(),
                to: self.hostile_failure.source_snapshot.as_str().to_owned(),
                kind: "hostile_commit_to_snapshot",
                surface: "hostile",
            },
            PricingLineageProvenanceEdge {
                from: self.digest(),
                to: self.suite_25_digest_evidence().causality_digest,
                kind: "bundle_to_causality_digest",
                surface: "causality",
            },
        ]
    }
}
