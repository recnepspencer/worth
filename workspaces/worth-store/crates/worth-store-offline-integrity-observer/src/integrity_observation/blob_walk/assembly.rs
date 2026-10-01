use super::*;

impl BlobRecordWalk {
    pub(crate) fn observe_valid_extent_chunk(
        &mut self,
        expected: &ChildExpectation,
        frame: &[u8],
        store: Option<[u8; 16]>,
        counters: &mut OfflineIntegrityObservationCounters,
    ) {
        let ChildScope::ExtentChunk {
            arena,
            extent,
            record,
            logical_bytes,
            logical_offset,
            ..
        } = expected.scope
        else {
            return;
        };
        let content = &frame[112..]; // C.5 extent frame already independently validated.
        if logical_offset == 0 {
            if let Some(pending) = self.pending.take() {
                self.selected.push(projection::incomplete(pending, None));
            }
            if !blob_record::is_blob_prefix(content) {
                return;
            }
            if logical_bytes > (1 << 20) {
                self.selected.push(Selected {
                    record,
                    path: expected.path.clone(),
                    generation: expected.generation,
                    family: projection::family_from_prefix(content),
                    fact: None,
                    outcome: damage(Cause::Framing),
                    route: None,
                });
                return;
            }
            self.pending = Some(Pending {
                record,
                logical_bytes,
                path: expected.path.clone(),
                generation: expected.generation,
                bytes: Vec::with_capacity(logical_bytes as usize),
                route: ExtentRoute {
                    format: expected.format,
                    arena,
                    extent,
                    logical_bytes,
                    frames: Vec::new(),
                },
                interruption: None,
            });
        }
        let Some(pending) = self.pending.as_mut() else {
            return;
        };
        if pending.record != record
            || pending.bytes.len() as u64 != logical_offset
            || pending.bytes.len().saturating_add(content.len()) as u64 > pending.logical_bytes
        {
            let pending = self.pending.take().expect("present pending");
            self.selected.push(projection::incomplete(pending, None));
            return;
        }
        pending.bytes.extend_from_slice(content);
        pending.route.frames.push((
            expected.offset,
            expected.length.expect("routed chunk length"),
        ));
        if pending.bytes.len() as u64 == pending.logical_bytes {
            let pending = self.pending.take().expect("complete pending");
            let family = projection::family_from_prefix(&pending.bytes);
            let decoded = blob_record::decode(&pending.bytes, store, counters);
            let (fact, outcome) = match decoded {
                Ok(fact) => self.admit_fact(fact),
                Err(outcome) => (None, outcome),
            };
            let family = fact.as_ref().map_or(family, |fact| fact.family().into());
            self.selected.push(Selected {
                record: pending.record,
                path: pending.path,
                generation: pending.generation,
                family,
                fact,
                outcome,
                route: Some(pending.route),
            });
        }
    }
}
