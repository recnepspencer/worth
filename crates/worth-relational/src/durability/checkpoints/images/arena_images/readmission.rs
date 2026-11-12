//! One arena conversion, with original-request admission at each native owner.
use super::*;
use crate::durability::data::RelationalRecoveryStop as Stop;
use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::storage::substrate::{SharedColumn, SharedMap, StorageAllocation, StoredValue};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

pub(crate) struct ImageReadmission<'a, 's, 'h> {
    pub(crate) request: Option<&'a ExecutionRequest<'s, 'h>>,
}
impl ImageReadmission<'_, '_, '_> {
    pub(crate) fn charge(&self, units: u64) -> Result<(), Stop> {
        let Some(request) = self.request else {
            return Ok(());
        };
        let mut outcome = None;
        let run = request.run_native_with_checkpoint(|context| {
            outcome = Some(crate::execution::request_stage::checkpoint(context, units));
        });
        match (run, outcome) {
            (_, Some(Err(cause))) => Err(Stop::Request(
                worth_execution::WorkCeilingDenial::Stopped(cause).into(),
            )),
            (Err(cause), _) => Err(Stop::Request(cause.into())),
            (Ok(_), Some(Ok(()))) => Ok(()),
            (Ok(_), None) => unreachable!("completed charge executes callback"),
        }
    }
    pub(crate) fn reserve(&self, bytes: u64) -> Result<Option<ExecutionMemoryReservation>, Stop> {
        self.request
            .map(|request| {
                request
                    .reserve_retained_memory(bytes)
                    .map_err(|cause| Stop::Request(cause.into()))
            })
            .transpose()
            .map(Option::flatten)
    }
    pub(crate) fn scratch<T>(
        &self,
        count: usize,
    ) -> Result<Option<ExecutionMemoryReservation>, Stop> {
        let bytes = std::alloc::Layout::array::<T>(count)
            .ok()
            .and_then(|layout| u64::try_from(layout.size()).ok())
            .ok_or(Stop::Request(Cause::ChargedBytesOverflow))?;
        self.charge(bytes)?;
        self.reserve(bytes)
    }
    fn column<T: StoredValue>(&self, values: Vec<T>) -> Result<SharedColumn<T>, Stop> {
        match self.request {
            Some(request) => {
                SharedColumn::from_dense_carriers_for_request(values, request).map_err(Stop::from)
            }
            None => Ok(SharedColumn::from_dense_carriers(values)),
        }
    }
    fn map<K: Ord + Copy, V: StoredValue>(
        &self,
        values: impl IntoIterator<Item = (K, V::Value)>,
    ) -> Result<SharedMap<K, V>, Stop> {
        let mut result = SharedMap::default();
        for (key, value) in values {
            self.charge(1)?;
            match self.request {
                Some(request) => {
                    result.insert_for_request(key, value, request)?;
                }
                None => {
                    result.insert(key, value);
                }
            }
        }
        Ok(result)
    }
    fn bitset(
        &self,
        lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
        value: DurableBitSet,
    ) -> Result<crate::storage::partition::DenseSlotBitSet, Stop> {
        match self.request {
            Some(request) => {
                super::super::bitset_image::restore_bitset_for_request(lane, value, request)
                    .map_err(Stop::from)
            }
            None => Ok(restore_bitset(lane, value)),
        }
    }
}
fn corrupt(detail: &'static str) -> Stop {
    DurabilityError::new(RecoveryFailureClass::CorruptCheckpoint, detail).into()
}
pub(super) fn arena<K: CheckpointArenaKind>(
    lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
    partition_id: PartitionId,
    mut image: RecordArenaCheckpointImage<K::ImageKind>,
    catalog: &AspectContractPlanCatalog,
    contracts: &CheckpointAspectContractCatalog,
    admission: &ImageReadmission<'_, '_, '_>,
) -> Result<RecordArena<K>, Stop> {
    let n = image.generations.len();
    if [
        image.lifecycle.len(),
        image.kind_ids.len(),
        image.metadata_history.len(),
        image.created_at.len(),
        image.retired_at.len(),
        image.extra.len(),
        image.aspect_versions.len(),
        image.field_revisions.len(),
        image.diagnostics_enrichment.len(),
        image.branch_pins.len(),
        image.replay_pins.len(),
        image.snapshot_pins.len(),
    ]
    .into_iter()
    .any(|len| len != n)
    {
        return Err(corrupt("record arena SoA column lengths differ"));
    }
    let _implicit_slots = admission.scratch::<u64>(if image.slots.is_empty() { n } else { 0 })?;
    let slots = if image.slots.is_empty() && n != 0 {
        (0..n as u64).collect()
    } else {
        std::mem::take(&mut image.slots)
    };
    if slots.len() != n {
        return Err(corrupt("record arena slot directory length differs"));
    }
    let slots = match admission.request {
        Some(request) => crate::storage::substrate::RecordSlotDirectory::restore_for_request(
            lane, slots, request,
        )
        .map_err(|cause| match cause {
            crate::storage::substrate::MetadataWriteDenial::Request(cause) => Stop::Request(cause),
            crate::storage::substrate::MetadataWriteDenial::Invalid(detail) => corrupt(detail),
        })?,
        None => {
            crate::storage::substrate::RecordSlotDirectory::restore(lane, slots).map_err(corrupt)?
        }
    };
    let _metadata_rows = admission.scratch::<SharedColumn<StorageAllocation<K::Meta>>>(n)?;
    let mut metadata = Vec::with_capacity(n);
    for entries in image.metadata_history {
        let _row = admission.scratch::<StorageAllocation<K::Meta>>(entries.len())?;
        let mut row = Vec::with_capacity(entries.len());
        for value in entries {
            admission.charge(1)?;
            let kind = K::meta_kind_from_image(&value);
            K::plan(catalog, kind).ok_or_else(|| missing_plan(kind))?;
            admission.charge(StorageAllocation::<K::Meta>::layout_bytes())?;
            row.push(StorageAllocation::new(K::meta_from_image_using(
                value, contracts, admission,
            )?));
        }
        metadata.push(admission.column(row)?);
    }
    let _extra_rows = admission.scratch::<StorageAllocation<K::Extra>>(n)?;
    let mut extras = Vec::with_capacity(n);
    for (value, kind) in image.extra.into_iter().zip(image.kind_ids.iter().copied()) {
        admission.charge(1)?;
        if let Some(kind) = kind {
            K::plan(catalog, kind).ok_or_else(|| missing_plan(kind))?;
        }
        extras.push(StorageAllocation::new(K::extra_from_image_using(
            value, contracts, admission,
        )?));
    }
    let _aspect_rows = admission.scratch::<crate::storage::substrate::AspectVersions>(n)?;
    let mut aspects = Vec::with_capacity(n);
    for row in image.aspect_versions {
        aspects.push(admission.map(row)?);
    }
    let _field_rows = admission.scratch::<Option<crate::storage::substrate::FieldRevisions>>(n)?;
    let mut fields = Vec::with_capacity(n);
    for row in image.field_revisions {
        fields.push(row.map(|row| admission.map(row)).transpose()?);
    }
    let _diagnostic_rows = admission.scratch::<StorageAllocation<
        std::collections::BTreeMap<crate::symbols::data::Symbol, String>,
    >>(n)?;
    let mut diagnostics = Vec::with_capacity(n);
    for row in image.diagnostics_enrichment {
        let mut bytes = StorageAllocation::<
            std::collections::BTreeMap<crate::symbols::data::Symbol, String>,
        >::layout_bytes();
        let node = std::alloc::Layout::new::<(
            usize,
            u16,
            u16,
            [crate::symbols::data::Symbol; 11],
            [String; 11],
            [usize; 12],
        )>()
        .size() as u64;
        bytes = bytes
            .checked_add(
                node.checked_mul(if row.is_empty() {
                    0
                } else {
                    (row.len().div_ceil(5) + 2) as u64
                })
                .ok_or(Cause::ChargedBytesOverflow)?,
            )
            .ok_or(Cause::ChargedBytesOverflow)?;
        for value in row.values() {
            admission.charge(1)?;
            bytes = bytes
                .checked_add(value.capacity() as u64)
                .ok_or(Cause::ChargedBytesOverflow)?;
        }
        admission.charge(StorageAllocation::<
            std::collections::BTreeMap<crate::symbols::data::Symbol, String>,
        >::layout_bytes())?;
        diagnostics.push(StorageAllocation::from_reserved(
            row,
            admission.reserve(bytes)?,
        ));
    }
    let _partitions = admission.scratch::<PartitionId>(n)?;
    let default_memory = admission.reserve(StorageAllocation::<
        Option<crate::storage::substrate::RecordHistoryPrefix>,
    >::layout_bytes())?;
    let mut result = RecordArena {
        slots,
        partition_ids: admission.column(vec![partition_id; n])?,
        history_prefixes: SharedColumn::with_default_carrier(
            n,
            StorageAllocation::from_reserved(None, default_memory),
        ),
        generations: admission.column(image.generations)?,
        lifecycle: admission.column(image.lifecycle)?,
        kind_ids: admission.column(image.kind_ids)?,
        metadata_history: admission.column(metadata)?,
        created_at: admission.column(image.created_at)?,
        retired_at: admission.column(image.retired_at)?,
        extra: admission.column(extras)?,
        aspect_versions: admission.column(aspects)?,
        field_revisions: admission.column(fields)?,
        diagnostics_enrichment: admission.column(diagnostics)?,
        branch_pins: Vec::new().into(),
        replay_pins: Vec::new().into(),
        snapshot_pins: Vec::new().into(),
        live_bitset: admission.bitset(lane, image.live_bitset)?,
        reclaimable_bitset: admission.bitset(lane, image.reclaimable_bitset)?,
    };
    match admission.request {
        Some(request) => {
            result
                .branch_pins
                .restore_values_for_request(image.branch_pins, request)?;
            result
                .replay_pins
                .restore_values_for_request(image.replay_pins, request)?;
            result
                .snapshot_pins
                .restore_values_for_request(image.snapshot_pins, request)?;
        }
        None => {
            result.branch_pins = image.branch_pins.into();
            result.replay_pins = image.replay_pins.into();
            result.snapshot_pins = image.snapshot_pins.into();
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn excess_scalar_columns_refuse_before_schema_lookup() {
        let changes: [fn(&mut RecordArenaCheckpointImage<EntityCheckpointImageKind>); 4] = [
            |image| image.created_at.push(crate::identity::data::VersionId(1)),
            |image| image.retired_at.push(None),
            |image| image.kind_ids.push(None),
            |image| {
                image
                    .lifecycle
                    .push(crate::storage::data::RecordLifecycleState::Live)
            },
        ];
        for change in changes {
            let mut image = RecordArenaCheckpointImage::<EntityCheckpointImageKind> {
                slots: vec![],
                generations: vec![],
                lifecycle: vec![],
                kind_ids: vec![],
                metadata_history: vec![],
                created_at: vec![],
                retired_at: vec![],
                aspect_versions: vec![],
                field_revisions: vec![],
                extra: vec![],
                diagnostics_enrichment: vec![],
                branch_pins: vec![],
                replay_pins: vec![],
                snapshot_pins: vec![],
                live_bitset: DurableBitSet {
                    words: vec![],
                    sparse_words: vec![],
                },
                reclaimable_bitset: DurableBitSet {
                    words: vec![],
                    sparse_words: vec![],
                },
                free_list: vec![],
                marker: std::marker::PhantomData,
            };
            change(&mut image);
            let contracts = CheckpointAspectContractCatalog::from_contracts(&[]).unwrap();
            let error = arena::<EntityRecordKind>(
                &crate::storage::substrate::maintenance::ReconstructionLane::oracle(),
                PartitionId::main(),
                image,
                &AspectContractPlanCatalog::empty(),
                &contracts,
                &ImageReadmission { request: None },
            )
            .unwrap_err();
            assert!(matches!(error, Stop::Durability(error)
                if error.class == RecoveryFailureClass::CorruptCheckpoint));
        }
    }
}
