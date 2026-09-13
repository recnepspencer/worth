use super::super::*;
use super::actions::StagingActionBasis;
use super::materialization::ProjectedMaterializationBasis;
use super::pending::PendingProjectionBasis;

pub(super) fn assemble(
    selection: &PhysicalSourceSelection,
    selected_source: &RecoverySelectedSourceInventory,
    pending: &PendingProjectionBasis<'_>,
    materialization: ProjectedMaterializationBasis,
    actions: StagingActionBasis,
) -> Result<RecoveryStagingLayoutPlan, ExecutionBasisDenial> {
    let source_artifacts = selected_source_artifacts(selection, selected_source);
    let commands = super::super::command::exact_commands(
        materialization.frames.values().cloned(),
        materialization
            .manifests
            .values()
            .map(|manifest| (manifest.artifact(), manifest.bytes().into())),
        &source_artifacts,
    )?;
    let write_bytes = commands.iter().try_fold(0_u64, |bytes, command| {
        bytes.checked_add(command.byte_count())
    });
    let write_bytes = write_bytes.ok_or(ExecutionBasisDenial::Invalid)?;
    let base = base_image(selection, pending, materialization, source_artifacts);
    Ok(RecoveryStagingLayoutPlan {
        source_generation: pending.source_generation,
        staging_generation: pending.staging_generation,
        base,
        actions: actions.actions.into_boxed_slice(),
        commands,
        allocated_targets: actions.allocated_targets.into_boxed_slice(),
        allocated_bytes: pending.allocated_bytes,
        write_bytes,
    })
}

fn base_image(
    selection: &PhysicalSourceSelection,
    pending: &PendingProjectionBasis<'_>,
    materialization: ProjectedMaterializationBasis,
    source_artifacts: Box<[RecordArtifactFile]>,
) -> RecoveryBaseImagePlan {
    let ProjectedMaterializationBasis {
        frames: _,
        placements: projected_placements,
        projected_records,
        segment_updates,
        manifests,
        root_states,
    } = materialization;
    let mut placements = selection
        .page_facts()
        .placements()
        .iter()
        .map(|placement| (placement.record(), *placement))
        .collect::<BTreeMap<_, _>>();
    placements.extend(projected_placements);
    let actions = placements
        .into_values()
        .enumerate()
        .map(|(ordinal, placement)| base_action(ordinal, placement, &projected_records))
        .collect::<Vec<_>>()
        .into_boxed_slice();
    RecoveryBaseImagePlan {
        selected_selector: selection.root().selected().selector(),
        selected_root: selection.root().selected().manifest().clone(),
        selected_root_topology: selection
            .page_facts()
            .routing_topology()
            .to_vec()
            .into_boxed_slice(),
        destination_generation: pending.staging_generation,
        actions,
        segment_updates: segment_updates
            .into_values()
            .enumerate()
            .map(|(ordinal, update)| RecoverySegmentRoutingAction {
                ordinal: ordinal as u64,
                update,
            })
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        manifests: manifests
            .into_values()
            .enumerate()
            .map(|(ordinal, manifest)| RecoveryPayloadManifestAction {
                ordinal: ordinal as u64,
                artifact: manifest.artifact(),
            })
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        root_states: root_states.into_boxed_slice(),
        source_artifacts,
    }
}

fn selected_source_artifacts(
    selection: &PhysicalSourceSelection,
    selected_source: &RecoverySelectedSourceInventory,
) -> Box<[RecordArtifactFile]> {
    let selected = selection.root().selected();
    let mut artifacts = BTreeSet::from([
        match selected.selector().role() {
            worth_store_physical_format::RootSelectorRole::Current => {
                RecordArtifactFile::CurrentRootSelector
            }
            worth_store_physical_format::RootSelectorRole::Previous => {
                RecordArtifactFile::PreviousRootSelector
            }
        },
        RecordArtifactFile::RootManifest {
            generation: selected.manifest().generation(),
        },
    ]);
    artifacts.extend(
        selection
            .page_facts()
            .routing_blocks()
            .iter()
            .map(|reference| RecordArtifactFile::RootRoutingBlock {
                generation: reference.generation(),
                block: reference.block(),
            }),
    );
    artifacts.extend(
        selection
            .page_facts()
            .placements()
            .iter()
            .flat_map(placement_artifacts),
    );
    if let Some(previous) = selection.root().retained_previous() {
        artifacts.insert(RecordArtifactFile::PreviousRootSelector);
        artifacts.insert(RecordArtifactFile::RootManifest {
            generation: previous.manifest().generation(),
        });
    }
    if let Some(facts) = selection.retained_previous_page_facts() {
        artifacts.extend(facts.routing_blocks().iter().map(|reference| {
            RecordArtifactFile::RootRoutingBlock {
                generation: reference.generation(),
                block: reference.block(),
            }
        }));
        artifacts.extend(facts.placements().iter().flat_map(placement_artifacts));
    }
    artifacts.extend(selected_source.source_artifacts.iter().copied());
    artifacts.into_iter().collect::<Vec<_>>().into_boxed_slice()
}

fn placement_artifacts(
    placement: &CurrentPhysicalRecordPlacement,
) -> impl Iterator<Item = RecordArtifactFile> {
    let artifacts = match placement {
        CurrentPhysicalRecordPlacement::Inline(inline) => vec![RecordArtifactFile::Segment {
            segment: inline.segment().get(),
            generation: inline.segment_generation(),
        }],
        CurrentPhysicalRecordPlacement::Extent(extent) => vec![
            RecordArtifactFile::Extent {
                extent: extent.extent().get(),
                generation: extent.extent_generation(),
            },
            RecordArtifactFile::ExtentManifest {
                extent: extent.extent().get(),
                generation: extent.extent_generation(),
            },
        ],
    };
    artifacts.into_iter()
}

fn base_action(
    ordinal: usize,
    placement: CurrentPhysicalRecordPlacement,
    projected_records: &BTreeSet<worth_store_physical_format::PersistedRecordIdentity>,
) -> RecoveryBaseImageAction {
    if projected_records.contains(&placement.record()) {
        RecoveryBaseImageAction::ProjectRecoveryPlacement {
            ordinal: ordinal as u64,
            placement,
        }
    } else {
        RecoveryBaseImageAction::ReuseImmutableSelectedPlacement {
            ordinal: ordinal as u64,
            placement,
        }
    }
}
