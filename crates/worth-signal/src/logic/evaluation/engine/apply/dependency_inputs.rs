mod capture_work;
#[cfg(test)]
mod tests;
use crate::data::dependency::{
    CommittedSnapshotUpdate, DependencyEdge, DependencyInputScan, DependencySnapshot,
    ReplacementSnapshotUpdate, SnapshotDeltaRecord, SnapshotShapeHandle, StableShapeSnapshotBasis,
    VersionOnlySnapshotUpdate, VersionVector,
};
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::logic::evaluation::EvaluationWork;
use crate::logic::evaluation::{DependencyInputContext, EffectDependencyInputs};

use super::telemetry;

pub(super) fn resolve_effect_dependency_inputs(
    graph: &mut SignalGraph,
    node: NodeId,
    dependency_inputs: Option<EffectDependencyInputs>,
    work: &mut EvaluationWork<'_>,
) -> Result<EffectDependencyInputs, SignalError> {
    match dependency_inputs {
        Some(inputs) if dependency_inputs_match_graph(graph, node, &inputs)? => {
            telemetry::record_dependency_input_reuse(graph);
            Ok(inputs)
        }
        Some(_) => {
            telemetry::record_dependency_input_rebuild(graph);
            build_effect_dependency_inputs(graph, node, work)
        }
        None => build_effect_dependency_inputs(graph, node, work),
    }
}

fn dependency_inputs_match_graph(
    graph: &SignalGraph,
    node: NodeId,
    dependency_inputs: &EffectDependencyInputs,
) -> Result<bool, SignalError> {
    let (dependency_set_id, dependency_snapshot_id) = graph.node_dependency_ids(node)?;
    Ok(
        dependency_inputs.context.dependency_set_id == dependency_set_id
            && dependency_inputs.context.dependency_snapshot_id == dependency_snapshot_id,
    )
}

fn build_effect_dependency_inputs(
    graph: &mut SignalGraph,
    node: NodeId,
    work: &mut EvaluationWork<'_>,
) -> Result<EffectDependencyInputs, SignalError> {
    let (dependency_set_id, dependency_snapshot_id) = graph.node_dependency_ids(node)?;
    let context = DependencyInputContext {
        dependency_set_id,
        dependency_snapshot_id,
    };
    graph.refresh_runtime_dependencies_with_work(node, work)?;
    let dependencies = graph.current_runtime_dependencies_of(node)?;
    capture_work::dependencies(dependencies, work)?;
    let dependencies = dependencies.to_vec();
    build_effect_dependency_inputs_for_dependencies(
        graph,
        node,
        context,
        dependencies.as_slice(),
        work,
    )
}

pub(crate) fn collect_effect_dependency_inputs_iter<I>(
    graph: &mut SignalGraph,
    nodes: I,
) -> Result<Vec<EffectDependencyInputs>, SignalError>
where
    I: IntoIterator<Item = NodeId>,
{
    nodes
        .into_iter()
        .map(|node| build_effect_dependency_inputs(graph, node, &mut EvaluationWork::Ordinary))
        .collect()
}

pub(crate) fn build_effect_dependency_inputs_for_dependencies(
    graph: &mut SignalGraph,
    node: NodeId,
    context: DependencyInputContext,
    dependencies: &[DependencyEdge],
    work: &mut EvaluationWork<'_>,
) -> Result<EffectDependencyInputs, SignalError> {
    let shape_handle_lookup_start = crate::clock::RuntimeInstant::now();
    let previous_shape_handle = graph
        .dependency_snapshot_shape_handle_for_evaluation(context.dependency_snapshot_id, work)?;
    let shape_handle_lookup_nanos = shape_handle_lookup_start.elapsed().as_nanos();
    let previous_snapshot_fetch_start = crate::clock::RuntimeInstant::now();
    let previous_snapshot = graph.get_dep_snapshot(node)?.clone();
    let previous_snapshot_fetch_nanos = previous_snapshot_fetch_start.elapsed().as_nanos();
    let shape_scan = scan_dependency_shape(graph, dependencies, previous_snapshot.entries(), work)?;
    let (stable_shape_proved, inputs) = {
        if shape_scan.shape_stable
            && shape_scan.matched_entry_count == previous_snapshot.entries().len()
        {
            let inputs = build_stable_shape_dependency_inputs(
                graph,
                node,
                context,
                &previous_snapshot,
                previous_shape_handle,
                shape_scan.matched_entry_count,
                shape_scan.stable_shape_versions,
                shape_scan.changes,
                shape_handle_lookup_nanos,
                previous_snapshot_fetch_nanos,
                shape_scan.version_scan_nanos,
                work,
            )?;
            (true, inputs)
        } else {
            // `runtime_dependencies_of(node)` must preserve canonical dependency order
            // by `DependencyEdge::sort_key()`. Snapshot reuse and delta detection rely
            // on stable ordering between the current dependency view and the prior
            // snapshot entries.
            let inputs = build_replacement_dependency_inputs(
                graph,
                node,
                context,
                &previous_snapshot,
                dependencies,
                shape_handle_lookup_nanos,
                previous_snapshot_fetch_nanos,
                shape_scan.version_scan_nanos,
                work,
            )?;
            (false, inputs)
        }
    };
    telemetry::record_storage_shape_proof(graph, stable_shape_proved);
    Ok(inputs)
}

struct DependencyShapeScan {
    shape_stable: bool,
    matched_entry_count: usize,
    changes: u32,
    stable_shape_versions: Vec<u64>,
    version_scan_nanos: u128,
}

fn scan_dependency_shape(
    graph: &mut SignalGraph,
    dependencies: &[DependencyEdge],
    previous_entries: &[crate::data::dependency::DependencySnapshotEntry],
    work: &mut EvaluationWork<'_>,
) -> Result<DependencyShapeScan, SignalError> {
    let mut matched_entry_count = 0usize;
    let mut shape_stable = dependencies.len() == previous_entries.len();
    let mut changes = 0_u32;
    work.reserve(
        dependencies
            .len()
            .checked_mul(std::mem::size_of::<u64>() + 2),
    )?;
    let mut stable_shape_versions = Vec::with_capacity(dependencies.len());
    let version_scan_start = crate::clock::RuntimeInstant::now();
    for dep in dependencies {
        let source = dep.source();
        let Some(previous_entry) = previous_entries.get(matched_entry_count) else {
            shape_stable = false;
            break;
        };
        if !graph.is_alive(source) {
            shape_stable = false;
            break;
        }

        let version = capture_work::version(graph, dep, work)?;
        stable_shape_versions.push(version);
        capture_work::scope_comparison(dep.scope_ref(), work)?;
        if !previous_entry.compare_dependency(dep).is_eq() {
            shape_stable = false;
            break;
        }
        if previous_entry.cached_version != version {
            changes += 1;
        }
        matched_entry_count += 1;
    }
    Ok(DependencyShapeScan {
        shape_stable,
        matched_entry_count,
        changes,
        stable_shape_versions,
        version_scan_nanos: version_scan_start.elapsed().as_nanos(),
    })
}

fn build_stable_shape_dependency_inputs(
    graph: &mut SignalGraph,
    node: NodeId,
    context: DependencyInputContext,
    previous_snapshot: &DependencySnapshot,
    previous_shape_handle: SnapshotShapeHandle,
    previous_entry_count: usize,
    stable_shape_versions: Vec<u64>,
    changes: u32,
    shape_handle_lookup_nanos: u128,
    previous_snapshot_fetch_nanos: u128,
    version_scan_nanos: u128,
    work: &mut EvaluationWork<'_>,
) -> Result<EffectDependencyInputs, SignalError> {
    work.reserve(
        previous_snapshot
            .entries()
            .len()
            .checked_mul(3)
            .and_then(|n| n.checked_add(16)),
    )?;
    let stable_proof_start = crate::clock::RuntimeInstant::now();
    let scan = DependencyInputScan::stable_shape(
        node,
        context.dependency_snapshot_id,
        previous_entry_count,
        stable_shape_versions.len(),
        stable_shape_versions,
    );
    let basis = StableShapeSnapshotBasis::prove(&scan, previous_shape_handle).ok_or_else(|| {
        SignalError::internal("stable-shape dependency scan failed to produce a proof")
    })?;
    let versions = VersionVector::from_scan(&basis, &scan);
    let stable_proof_nanos = stable_proof_start.elapsed().as_nanos();
    let version_delta_start = crate::clock::RuntimeInstant::now();
    let snapshot_delta = SnapshotDeltaRecord::for_version_update(
        node,
        previous_snapshot,
        scan.stable_shape_versions(),
    );
    let dependency_snapshot_update = CommittedSnapshotUpdate::VersionOnly(
        VersionOnlySnapshotUpdate::from_basis_and_versions(basis, versions),
    );
    let version_delta_nanos = version_delta_start.elapsed().as_nanos();
    telemetry::record_stable_shape_timing(
        graph,
        shape_handle_lookup_nanos,
        previous_snapshot_fetch_nanos,
        version_scan_nanos,
        stable_proof_nanos,
        version_delta_nanos,
    );
    Ok(EffectDependencyInputs {
        context,
        snapshot_delta,
        dependency_snapshot_update,
        meaningful_input_changes: changes,
    })
}

fn build_replacement_dependency_inputs(
    graph: &mut SignalGraph,
    node: NodeId,
    context: DependencyInputContext,
    previous_snapshot: &DependencySnapshot,
    dependencies: &[DependencyEdge],
    shape_handle_lookup_nanos: u128,
    previous_snapshot_fetch_nanos: u128,
    version_scan_nanos: u128,
    work: &mut EvaluationWork<'_>,
) -> Result<EffectDependencyInputs, SignalError> {
    let replacement_build_start = crate::clock::RuntimeInstant::now();
    let (snapshot, changes) =
        build_replacement_dependency_snapshot(graph, dependencies, previous_snapshot, work)?;

    let replacement_snapshot =
        crate::data::dependency::SharedDependencySnapshot::new(snapshot.clone());
    capture_work::snapshot_comparison(previous_snapshot.entries(), snapshot.entries(), work)?;
    let snapshot_delta =
        SnapshotDeltaRecord::between(node, previous_snapshot, &replacement_snapshot);
    let dependency_snapshot_update = CommittedSnapshotUpdate::Replace(
        ReplacementSnapshotUpdate::from_snapshot_with_work(snapshot, work)?,
    );
    let replacement_build_nanos = replacement_build_start.elapsed().as_nanos();
    telemetry::record_replacement_timing(
        graph,
        shape_handle_lookup_nanos,
        previous_snapshot_fetch_nanos,
        version_scan_nanos,
        replacement_build_nanos,
    );
    Ok(EffectDependencyInputs {
        context,
        snapshot_delta,
        dependency_snapshot_update,
        meaningful_input_changes: changes,
    })
}

fn build_replacement_dependency_snapshot(
    graph: &mut SignalGraph,
    dependencies: &[DependencyEdge],
    previous_snapshot: &DependencySnapshot,
    work: &mut EvaluationWork<'_>,
) -> Result<(DependencySnapshot, u32), SignalError> {
    work.reserve(
        dependencies
            .len()
            .checked_mul(
                std::mem::size_of::<crate::data::dependency::DependencySnapshotEntry>() + 2,
            )
            .filter(|n| *n <= isize::MAX as usize),
    )?;
    let mut entries = Vec::with_capacity(dependencies.len());
    let snapshot_entries = previous_snapshot.entries();
    let mut snapshot_index = 0usize;
    let mut changes = 0_u32;
    for dep in dependencies {
        let source = dep.source();
        let aspect = dep.aspect();
        if graph.is_alive(source) {
            let ver = capture_work::version(graph, dep, work)?;
            capture_work::scope_copy(dep.scope_ref(), work)?;
            entries.push(crate::data::dependency::DependencySnapshotEntry {
                source,
                aspect,
                cached_version: ver,
                scope: dep.scope_ref().cloned(),
            });
            while snapshot_index < snapshot_entries.len() {
                capture_work::scope_comparison(dep.scope_ref(), work)?;
                if !snapshot_entries[snapshot_index]
                    .compare_dependency(dep)
                    .is_lt()
                {
                    break;
                }
                snapshot_index += 1;
            }
            capture_work::scope_comparison(dep.scope_ref(), work)?;
            if snapshot_index < snapshot_entries.len()
                && snapshot_entries[snapshot_index]
                    .compare_dependency(dep)
                    .is_eq()
            {
                if snapshot_entries[snapshot_index].cached_version != ver {
                    changes += 1;
                }
                snapshot_index += 1;
            }
        } else {
            changes += 1;
        }
    }
    capture_work::snapshot_comparison(&entries, &[], work)?;
    Ok((DependencySnapshot::from_ordered_unique(entries), changes))
}
