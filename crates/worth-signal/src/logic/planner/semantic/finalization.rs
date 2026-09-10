use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::diagnostics::recorder::stamp_trace_summary_and_record_lineage_transition_from_image;

use super::super::apply::serial_batch::{FinalizedSerialStageBatch, ReadySerialFinalizeBatch};
use super::super::reporting::classify_task_execution_record;
use super::super::types::{ExecutionRecordId, ExecutionReport, SemanticTaskRange};
use super::artifacts::record_semantic_artifacts;
use super::reporting::record_semantic_update;
#[cfg(feature = "parallel")]
use super::segments::StageSemanticBatch;

#[cfg(feature = "parallel")]
use super::super::types::EligibleTask;
#[cfg(feature = "parallel")]
use super::segments::{SemanticSegment, SemanticTaskUpdate};

#[cfg(feature = "parallel")]
pub(in crate::logic::planner) fn finalize_stage_batch(
    graph: &mut SignalGraph,
    stage_tasks: &[EligibleTask],
    batch: StageSemanticBatch,
    report: &mut ExecutionReport,
    stage_record: &mut super::super::types::StageExecutionRecord,
) -> Result<(), SignalError> {
    if batch.is_empty() {
        return Ok(());
    }

    let mut segments = batch.into_segments();
    if !segments_are_sorted(segments.as_slice()) {
        segments.sort_by_key(|segment| (segment.task_range().start.0, segment.id().0));
    }
    let Some(first_segment) = segments.first() else {
        return Err(SignalError::internal(
            "semantic finalize expected at least one segment after non-empty batch validation",
        ));
    };
    let Some(last_segment) = segments.last() else {
        return Err(SignalError::internal(
            "semantic finalize expected a tail segment after non-empty batch validation",
        ));
    };
    stage_record.semantic_segment_count = segments.len() as u32;
    report.semantic_segment_count += segments.len() as u32;
    stage_record.semantic_task_range = Some(SemanticTaskRange {
        start: first_segment.task_range().start,
        end: last_segment.task_range().end,
    });

    let mut task_records = Vec::with_capacity(stage_tasks.len());
    for segment in segments {
        finalize_segment(graph, stage_tasks, segment, report, &mut task_records)?;
    }
    if !task_records_are_sorted(task_records.as_slice()) {
        task_records.sort_by_key(|record| record.id.0);
    }
    stage_record.task_records = task_records;
    Ok(())
}

#[cfg(feature = "parallel")]
fn finalize_segment(
    graph: &mut SignalGraph,
    stage_tasks: &[EligibleTask],
    segment: SemanticSegment,
    report: &mut ExecutionReport,
    task_records: &mut Vec<super::super::types::TaskExecutionRecord>,
) -> Result<(), SignalError> {
    for update in segment.into_updates() {
        finalize_update(graph, stage_tasks, update, report, task_records)?;
    }
    Ok(())
}

#[cfg(feature = "parallel")]
fn finalize_update(
    graph: &mut SignalGraph,
    stage_tasks: &[EligibleTask],
    update: SemanticTaskUpdate,
    report: &mut ExecutionReport,
    task_records: &mut Vec<super::super::types::TaskExecutionRecord>,
) -> Result<(), SignalError> {
    let (
        task_index,
        node,
        identity,
        before_state,
        before_artifact_state,
        after_state,
        dependency_updates,
        recomputed,
        partition_aware,
        temporal_eligibility,
        rewiring,
        verdict,
        memoized_origin,
        reuse_basis,
    ) = update.into_parts();
    let after_finalize_image = graph.node_runtime_artifact_finalize_image(node)?;
    if let Some(after_finalize_image) = after_finalize_image.as_ref() {
        stamp_trace_summary_and_record_lineage_transition_from_image(
            graph,
            node,
            before_artifact_state.as_ref(),
            after_finalize_image,
            identity.record_id,
            identity.segment_id,
            &mut crate::logic::evaluation::EvaluationWork::Ordinary,
        )?;
    }
    let task = &stage_tasks[task_index];
    let task_record = classify_task_execution_record(
        identity.record_id,
        identity.segment_id,
        task,
        before_state,
        after_state,
        before_artifact_state.as_ref(),
        after_finalize_image.as_ref(),
        verdict,
        temporal_eligibility,
        memoized_origin,
        reuse_basis,
    );
    record_semantic_update(
        graph,
        report,
        &task_record,
        dependency_updates,
        recomputed,
        partition_aware,
    );
    record_semantic_artifacts(graph, node, rewiring.as_ref())?;
    task_records.push(task_record);
    Ok(())
}

pub(in crate::logic::planner) fn finalize_serial_stage_batch(
    graph: &mut SignalGraph,
    batch: ReadySerialFinalizeBatch,
    report: &mut ExecutionReport,
    stage_record: &mut super::super::types::StageExecutionRecord,
) -> Result<FinalizedSerialStageBatch, SignalError> {
    let (stage_tasks, finalize_seeds, applied_tasks, _stage_order) = batch.into_parts();

    if finalize_seeds.is_empty() {
        let empty_range = SemanticTaskRange {
            start: ExecutionRecordId(0),
            end: ExecutionRecordId(0),
        };
        return Ok(FinalizedSerialStageBatch::new(empty_range, Vec::new(), 0));
    }
    let Some(first_seed) = finalize_seeds.first() else {
        return Err(SignalError::internal(
            "serial finalize expected a first seed after non-empty seed validation",
        ));
    };
    let Some(last_seed) = finalize_seeds.last() else {
        return Err(SignalError::internal(
            "serial finalize expected a tail seed after non-empty seed validation",
        ));
    };

    let semantic_task_range = SemanticTaskRange {
        start: first_seed.identity.record_id,
        end: last_seed.identity.record_id,
    };
    let mut task_records = Vec::with_capacity(applied_tasks.len());

    for (seed, applied) in finalize_seeds.into_iter().zip(applied_tasks.into_iter()) {
        if seed.node != applied.node {
            return Err(SignalError::internal(
                "serial finalize proof was violated: stage-ordered seed and applied node diverged",
            ));
        }
        let after_finalize_image = graph.node_runtime_artifact_finalize_image(applied.node)?;
        if let Some(after_finalize_image) = after_finalize_image.as_ref() {
            stamp_trace_summary_and_record_lineage_transition_from_image(
                graph,
                applied.node,
                seed.before_artifact_state.as_ref(),
                after_finalize_image,
                seed.identity.record_id,
                seed.identity.segment_id,
                &mut crate::logic::evaluation::EvaluationWork::Ordinary,
            )?;
        }
        let task = &stage_tasks[seed.task_index];
        let task_record = classify_task_execution_record(
            seed.identity.record_id,
            seed.identity.segment_id,
            task,
            seed.before_state,
            applied.after_state,
            seed.before_artifact_state.as_ref(),
            after_finalize_image.as_ref(),
            applied.verdict,
            applied.temporal_eligibility,
            applied.memoized_origin,
            applied.reuse_basis,
        );
        record_semantic_update(
            graph,
            report,
            &task_record,
            seed.dependency_updates,
            seed.recomputed,
            seed.partition_aware,
        );
        record_semantic_artifacts(graph, applied.node, seed.rewiring.as_ref())?;
        task_records.push(task_record);
    }

    let semantic_segment_count = task_records.len() as u32;
    stage_record.semantic_segment_count = semantic_segment_count;
    Ok(FinalizedSerialStageBatch::new(
        semantic_task_range,
        task_records,
        semantic_segment_count,
    ))
}

#[cfg(feature = "parallel")]
fn segments_are_sorted(segments: &[SemanticSegment]) -> bool {
    segments.windows(2).all(|pair| {
        pair[0].task_range().start.0 < pair[1].task_range().start.0
            || (pair[0].task_range().start.0 == pair[1].task_range().start.0
                && pair[0].id().0 <= pair[1].id().0)
    })
}

#[cfg(feature = "parallel")]
fn task_records_are_sorted(records: &[super::super::types::TaskExecutionRecord]) -> bool {
    records.windows(2).all(|pair| pair[0].id.0 <= pair[1].id.0)
}
