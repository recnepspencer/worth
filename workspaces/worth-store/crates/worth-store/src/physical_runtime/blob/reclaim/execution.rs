use std::num::NonZeroU32;

use crate::physical_runtime::durability::RetiredArtifact;
use crate::physical_runtime::PhysicalMutationDeadline;

use super::{
    admission::AdmittedBlobDrop,
    publication::{ReclaimPublication, ReclaimSource},
    BlobReclaimDisplacedExtent, BlobReclaimDisposition, BlobReclaimFailure, BlobReclaimHandle,
    BlobReclaimReceipt, BlobReclaimRetirement, BlobReclaimRetirementBudget,
};

pub(super) fn execute(
    mut handle: BlobReclaimHandle<'_>,
) -> Result<BlobReclaimReceipt, BlobReclaimFailure> {
    if let Some(admitted) = handle.manifest_residue.take() {
        return execute_manifest_residue(handle, admitted);
    }
    let Some(admitted) = handle.admitted.take() else {
        return Ok(BlobReclaimReceipt {
            store: handle.runtime.store_identity(),
            runtime: handle.runtime.runtime_identity(),
            disposition: BlobReclaimDisposition::ProvenNoEffect,
            dropped: Vec::new(),
            displaced: Vec::new(),
            completed: Vec::new(),
            remaining_payload_records: 0,
            bytes_released: 0,
            retirement: BlobReclaimRetirement::NotRequired,
            observation: handle.observation,
        });
    };
    // Materialize receipt storage before the first durable effect, while the
    // operation's bounded allocation covers all simultaneously live vectors.
    let mut dropped = Vec::new();
    dropped
        .try_reserve_exact(admitted.dropped().len())
        .map_err(|_| BlobReclaimFailure::ScratchUnavailable)?;
    dropped.extend_from_slice(admitted.dropped());
    let mut displaced = Vec::new();
    displaced
        .try_reserve_exact(admitted.displaced().len())
        .map_err(|_| BlobReclaimFailure::ScratchUnavailable)?;
    for entry in admitted.displaced() {
        let RetiredArtifact::Extent {
            extent,
            generation,
            range,
        } = entry.artifact
        else {
            return Err(BlobReclaimFailure::ConflictingSelectedFate);
        };
        displaced.push(BlobReclaimDisplacedExtent {
            extent,
            generation,
            range,
            source_root: entry.source_root,
            bytes: entry.bytes,
        });
    }
    displaced.sort_unstable_by_key(super::retirement::extent_key);
    if displaced.windows(2).any(|pair| {
        super::retirement::extent_key(&pair[0]) == super::retirement::extent_key(&pair[1])
    }) {
        return Err(BlobReclaimFailure::ConflictingSelectedFate);
    }
    let mut completed = Vec::new();
    completed
        .try_reserve_exact(displaced.len())
        .map_err(|_| BlobReclaimFailure::ScratchUnavailable)?;
    completed.resize(displaced.len(), false);
    let remaining_payload_records = admitted.remaining();
    let source = match &admitted {
        AdmittedBlobDrop::Failed(value) => ReclaimSource::Failed(value),
        AdmittedBlobDrop::Released(value) => ReclaimSource::Released(value),
    };
    let retired_source_root = ReclaimPublication {
        runtime: handle.runtime,
        admitted: source,
        placement: handle.placement,
        deadline: handle.deadline,
        limits: handle.limits,
        allocation: &handle._allocation,
    }
    .publish()?;
    for extent in &mut displaced {
        extent.source_root = retired_source_root;
    }
    if !admitted.complete() {
        return Err(BlobReclaimFailure::FenceLost);
    }
    let mut receipt = BlobReclaimReceipt {
        store: handle.runtime.store_identity(),
        runtime: handle.runtime.runtime_identity(),
        disposition: BlobReclaimDisposition::Dropped,
        dropped,
        displaced,
        completed,
        remaining_payload_records,
        bytes_released: 0,
        retirement: BlobReclaimRetirement::AwaitingRetirement,
        observation: handle.observation,
    };
    let maximum_work = NonZeroU32::new(receipt.displaced.len().min(1024) as u32)
        .expect("a published reclaim drops a nonempty bounded extent set");
    let budget = BlobReclaimRetirementBudget::new(
        maximum_work,
        PhysicalMutationDeadline::after_milliseconds(30_000)
            .expect("the initial retirement deadline is positive"),
    );
    super::retirement::retire_batch(handle.runtime, &mut receipt, budget);
    Ok(receipt)
}

fn execute_manifest_residue(
    handle: BlobReclaimHandle<'_>,
    admitted: crate::physical_runtime::durability::AdmittedManifestResidueRetirement,
) -> Result<BlobReclaimReceipt, BlobReclaimFailure> {
    let removed = admitted.removed_records();
    let displaced_facts = admitted.displaced_manifest();
    let count = displaced_facts.count() as usize;
    // Receipt storage is admitted before the root-only durable effect.
    let mut dropped = Vec::new();
    dropped
        .try_reserve_exact(count)
        .map_err(|_| BlobReclaimFailure::ScratchUnavailable)?;
    dropped.extend(removed.into_iter().flatten());
    dropped.sort_unstable();
    if dropped.len() != count || dropped.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(BlobReclaimFailure::ConflictingSelectedFate);
    }
    let mut displaced = Vec::new();
    displaced
        .try_reserve_exact(count)
        .map_err(|_| BlobReclaimFailure::ScratchUnavailable)?;
    for fact in [Some(displaced_facts.manifest), displaced_facts.reserved]
        .into_iter()
        .flatten()
    {
        let RetiredArtifact::Extent {
            extent,
            generation,
            range,
        } = fact.artifact
        else {
            return Err(BlobReclaimFailure::ConflictingSelectedFate);
        };
        displaced.push(BlobReclaimDisplacedExtent {
            extent,
            generation,
            range,
            source_root: fact.source_root,
            bytes: fact.bytes,
        });
    }
    displaced.sort_unstable_by_key(super::retirement::extent_key);
    if displaced.windows(2).any(|pair| {
        super::retirement::extent_key(&pair[0]) == super::retirement::extent_key(&pair[1])
    }) {
        return Err(BlobReclaimFailure::ConflictingSelectedFate);
    }
    let mut completed = Vec::new();
    completed
        .try_reserve_exact(count)
        .map_err(|_| BlobReclaimFailure::ScratchUnavailable)?;
    completed.resize(count, false);
    let (source_root, retired) = handle
        .runtime
        .publish_blob_manifest_residue(admitted)
        .map_err(BlobReclaimFailure::ManifestResiduePublication)?;
    if retired.manifest != displaced_facts.manifest
        || retired.reserved != displaced_facts.reserved
        || retired.manifest.source_root != source_root
        || retired
            .reserved
            .is_some_and(|fact| fact.source_root != source_root)
    {
        return Err(BlobReclaimFailure::ConflictingSelectedFate);
    }
    for extent in &mut displaced {
        extent.source_root = source_root;
    }
    let mut receipt = BlobReclaimReceipt {
        store: handle.runtime.store_identity(),
        runtime: handle.runtime.runtime_identity(),
        disposition: BlobReclaimDisposition::Dropped,
        dropped,
        displaced,
        completed,
        remaining_payload_records: 0,
        bytes_released: 0,
        retirement: BlobReclaimRetirement::AwaitingRetirement,
        observation: handle.observation,
    };
    let budget = BlobReclaimRetirementBudget::new(
        NonZeroU32::new(count as u32).expect("one or two metadata extents"),
        PhysicalMutationDeadline::after_milliseconds(30_000)
            .expect("the initial retirement deadline is positive"),
    );
    super::retirement::retire_batch(handle.runtime, &mut receipt, budget);
    Ok(receipt)
}
