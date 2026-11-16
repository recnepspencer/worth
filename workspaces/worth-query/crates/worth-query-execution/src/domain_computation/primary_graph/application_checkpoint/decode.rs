//! Current checkpoint metadata reconstruction under the bootstrap payer.
use super::*;

impl WorthQueryApplicationCheckpoint {
    pub(super) fn decode_current_body(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<DecodedApplicationCheckpoint, CheckpointDecodeDenial> {
        let header::CurrentSections {
            bootstrap_commit_id,
            accepted_count,
            native,
            native_image: _,
            world_originals: _,
            query_header: _,
            accepted_count_bytes: _,
            outputs,
        } = header::sections(self.bytes.bytes())?;
        let native_len = native.len();
        let mut cursor = CheckpointCursor::new(outputs);
        let minimum = MINIMUM_ACCEPTED_OUTPUT_BYTES;
        if accepted_count > cursor.remaining.len() / minimum {
            return Err("checkpoint accepted-output count exceeds its payload"
                .to_owned()
                .into());
        }
        let mut metadata = CheckpointMetadataFunding::new(execution)?;
        let mut accepted_outputs = metadata.vec::<super::super::application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity>(accepted_count, execution)?;
        for _ in 0..accepted_count {
            accepted_outputs.push(super::accepted_output::read(&mut cursor, execution)?);
        }
        if accepted_outputs
            .windows(2)
            .any(|pair| pair[0].canonical_cmp(&pair[1]) != std::cmp::Ordering::Less)
        {
            return Err(
                "checkpoint accepted outputs are duplicated or non-canonical"
                    .to_owned()
                    .into(),
            );
        }
        // A funded temporary permutation checks duplicate slots without copying
        // producer strings into another authoritative inventory.
        let mut scratch = CheckpointMetadataFunding::new(execution)?;
        let mut slots = scratch.vec::<usize>(accepted_count, execution)?;
        slots.extend(0..accepted_count);
        slots.sort_unstable_by(|left, right| {
            (
                &accepted_outputs[*left].producer,
                accepted_outputs[*left].scope,
                accepted_outputs[*left].source_partition,
            )
                .cmp(&(
                    &accepted_outputs[*right].producer,
                    accepted_outputs[*right].scope,
                    accepted_outputs[*right].source_partition,
                ))
        });
        if slots.windows(2).any(|pair| {
            accepted_outputs[pair[0]].output_slot() == accepted_outputs[pair[1]].output_slot()
        }) {
            return Err("checkpoint output slots are duplicated".to_owned().into());
        }
        drop(slots);
        drop(scratch);
        if !cursor.is_empty() {
            return Err("Query application checkpoint payload length differs"
                .to_owned()
                .into());
        }
        let native_end = HEADER_BYTES
            .checked_add(native_len)
            .ok_or_else(|| "checkpoint native payload length overflows".to_owned())?;
        let native = worth_relational::facade::durability::RelationalNativeCheckpoint::from_untrusted_bytes_region(
            self.bytes,
            HEADER_BYTES..native_end,
        )
        .map_err(str::to_owned)?;
        Ok(DecodedApplicationCheckpoint {
            origin: self.origin,
            native,
            bootstrap_commit_id,
            accepted_outputs,
            metadata,
        })
    }
}

mod court_doors_for_test;
