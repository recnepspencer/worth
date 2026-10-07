//! The record-less root projection: a member that moves the root without
//! appending, dropping, placing or rebinding any record. The constructor and
//! the predicate over the merged plan live together, so the shape has one
//! spelling.

use std::collections::BTreeMap;
use std::num::NonZeroU64;

use worth_store_physical_format::DurablePhysicalRootManifest;

use super::{DerivedRootUpdates, PreparedPhysicalRootProjection};
use crate::physical_runtime::record_serving::{
    planning::prepared_payload::PreparedRecordPayloadPlan,
    publication::append_observation::PublicationObservation, AdmittedRecordPlacementPolicy,
};
use crate::physical_runtime::PhysicalManifestCapacityTransition;

impl PreparedPhysicalRootProjection {
    /// `root_publication_allocation_bytes` is what the successor root itself
    /// allocates. The successor inherits the source root's maintenance
    /// protocol requirement, so the member states none of its own.
    pub(in crate::physical_runtime::record_serving) fn record_less(
        source_root: DurablePhysicalRootManifest,
        placement: AdmittedRecordPlacementPolicy,
        root_publication_allocation_bytes: NonZeroU64,
    ) -> Self {
        Self {
            derived_updates: DerivedRootUpdates::default(),
            release_head_effect: None,
            arena_reservations: Vec::new(),
            root_publication_allocation_bytes,
            source_root,
            blob_reuse_source_fence: false,
            manifest_capacity_transition: PhysicalManifestCapacityTransition::PreserveCurrent,
            placement,
            records: Vec::new(),
            inserted_records: 0,
            drop_records: Default::default(),
            payload_manifests: Vec::new(),
            placements: BTreeMap::new(),
            retired_inline_witnesses: BTreeMap::new(),
            segment_updates: BTreeMap::new(),
            inline_allocations: Vec::new(),
            last_inline_record: None,
            last_inline_segment: None,
            requires_maintenance_protocol: false,
            observation: PublicationObservation::default(),
        }
    }
}

impl DerivedRootUpdates {
    fn is_none(&self) -> bool {
        self.latest_blob_publication.is_none()
            && self.latest_blob_quarantine.is_none()
            && self.directory.is_none()
            && self.expected_previous_directory.is_none()
            && self.indexed_through_quarantine.is_none()
            && self.released_directory_rebinding.is_none()
    }
}

impl PreparedRecordPayloadPlan {
    /// Whether the plan still has exactly the record effect `record_less`
    /// gave it: none. A settled group that merged another member in does not.
    pub(in crate::physical_runtime::record_serving) fn is_record_less(&self) -> bool {
        self.derived_updates.is_none()
            && self.arena_reservations.is_empty()
            && !self.blob_reuse_source_fence
            && self.records.is_empty()
            && self.drop_records.is_empty()
            && self.data.is_empty()
            && self.payload_manifests.is_empty()
            && self.placements.is_empty()
            && self.segment_updates.is_empty()
            && self.inline_allocations.is_empty()
            && self.last_inline_record.is_none()
            && self.last_inline_segment.is_none()
            && !self.requires_maintenance_protocol
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU64;

    use worth_store_physical_format::{
        DurablePhysicalRootManifest, PersistedRecordIdentity, PhysicalRecordFormatDeclaration,
    };

    use super::PreparedPhysicalRootProjection;
    use crate::physical_runtime::record_serving::planning::prepared_payload::PreparedRecordPayloadPlan;
    use crate::physical_runtime::{AdmittedPhysicalRecordFormat, PhysicalRecordPlacementPolicy};

    fn plan() -> PreparedRecordPayloadPlan {
        let format = AdmittedPhysicalRecordFormat::admit(
            PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
        );
        PreparedPhysicalRootProjection::record_less(
            DurablePhysicalRootManifest::builder(3, 1, 2, 1)
                .admit()
                .unwrap(),
            PhysicalRecordPlacementPolicy::builder()
                .admit(format)
                .unwrap(),
            NonZeroU64::new(4096).unwrap(),
        )
        .into_payload_plan()
    }

    #[test]
    fn the_constructed_projection_is_what_the_predicate_admits() {
        assert!(plan().is_record_less());
    }

    #[test]
    fn any_record_effect_merged_into_the_plan_is_not_record_less() {
        let record = PersistedRecordIdentity::new([1; 16], 1).unwrap();
        let effects: [fn(&mut PreparedRecordPayloadPlan, PersistedRecordIdentity); 6] = [
            |plan, record| plan.records.push(record),
            |plan, record| plan.drop_records.extend([record]),
            |plan, record| plan.last_inline_record = Some(record),
            |plan, record| plan.derived_updates.latest_blob_quarantine = Some(record),
            |plan, _| plan.derived_updates.expected_previous_directory = Some(None),
            |plan, _| plan.requires_maintenance_protocol = true,
        ];
        for effect in effects {
            let mut plan = plan();
            effect(&mut plan, record);
            assert!(!plan.is_record_less());
        }
    }
}
