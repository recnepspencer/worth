use super::*;
use sha2::{Digest, Sha256};

impl PhysicalWalRuntimeOwner {
    pub(in crate::physical_runtime) fn plan_maintenance_frame(
        &self,
        payload: &[u8],
    ) -> Result<(ArtifactTreeFile, u64, u64, u64, Vec<u8>), ()> {
        let mut state = self
            .shared
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if state.sealed || state.in_flight {
            return Err(());
        }
        let extent_copy = if payload.starts_with(worth_store_physical_format::EXTENT_COPY_DOMAIN) {
            Some(
                worth_store_physical_format::PhysicalExtentCopyRecord::decode(
                    payload,
                    state.record_format,
                )
                .map_err(|_| ())?,
            )
        } else {
            None
        };
        if let Some(record) = extent_copy {
            let mut projected = state.copy_obligations.clone();
            let start = state.frontier.last_lsn_end().map_or(1, |lsn| lsn.get());
            super::super::copy_obligation::observe_copy_record(
                &mut projected,
                record,
                state.frontier.segment().get(),
                state.frontier.generation().get(),
                start,
                start.checked_add(1).ok_or(())?,
                0,
            )?;
        }
        let start = state
            .frontier
            .last_lsn_end()
            .unwrap_or(LogSequenceNumber::new(LogSequenceNumber::GENESIS.get() + 1));
        let end = LogSequenceNumber::new(start.get().checked_add(1).ok_or(())?);
        let range = worth_store_wal::WalLsnRange::new(start, end).map_err(|_| ())?;
        let planned = worth_store_wal::plan_wal_frame_append(
            state.frontier,
            range,
            if extent_copy.is_some() {
                "store.physical.extent-copy.v1"
            } else {
                "store.physical.retirement.v2"
            },
            payload,
        )
        .map_err(|_| ())?;
        let byte_limit = state.policy.segment_byte_limit().get().get();
        if planned.resulting_frontier().valid_prefix_bytes() > byte_limit {
            return Err(());
        }
        let bytes = planned.frame().encoded_frame().to_vec();
        let offset = state.frontier.valid_prefix_bytes();
        let segment = state.frontier.segment().get();
        let generation = state.frontier.generation().get();
        state.in_flight = true;
        let retirement = super::super::super::retention::decode_retirement(payload)
            .map(|record| (record.artifact, record.completion));
        state.maintenance = Some(PlannedMaintenanceFrame {
            payload_digest: Sha256::digest(payload).into(),
            bytes: bytes.clone(),
            frontier: planned.resulting_frontier(),
            segment: state.frontier.segment(),
            generation: state.frontier.generation(),
            lsn_range: range,
            retirement,
            extent_copy,
        });
        Ok((
            state.active_artifact.clone(),
            segment,
            generation,
            offset,
            bytes,
        ))
    }

    pub(in crate::physical_runtime) fn planned_maintenance_interval(
        &self,
    ) -> Option<(u64, u64, u64, u64, u64, u64)> {
        let state = self
            .shared
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let planned = state.maintenance.as_ref()?;
        Some((
            planned.segment.get(),
            planned.generation.get(),
            planned.lsn_range.start().get(),
            planned.lsn_range.end_exclusive().get(),
            state.frontier.valid_prefix_bytes(),
            planned.bytes.len() as u64,
        ))
    }

    pub(in crate::physical_runtime) fn maintenance_payload_matches(
        &self,
        payload_digest: [u8; 32],
    ) -> bool {
        self.shared
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .maintenance
            .as_ref()
            .is_some_and(|planned| planned.payload_digest == payload_digest)
    }

    pub(in crate::physical_runtime) fn note_maintenance_written(&self) {
        self.shared
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .awaiting_barrier = true;
    }

    pub(in crate::physical_runtime) fn maintenance_awaiting_barrier(&self) -> bool {
        self.shared
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .awaiting_barrier
    }

    pub(in crate::physical_runtime) fn planned_maintenance_artifact(
        &self,
    ) -> Option<ArtifactTreeFile> {
        let state = self
            .shared
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state
            .maintenance
            .as_ref()
            .map(|_| state.active_artifact.clone())
    }

    pub(in crate::physical_runtime) fn abort_maintenance_frame(&self) {
        let mut state = self
            .shared
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.maintenance = None;
        state.in_flight = false;
        state.awaiting_barrier = false;
    }

    pub(in crate::physical_runtime) fn finish_maintenance_frame(&self) -> Result<(), ()> {
        let mut state = self
            .shared
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.awaiting_barrier = false;
        let planned = state.maintenance.take().ok_or(())?;
        let identity =
            worth_store_wal::WalSegmentArtifactIdentity::new(planned.segment, planned.generation);
        if state
            .segments
            .record_completed_append(identity, planned.lsn_range, planned.bytes.len() as u64)
            .is_err()
        {
            state.sealed = true;
            state.in_flight = false;
            return Err(());
        }
        state.frontier = planned.frontier;
        state.appended_frames = state.appended_frames.saturating_add(1);
        state.appended_bytes = state
            .appended_bytes
            .saturating_add(planned.bytes.len() as u64);
        state.in_flight = false;
        let start = planned.lsn_range.start().get();
        let end = planned.lsn_range.end_exclusive().get();
        super::super::super::retention::note_retirement_hold(
            &mut state.unresolved_retirement_spans,
            planned.retirement,
            start,
            end,
        );
        if let Some(record) = planned.extent_copy {
            if super::super::copy_obligation::observe_copy_record(
                &mut state.copy_obligations,
                record,
                planned.segment.get(),
                planned.generation.get(),
                start,
                end,
                0,
            )
            .is_err()
            {
                state.sealed = true;
                return Err(());
            }
        }
        if state.record_durable_barrier(start, end) {
            Ok(())
        } else {
            Err(())
        }
    }
}
