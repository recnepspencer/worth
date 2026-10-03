use super::RecordPublicationDirector;
use crate::physical_runtime::record_serving::{
    planning::{
        batch_placement::{append_operation_allocation_bytes, classify_batch},
        placement_context::PlacementPlanningContext,
        prepared_payload::prepare_payload_plan,
    },
    publication::{
        materialize_durable_data, PhysicalMutationAdmissionDisposition, PreparedPhysicalMutation,
    },
    RecordAppendDenial, RecordAppendError,
};
use worth_store_physical_format::BlobRecordKind;

impl RecordPublicationDirector {
    pub(super) fn plan_prepared_data_for_wal(
        &self,
        mut prepared: PreparedPhysicalMutation,
    ) -> Result<PreparedPhysicalMutation, (PreparedPhysicalMutation, RecordAppendDenial)> {
        let identity = prepared.mutation_identity();
        if identity.store_identity() != self.durability.store_identity()
            || identity.runtime_identity() != self.durability.runtime_identity()
            || prepared.signal_profile() != self.signal_profile
        {
            return Ok(prepared);
        }
        if prepared.disposition() == PhysicalMutationAdmissionDisposition::DuplicateUnresolved {
            return Ok(prepared);
        }
        if prepared.data_is_planned() {
            if let Some(source) = prepared.extent_copy_source() {
                let (current, _) = self.root_owner.snapshot();
                if self.current_extent_source(&current, source.record()).ok() != Some(source)
                    || prepared
                        .refresh_planned_extent_copy(current.clone())
                        .is_err()
                {
                    return Err((prepared, RecordAppendDenial::RewriteSpanNotLive));
                }
                self.root_owner.note_displaced(
                    prepared.mutation_identity(),
                    current.generation(),
                    crate::physical_runtime::durability::RetiredArtifact::Extent {
                        extent: source.extent().get(),
                        generation: source.extent_generation(),
                        range: source.arena_range(),
                    },
                    source.arena_range().length(),
                );
            }
            return Ok(prepared);
        }
        let planned = if let Some(copy) = prepared.take_completed_extent_copy() {
            match self.build_extent_copy_adoption(&mut prepared, copy) {
                Ok(plans) => Ok(plans),
                Err((copy, error)) => {
                    prepared = prepared.attach_completed_extent_copy(copy);
                    Err(error)
                }
            }
        } else if let Some(source) = prepared.extent_rewrite_source() {
            self.build_extent_record_rewrite(&prepared, source)
        } else if prepared.selected_segment_rewrite() {
            self.build_selected_segment_rewrite(&prepared)
        } else {
            self.build_durable_data_plan(&mut prepared)
        };
        match planned {
            Ok((data, root)) => Ok(prepared.attach_plans(data, root)),
            Err(error) => {
                let denial = data_planning_denial(error);
                if let Some(runtime) = self.runtime.upgrade() {
                    runtime.health.observe_append_denial(&denial);
                }
                Err((prepared, denial))
            }
        }
    }

    fn build_durable_data_plan(
        &self,
        prepared: &mut PreparedPhysicalMutation,
    ) -> Result<
        (
            crate::physical_runtime::durability::PreparedPhysicalDataPlan,
            crate::physical_runtime::record_serving::PreparedPhysicalRootProjection,
        ),
        RecordAppendError,
    > {
        let runtime = self.runtime.upgrade().ok_or(RecordAppendError::Denied(
            RecordAppendDenial::PublicationAuthorityReleased,
        ))?;
        let released_drop_basis = prepared.released_drop_basis();
        let release_head_basis = released_drop_basis.map(|basis| basis.head());
        if (prepared.blob_record_kind() == Some(BlobRecordKind::ReclaimDescriptorV3))
            != release_head_basis.is_some()
            || prepared.released_directory_record()
                != released_drop_basis.is_some_and(|basis| basis.directory().is_some())
        {
            return Err(RecordAppendError::Denied(
                RecordAppendDenial::ReclaimFenceUnavailable,
            ));
        }
        if matches!(
            prepared.blob_record_kind(),
            Some(BlobRecordKind::DropSetManifestV3 | BlobRecordKind::ReclaimDescriptorV3)
        ) && prepared.released_control_placement().is_none()
        {
            return Err(RecordAppendError::Denied(
                RecordAppendDenial::ReclaimFenceUnavailable,
            ));
        }
        let batch = prepared.duplicate_prepared_batch();
        let mut bytes = append_operation_allocation_bytes(
            self.format,
            prepared.placement(),
            &batch,
            prepared.blob_record_kind(),
            prepared.inline_only(),
        );
        if release_head_basis.is_some() {
            bytes = bytes
                .checked_add(
                    super::release_head_preparation::release_head_reservation_bytes(
                        self.format.declaration(),
                    )
                    .ok_or(RecordAppendError::Denied(
                        RecordAppendDenial::PhysicalPressure,
                    ))?,
                )
                .ok_or(RecordAppendError::Denied(
                    RecordAppendDenial::PhysicalPressure,
                ))?;
        }
        let allocation = self
            .residency
            .begin_foreground_write_operation(
                std::num::NonZeroU64::new(bytes)
                    .expect("an admitted nonempty append has nonzero planning bytes"),
            )
            .map_err(|denial| {
                RecordAppendError::Denied(RecordAppendDenial::from_residency(denial))
            })?;
        let (current_root, current_free_space) = self.root_owner.snapshot();
        if prepared.blob_record_kind() == Some(BlobRecordKind::ChunkReuseClaimV2) {
            let declaration =
                prepared
                    .reuse_declaration_basis()
                    .ok_or(RecordAppendError::Denied(
                        RecordAppendDenial::ReuseDestinationInvalid,
                    ))?;
            self.verify_new_reuse_claim(&batch, current_root.generation(), declaration)?;
        }
        if prepared.blob_record_kind() == Some(BlobRecordKind::DedupeQuarantine) {
            let declaration =
                prepared
                    .reuse_declaration_basis()
                    .ok_or(RecordAppendError::Denied(
                        RecordAppendDenial::ReuseDestinationInvalid,
                    ))?;
            self.verify_new_quarantine_claim(&batch, current_root.generation(), declaration)?;
        }
        let admitted = batch
            .admit(self.access)
            .map_err(RecordAppendError::Denied)?;
        let reader =
            crate::physical_runtime::record_serving::access::manifest_routing::ManifestReader::
                serving(
                    self.residency.clone(),
                    self.format,
                    self.access,
                    current_root.clone(),
                );
        let classified = classify_batch(
            &reader,
            &allocation,
            prepared.placement(),
            prepared.blob_record_kind(),
            prepared.selected_content_class(),
            prepared.released_directory_record(),
            prepared.inline_only(),
            admitted,
        )?;
        let shape = classified.identity_reservation_shape()?;
        let mut preparation = self
            .preparation
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut candidate_frontier = preparation
            .allocation_frontier
            .reserve(shape.segments(), shape.pages(), shape.extents())
            .ok_or(RecordAppendError::Denied(
                RecordAppendDenial::PhysicalIdentityExhausted,
            ))?;
        drop(preparation);
        let arena_owner =
            self.arena_allocation_owner(&allocation, prepared.placement(), &current_free_space)?;
        let mut payload = prepare_payload_plan(
            PlacementPlanningContext {
                arena_owner,
                released_control_placement: prepared.released_control_placement(),
                allocation: &allocation,
                media: runtime.executor.record_serving_media(),
                format: self.format,
                access: self.access,
                current_root: &current_root,
                current_free_space: &current_free_space,
                frontier: &mut candidate_frontier,
                placement: prepared.placement(),
                residency: self.residency.clone(),
            },
            classified,
            true,
        )?;
        prepared
            .materialization_observation()
            .apply_to(&mut payload.observation);
        let (data, mut root) = materialize_durable_data(
            payload,
            self.format,
            std::num::NonZeroU64::new(bytes)
                .expect("an admitted nonempty append has nonzero planning bytes"),
            prepared.manifest_capacity_transition(),
        )?;
        if matches!(
            prepared.blob_record_kind(),
            Some(
                BlobRecordKind::ChunkReuseClaim
                    | BlobRecordKind::ChunkReuseClaimV2
                    | BlobRecordKind::DedupeQuarantine
            )
        ) {
            root.blob_reuse_source_fence = true;
        }
        if matches!(
            prepared.blob_record_kind(),
            Some(
                BlobRecordKind::ReclaimDescriptor
                    | BlobRecordKind::ReclaimDescriptorV2
                    | BlobRecordKind::ReclaimDescriptorV3
            )
        ) {
            let drops = self
                .root_owner
                .reclaim_drops_for(prepared.mutation_identity())
                .ok_or(RecordAppendError::Denied(
                    RecordAppendDenial::ReclaimFenceUnavailable,
                ))?;
            root.drop_records = drops.into_iter().collect();
            if !self
                .root_owner
                .note_reclaim_displaced_batch(prepared.mutation_identity())
            {
                return Err(RecordAppendError::Denied(
                    RecordAppendDenial::ReclaimFenceUnavailable,
                ));
            }
        }
        if let Some(basis) = release_head_basis {
            self.plan_released_head_for_wal(&allocation, &current_root, basis, &mut root)?;
        }
        if let Some(rebinding) = released_drop_basis.and_then(|basis| basis.directory()) {
            if current_root.derived_family_directory() != Some(rebinding.expected_previous()) {
                return Err(RecordAppendError::Denied(
                    RecordAppendDenial::DerivedDirectorySourceChanged,
                ));
            }
            let mut records = root.recovery_record_identities();
            let Some(_) = records.next() else {
                return Err(RecordAppendError::Denied(
                    RecordAppendDenial::ReclaimFenceUnavailable,
                ));
            };
            let Some(directory_record) = records.next() else {
                return Err(RecordAppendError::Denied(
                    RecordAppendDenial::ReclaimFenceUnavailable,
                ));
            };
            let exact_roster = records.next().is_none();
            drop(records);
            if !exact_roster
                || root
                    .set_released_directory_rebinding(directory_record, rebinding)
                    .is_none()
            {
                return Err(RecordAppendError::Denied(
                    RecordAppendDenial::ReclaimFenceUnavailable,
                ));
            }
        }
        for (artifact, growth_bytes) in data.retained_growth() {
            self.root_owner
                .hold_rewrite_candidate(prepared.mutation_identity(), artifact, growth_bytes)
                .map_err(|()| RecordAppendError::Denied(RecordAppendDenial::RetentionPressure))?;
        }
        if let Some(claims) = prepared
            .released_control_placement()
            .map(|placement| placement.claim_count())
        {
            let requested = root
                .arena_reservations
                .len()
                .checked_add(claims)
                .and_then(|count| {
                    count.checked_mul(std::mem::size_of::<
                        crate::physical_runtime::record_serving::arena::ArenaReservation,
                    >())
                })
                .and_then(|bytes| u64::try_from(bytes).ok())
                .ok_or(RecordAppendError::Denied(
                    RecordAppendDenial::PhysicalPressure,
                ))?;
            root.arena_reservations
                .try_reserve_exact(claims)
                .map_err(|cause| {
                    RecordAppendError::Denied(RecordAppendDenial::PlanningAllocationUnavailable {
                        requested,
                        cause,
                    })
                })?;
            let (control, directory) = prepared
                .take_released_control_placement()
                .expect("a checked prepared release claim remains owned")
                .into_reservations();
            root.arena_reservations.push(control);
            root.arena_reservations.extend(directory);
        }
        Ok((data, root))
    }
}

pub(super) fn data_planning_denial(error: RecordAppendError) -> RecordAppendDenial {
    match error {
        RecordAppendError::Denied(denial) => denial,
        RecordAppendError::PhysicalPressure { .. } => RecordAppendDenial::PhysicalPressure,
        RecordAppendError::StreamFailed(_) => RecordAppendDenial::PublishedLayoutDamaged,
    }
}
