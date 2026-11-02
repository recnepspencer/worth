//! Fully framed nested Entry content under the original finite scratch grant.
use super::super::content_storage::ProjectionContentStorageFault as Fault;
use super::*;
use worth_execution::ExecutionMemoryReservation;
struct Frame<'a> {
    rows: &'a [WorthQueryApplicationProjectionNode],
    row: usize,
    relation: usize,
    entered: bool,
}
impl<'a> Frame<'a> {
    fn new(rows: &'a [WorthQueryApplicationProjectionNode]) -> Self {
        Self {
            rows,
            row: 0,
            relation: 0,
            entered: false,
        }
    }
}
struct Scratch<'a> {
    frames: Vec<Frame<'a>>,
    _reservation: Option<ExecutionMemoryReservation>,
}
impl WorthQueryApplicationDisclosedProjectionTree {
    pub(in crate::domain_computation::primary_graph::application_query) fn append_tree_content<
        E,
    >(
        &self,
        writer: &mut ContentWriter<'_, E>,
        maximum_depth: usize,
        reserve: &mut impl FnMut(u64) -> Result<Option<ExecutionMemoryReservation>, E>,
        fault: &mut impl FnMut(Fault) -> E,
    ) -> Result<(), E> {
        let layout = std::alloc::Layout::array::<Frame<'_>>(maximum_depth)
            .map_err(|_| fault(Fault::CapacityOverflow))?;
        if maximum_depth == 0 {
            return Err(fault(Fault::CapacityOverflow));
        }
        let bytes = u64::try_from(layout.size()).map_err(|_| fault(Fault::CapacityOverflow))?;
        (writer.admit)(bytes)?;
        let reservation = reserve(bytes)?;
        let mut frames = Vec::new();
        frames
            .try_reserve_exact(maximum_depth)
            .map_err(|_| fault(Fault::AllocationFailed))?;
        let mut scratch = Scratch {
            frames,
            _reservation: reservation,
        };
        writer.number(self.raw_rows().len() as u64)?;
        scratch.frames.push(Frame::new(self.raw_rows()));
        while let Some(frame) = scratch.frames.last_mut() {
            (writer.admit)(1)?;
            if frame.row == frame.rows.len() {
                scratch.frames.pop();
                continue;
            }
            let node = &frame.rows[frame.row];
            if !frame.entered {
                writer.row(node)?;
                writer.number(node.relations.len() as u64)?;
                frame.entered = true;
            }
            if frame.relation == node.relations.len() {
                frame.row += 1;
                frame.relation = 0;
                frame.entered = false;
                continue;
            }
            let relation = &node.relations[frame.relation];
            frame.relation += 1;
            writer.bytes(relation.result_path.as_bytes())?;
            writer.bytes(relation.slot_type.as_bytes())?;
            writer.bytes(relation.slot_key.output_name().unwrap_or("").as_bytes())?;
            writer.number(match relation.cardinality {
                ApplicationQueryCardinality::ExactlyOne => 1,
                ApplicationQueryCardinality::OptionalOne => 2,
                ApplicationQueryCardinality::Many => 3,
            })?;
            writer.number(relation.predicate_sources.len() as u64)?;
            for entity in &relation.predicate_sources {
                writer.entity(*entity)?;
            }
            writer.number(relation.rows.len() as u64)?;
            if scratch.frames.len() == maximum_depth {
                return Err(fault(Fault::CapacityOverflow));
            }
            scratch.frames.push(Frame::new(&relation.rows));
        }
        (writer.admit)(0)
    }
}
