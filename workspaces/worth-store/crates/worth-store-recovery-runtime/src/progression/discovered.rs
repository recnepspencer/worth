use crate::entry::{
    PhysicalRecoveryBlockEvidence, PhysicalRecoveryBlockKind, PhysicalRecoveryOutcome,
};
use crate::orchestration::DiscoveryMaterial;

mod selection;

use selection::{select_sources, SelectionInput};

pub struct DiscoveredPhysicalRecovery {
    material: DiscoveryMaterial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PhysicalRecoveryDiscoveryCounters {
    pub selector_slots: u64,
    pub current_selector_integrity_admissions: u64,
    pub previous_selector_integrity_admissions: u64,
    pub current_selector_interpretations: u64,
    pub previous_selector_interpretations: u64,
    pub current_root_integrity_admissions: u64,
    pub previous_root_integrity_admissions: u64,
    pub current_root_candidate_interpretations: u64,
    pub previous_root_candidate_interpretations: u64,
    pub bootstrap_integrity_attempts: u64,
    pub bootstrap_integrity_admissions: u64,
    pub bootstrap_integrity_rejections: u64,
    pub bootstrap_absent: u64,
    pub bootstrap_owner_projections: u64,
    pub bootstrap_owner_decoder_entries: u64,
    pub root_candidates: u64,
    pub current_root_admitted: u64,
    pub current_root_rejected: u64,
    pub current_root_absent: u64,
    pub previous_root_admitted: u64,
    pub previous_root_rejected: u64,
    pub previous_root_absent: u64,
    pub checkpoint_candidates: u64,
    pub checkpoints_admitted: u64,
    pub checkpoints_rejected: u64,
    pub checkpoints_absent: u64,
    pub checkpoint_integrity_attempts: u64,
    pub checkpoint_integrity_admissions: u64,
    pub checkpoint_integrity_rejections: u64,
    pub checkpoint_owner_projections: u64,
    pub checkpoint_owner_decoder_entries: u64,
    pub wal_entries: u64,
    pub wal_integrity_attempts: u64,
    pub wal_integrity_admissions: u64,
    pub wal_integrity_rejections: u64,
    pub wal_owner_projections: u64,
    pub wal_owner_decoder_entries: u64,
    pub wal_segments: u64,
    pub wal_segments_scanned: u64,
    pub valid_wal_segments: u64,
    pub wal_frames: u64,
    pub wal_bytes: u64,
    pub valid_wal_frames: u64,
    pub valid_wal_bytes: u64,
    pub torn_suffix_frames: u64,
    pub torn_suffix_bytes: u64,
    pub wal_corruption_denials: u64,
    pub wal_missing_range_denials: u64,
    pub bytes_observed: u64,
    pub manifest_bytes: u64,
    pub manifest_blocks: u64,
    pub manifest_entries: u64,
    pub selected_page_facts: u64,
    pub distinct_pages_and_extents: u64,
    pub residue: u64,
    pub noncanonical_wal_residue: u64,
    pub nonregular_wal_residue: u64,
    pub trailing_empty_wal_residue: u64,
    pub interrupted_wal_start_residue: u64,
    pub unreferenced_compaction_residue: u64,
}

impl DiscoveredPhysicalRecovery {
    pub(crate) const fn from_material(material: DiscoveryMaterial) -> Self {
        Self { material }
    }

    pub const fn counters(&self) -> PhysicalRecoveryDiscoveryCounters {
        self.material.counters
    }

    pub fn select(self) -> Result<super::SelectedPhysicalRecovery, PhysicalRecoveryOutcome> {
        let DiscoveryMaterial {
            authority,
            mut coordination,
            current,
            previous,
            bootstrap,
            current_manifest_facts,
            previous_manifest_facts,
            checkpoint,
            wal,
            residue,
            root_protocol_denials,
            counters,
            integrity_trace,
        } = self.material;
        let input = SelectionInput {
            current,
            previous,
            bootstrap,
            current_manifest_facts,
            previous_manifest_facts,
            checkpoint,
            wal,
            residue,
            root_protocol_denials,
            counters,
            integrity_trace,
        };
        match select_sources(input, authority.limits) {
            Ok(selected) => {
                if let Some(basis) = selected.checkpoint_binding_basis {
                    assert!(coordination.install_checkpoint_binding_basis(basis));
                }
                Ok(super::SelectedPhysicalRecovery::new(
                    authority,
                    coordination,
                    selected.selection,
                    super::RecoveryIntegrityEvidence::new(
                        selected.admitted_wal,
                        selected.wal_integrity_observations,
                    ),
                    selected.counters,
                    selected.root_protocol_denials,
                    selected.integrity_trace,
                ))
            }
            Err(failure) => blocked(authority, coordination, failure.kind, failure.evidence),
        }
    }
}

fn blocked(
    authority: crate::entry::AdmittedPlatformAuthority,
    coordination: crate::orchestration::RecoveryCoordination,
    kind: PhysicalRecoveryBlockKind,
    evidence: PhysicalRecoveryBlockEvidence,
) -> Result<super::SelectedPhysicalRecovery, PhysicalRecoveryOutcome> {
    Err(crate::handoff::block_unsupported_scope(
        authority,
        coordination,
        kind,
        evidence,
    ))
}
