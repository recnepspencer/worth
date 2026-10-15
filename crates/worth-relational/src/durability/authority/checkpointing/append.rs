use super::*;

impl DurabilityAuthority<'_> {
    pub(crate) fn append_commit(
        &self,
        authority: crate::durability::authority::DurableAppendAuthority,
        positioned: &crate::history::data::PositionedCanonicalCommit,
        bytes: Option<&worth_execution::ExecutionImmutableBytes>,
    ) -> Result<(), DurabilityError> {
        let envelope = positioned.envelope();
        authority.validate(self.runtime.runtime_instance_id(), envelope)?;
        if self.runtime.durability.take_armed_append_failure_for_test() {
            return Err(DurabilityError::new(
                crate::durability::data::RecoveryFailureClass::DurableIoFailure,
                "test-injected durable append failure",
            ));
        }
        match self.runtime.runtime_config().durability.policy.mode {
            DurabilityMode::InMemoryCanonical => {
                self.runtime
                    .durability
                    .push_log_envelope(positioned.clone());
                Ok(())
            }
            DurabilityMode::PersistedSegmentedLocalFs => {
                let _store_io = crate::durability::log::local_store::lock_store(self.runtime)?;
                let mut store = ensure_loaded_store(self.runtime)?;
                let segment_capacity = store.layout.segment_commit_capacity.max(1);
                let active_segment = store.segments.last().cloned();
                // Repair may follow a completed segment rename but failed manifest
                // publication. Find the exact existing frame before selecting a new slot.
                let mut retry_segment = None;
                for segment in &store.segments {
                    if crate::durability::log::native_file_codec::read_segment_inventory(
                        &segment.path,
                    )?
                    .contains(&envelope.commit.commit_id)
                    {
                        retry_segment = Some(segment.segment_id);
                        break;
                    }
                }
                let segment_id = match retry_segment {
                    Some(segment) => segment,
                    None => match active_segment.as_ref() {
                        Some(segment) if segment.commit_count < segment_capacity => {
                            segment.segment_id
                        }
                        _ => DurableSegmentId(
                            store
                                .segments
                                .last()
                                .map(|segment| segment.segment_id.0)
                                .unwrap_or(0)
                                + 1,
                        ),
                    },
                };
                let segment_path = segment_file_path(&store.layout, segment_id);
                let existing_manifest = store
                    .segments
                    .iter()
                    .find(|segment| segment.segment_id == segment_id)
                    .cloned();
                let first_commit_id = existing_manifest
                    .as_ref()
                    .and_then(|segment| segment.first_commit_id)
                    .unwrap_or(envelope.commit.commit_id);
                let last_commit_id = if retry_segment.is_some() {
                    existing_manifest
                        .as_ref()
                        .and_then(|segment| segment.last_commit_id)
                        .unwrap_or(envelope.commit.commit_id)
                } else {
                    envelope.commit.commit_id
                };
                let commit_count = existing_manifest
                    .as_ref()
                    .map(|segment| {
                        segment.commit_count + if retry_segment.is_none() { 1 } else { 0 }
                    })
                    .unwrap_or(1);
                let bytes = bytes.ok_or_else(|| {
                    DurabilityError::new(
                        crate::durability::data::RecoveryFailureClass::DurableIoFailure,
                        "persisted append requires its original-policy prepared bytes",
                    )
                })?;
                crate::durability::log::native_file_codec::append_segment_bytes(
                    &segment_path,
                    positioned,
                    bytes,
                )?;
                if let Some(existing) = store
                    .segments
                    .iter_mut()
                    .find(|segment| segment.segment_id == segment_id)
                {
                    existing.first_commit_id = Some(first_commit_id);
                    existing.last_commit_id = Some(last_commit_id);
                    existing.commit_count = commit_count;
                    existing.integrity = DurableIntegrityStatus::Verified;
                } else {
                    store.segments.push(DurableSegmentManifest {
                        segment_id,
                        path: segment_path,
                        first_commit_id: Some(first_commit_id),
                        last_commit_id: Some(last_commit_id),
                        commit_count,
                        runtime_name: self.runtime.runtime_name().to_string(),
                        profile: self.runtime.runtime_profile(),
                        schema_version: self.runtime.primary_schema_version_id(),
                        integrity: DurableIntegrityStatus::Verified,
                    });
                }
                persist_store_manifest(&store)?;
                self.runtime.durability.set_store(Some(store));
                self.runtime
                    .durability
                    .push_log_envelope(positioned.clone());
                let latest_commit_id = self
                    .runtime
                    .durability
                    .log()
                    .last()
                    .map(|entry| entry.envelope().commit.commit_id)
                    .unwrap_or(envelope.commit.commit_id);
                self.runtime
                    .publication_authority()
                    .push_bounded_diagnostic(
                        DiagnosticsScope::History,
                        DiagnosticsArtifactKind::MinimalSummary,
                        vec![durable_segment_append_succeeded(
                            segment_id,
                            latest_commit_id,
                        )],
                    );
                Ok(())
            }
        }
    }
}
