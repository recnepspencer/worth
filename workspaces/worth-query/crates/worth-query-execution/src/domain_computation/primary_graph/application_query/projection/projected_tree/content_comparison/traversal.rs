//! Finite original-funded traversal preserves complete Source provenance.
use super::ProjectionContentComparisonFault as Fault;
use super::*;
use worth_execution::ExecutionMemoryReservation;

struct Frame<'a> {
    left: &'a [WorthQueryApplicationProjectionNode],
    right: &'a [WorthQueryApplicationProjectionNode],
    row: usize,
    relation: usize,
    checked: bool,
}
impl<'a> Frame<'a> {
    fn new(
        left: &'a [WorthQueryApplicationProjectionNode],
        right: &'a [WorthQueryApplicationProjectionNode],
    ) -> Self {
        Self {
            left,
            right,
            row: 0,
            relation: 0,
            checked: false,
        }
    }
}
struct Frames<'a> {
    // Rust drops fields in declaration order: backing before its payer.
    stack: Vec<Frame<'a>>,
    _reservation: Option<ExecutionMemoryReservation>,
}
impl WorthQueryApplicationDisclosedProjectionTree {
    pub(in crate::domain_computation::primary_graph::application_query) fn same_source_content<
        E,
    >(
        &self,
        other: &Self,
        maximum_depth: usize,
        other_depth: usize,
        admit: &mut impl FnMut(u64) -> Result<(), E>,
        reserve: &mut (impl FnMut(u64) -> Result<Option<ExecutionMemoryReservation>, E> + ?Sized),
        fault: &mut impl FnMut(Fault) -> E,
        compare_frames: &mut dyn FnMut(
            &worth_relational::facade::content::RelationalContentFrame,
            &worth_relational::facade::content::RelationalContentFrame,
        ) -> Result<bool, E>,
    ) -> Result<bool, E> {
        admit(1)?;
        if maximum_depth != other_depth {
            return Ok(false);
        }
        if maximum_depth <= 2 && maximum_depth != 0 {
            // Existing shallow profiles keep their exact allocation-free path.
            return self.same_one_hop_content(other, admit, fault, compare_frames);
        }
        let mut frames = frames(maximum_depth, admit, reserve, fault)?;
        frames
            .stack
            .push(Frame::new(self.raw_rows(), other.raw_rows()));
        while let Some(frame) = frames.stack.last_mut() {
            admit(1)?;
            if frame.left.len() != frame.right.len() {
                return Ok(false);
            }
            if frame.row == frame.left.len() {
                frames.stack.pop();
                continue;
            }
            let left = &frame.left[frame.row];
            let right = &frame.right[frame.row];
            if !frame.checked {
                if !same_row(left, right, admit, fault, compare_frames)?
                    || left.relations.len() != right.relations.len()
                {
                    return Ok(false);
                }
                frame.checked = true;
            }
            if frame.relation == left.relations.len() {
                frame.row += 1;
                frame.relation = 0;
                frame.checked = false;
                continue;
            }
            let left = &left.relations[frame.relation];
            let right = &right.relations[frame.relation];
            frame.relation += 1;
            if !same_relation_header(left, right, admit)? {
                return Ok(false);
            }
            if frames.stack.len() == maximum_depth {
                return Err(fault(Fault::CapacityOverflow));
            }
            frames.stack.push(Frame::new(&left.rows, &right.rows));
        }
        admit(0)?;
        Ok(true)
    }
}
fn frames<'a, E>(
    maximum_depth: usize,
    admit: &mut impl FnMut(u64) -> Result<(), E>,
    reserve: &mut (impl FnMut(u64) -> Result<Option<ExecutionMemoryReservation>, E> + ?Sized),
    fault: &mut impl FnMut(Fault) -> E,
) -> Result<Frames<'a>, E> {
    if maximum_depth == 0 {
        return Err(fault(Fault::CapacityOverflow));
    }
    let layout = std::alloc::Layout::array::<Frame<'a>>(maximum_depth)
        .map_err(|_| fault(Fault::CapacityOverflow))?;
    let bytes = u64::try_from(layout.size()).map_err(|_| fault(Fault::CapacityOverflow))?;
    admit(bytes)?;
    let reservation = reserve(bytes)?;
    let mut stack = Vec::new();
    stack
        .try_reserve_exact(maximum_depth)
        .map_err(|_| fault(Fault::AllocationFailed))?;
    Ok(Frames {
        stack,
        _reservation: reservation,
    })
}
