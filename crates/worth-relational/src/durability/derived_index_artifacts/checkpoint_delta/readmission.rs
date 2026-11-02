//! One generation validation algorithm for ordinary and funded image decode.
use super::super::decode_budget::IndexDecodeBudget;
use super::*;
use crate::durability::data::RelationalRecoveryStop as Stop;
impl DerivedIndexCheckpointArtifacts {
    pub(crate) fn readmit(&self) -> Result<Vec<DerivedIndexGeneration>, DurabilityError> {
        match self.readmit_controlled(&mut IndexDecodeBudget::default()) {
            Ok(generations) => Ok(generations),
            Err(Stop::Durability(cause)) => Err(cause),
            Err(Stop::Request(_)) | Err(Stop::Allocation(_)) => {
                unreachable!("uncontrolled decode has no request or allocation issuer")
            }
        }
    }
    pub(in super::super) fn readmit_controlled(
        &self,
        budget: &mut IndexDecodeBudget<'_, '_>,
    ) -> Result<Vec<DerivedIndexGeneration>, Stop> {
        if self.format_version != Self::FORMAT_VERSION {
            return Err(corrupt("unsupported derived index checkpoint format"));
        }
        if self.generation_ids
            != self
                .generations
                .iter()
                .map(|entry| entry.generation_id)
                .collect::<Vec<_>>()
        {
            return Err(corrupt("derived index checkpoint manifest mismatch"));
        }
        let mut previous = BTreeMap::<
            DerivedIndexId,
            (
                DerivedIndexGenerationId,
                (
                    Cow<'_, [u8]>,
                    Option<worth_execution::ExecutionMemoryReservation>,
                ),
            ),
        >::new();
        let mut seen = BTreeSet::new();
        let mut generations = Vec::with_capacity(self.generations.len());
        for entry in &self.generations {
            budget.consult()?;
            if !seen.insert(entry.generation_id) {
                return Err(corrupt("duplicate derived index checkpoint generation"));
            }
            let (entries_bytes, funding) = match entry.base_generation_id {
                Some(base_id) => {
                    let (previous_id, (base, _)) = previous
                        .get(&entry.index_id)
                        .ok_or_else(|| corrupt("missing derived index checkpoint base"))?;
                    if *previous_id != base_id {
                        return Err(corrupt("stale derived index checkpoint base"));
                    }
                    let prefix = usize::try_from(entry.shared_prefix_bytes)
                        .map_err(|_| corrupt("invalid derived index checkpoint prefix"))?;
                    let suffix = usize::try_from(entry.shared_suffix_bytes)
                        .map_err(|_| corrupt("invalid derived index checkpoint suffix"))?;
                    if prefix > base.len() || suffix > base.len() - prefix {
                        return Err(corrupt("derived index checkpoint delta exceeds base"));
                    }
                    let length = prefix
                        .checked_add(entry.changed_bytes.len())
                        .and_then(|n| n.checked_add(suffix))
                        .ok_or_else(super::super::decode_budget::overflow)?;
                    let funding = budget.transient(length)?;
                    let mut full = Vec::with_capacity(length);
                    for part in [
                        &base[..prefix],
                        entry.changed_bytes.as_slice(),
                        &base[base.len() - suffix..],
                    ] {
                        for chunk in part.chunks(64 * 1024) {
                            budget.consult()?;
                            full.extend_from_slice(chunk);
                        }
                    }
                    (Cow::Owned(full), funding)
                }
                None if entry.shared_prefix_bytes == 0 && entry.shared_suffix_bytes == 0 => {
                    (Cow::Borrowed(entry.changed_bytes.as_slice()), None)
                }
                None => return Err(corrupt("unbased derived index checkpoint delta")),
            };
            if let Some(request) = budget.request() {
                budget.reserve(super::super::generation_decode_quote::entries_quote(
                    crate::durability::canonical_commit_artifact::borrowed_payload::BorrowedPayload(&entries_bytes), &request
                ).map_err(super::super::decode_budget::wire_stop)?)?;
            }
            let entries: DerivedIndexEntries = rmp_serde::from_slice(&entries_bytes)
                .map_err(|error| corrupt(format!("index entries decoding failed: {error}")))?;
            let generation = DerivedIndexGeneration {
                generation_id: entry.generation_id,
                index_id: entry.index_id,
                source_commit_id: entry.source_commit_id,
                source_branch_id: entry.source_branch_id.clone(),
                applicability: DerivedIndexApplicability {
                    branch_id: entry.applicability_branch_id.clone(),
                    version_id: entry.version_id,
                    schema_version: entry.schema_version,
                },
                status: entry.status,
                entries,
            };
            if generation.source_branch_id != generation.applicability.branch_id {
                return Err(corrupt(
                    "derived index checkpoint generation branch affinity mismatches source",
                ));
            }
            if controlled_digest(
                budget,
                &CheckpointDigestHeader::from_generation(&generation),
                &entries_bytes,
            )? != entry.full_digest
            {
                return Err(corrupt("derived index checkpoint digest mismatch"));
            }
            previous.insert(
                entry.index_id,
                (entry.generation_id, (entries_bytes, funding)),
            );
            generations.push(generation);
        }
        Ok(generations)
    }
}
fn corrupt(detail: impl Into<String>) -> Stop {
    super::corrupt(detail).into()
}
fn controlled_digest(
    budget: &IndexDecodeBudget<'_, '_>,
    header: &CheckpointDigestHeader<'_>,
    entries: &[u8],
) -> Result<[u8; 32], Stop> {
    match budget.request() {
        None => super::digest(header, entries).map_err(Into::into),
        Some(request) => {
            let header = crate::durability::artifact_payload_emission::request::encode_named(
                &[],
                header,
                &request,
            )?;
            for _ in entries.chunks(64 * 1024) {
                budget.consult()?;
            }
            super::digest_bytes(header.bytes(), entries).map_err(Into::into)
        }
    }
}
