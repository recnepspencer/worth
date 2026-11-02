//! One finite original-funded continuation stack visits complete nested support.
use super::*;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
struct Frame<'a> {
    node: &'a WorthQueryApplicationProjectionNode,
    relation: usize,
    child: usize,
    entered: bool,
}
impl<'a> Frame<'a> {
    fn new(node: &'a WorthQueryApplicationProjectionNode) -> Self {
        Self {
            node,
            relation: 0,
            child: 0,
            entered: false,
        }
    }
}
pub(super) struct FootprintTraversal<'a> {
    frames: Vec<Frame<'a>>,
    nested_depth: Option<usize>,
    _reservation: Option<ExecutionMemoryReservation>,
}
impl<'a> FootprintTraversal<'a> {
    pub(super) fn admitted(
        execution: &ExecutionRequest<'_, '_>,
        contract: &WorthQueryInstalledGraphReadContract,
    ) -> Result<Self, WorthQueryApplicationReadExecutionDenial> {
        if contract
            .relations()
            .iter()
            .all(|relation| relation.parent_path() == "root")
        {
            return Ok(Self {
                frames: Vec::new(),
                nested_depth: None,
                _reservation: None,
            });
        }
        let maximum_depth = contract
            .relations()
            .len()
            .checked_add(1)
            .ok_or_else(|| denied(Cause::ChargedBytesOverflow))?;
        let layout = std::alloc::Layout::array::<Frame<'_>>(maximum_depth)
            .map_err(|_| denied(Cause::ChargedBytesOverflow))?;
        let bytes =
            u64::try_from(layout.size()).map_err(|_| denied(Cause::ChargedBytesOverflow))?;
        let reservation = execution
            .reserve_retained_memory(bytes)
            .map_err(|cause| denied(cause.into()))?;
        let mut frames = Vec::new();
        frames
            .try_reserve_exact(maximum_depth)
            .map_err(|_| denied(Cause::AllocationFailed))?;
        Ok(Self {
            frames,
            nested_depth: Some(maximum_depth),
            _reservation: reservation,
        })
    }
    #[allow(clippy::too_many_arguments)]
    pub(super) fn collect(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        projection: &VisibilityProjectionView,
        graph: &WorthQueryPrimaryGraphLayout,
        contract: &WorthQueryInstalledGraphReadContract,
        governance: &WorthQueryApplicationQueryGovernance,
        root: &'a WorthQueryApplicationProjectionNode,
        work: &mut ResultTreeWork,
        footprint: &mut WorthQueryObservedSourceFootprint,
        storage: &MaterialFootprintStorage<'_, '_, '_>,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        let Some(maximum_depth) = self.nested_depth else {
            collect_node_content(
                execution, projection, graph, contract, governance, root, work, footprint, storage,
            )?;
            for relation in root.relations() {
                for child in relation.rows() {
                    collect_node_content(
                        execution, projection, graph, contract, governance, child, work, footprint,
                        storage,
                    )?;
                }
            }
            return Ok(());
        };
        self.frames.push(Frame::new(root));
        while let Some(frame) = self.frames.last_mut() {
            work.checkpoint(frame.node.result_path())?;
            if !frame.entered {
                collect_node_content(
                    execution, projection, graph, contract, governance, frame.node, work,
                    footprint, storage,
                )?;
                frame.entered = true;
            }
            match frame.node.relations().get(frame.relation) {
                Some(relation) => match relation.rows().get(frame.child) {
                    Some(child) => {
                        frame.child += 1;
                        if self.frames.len() == maximum_depth {
                            return Err(projection_denial(child.result_path()));
                        }
                        self.frames.push(Frame::new(child));
                    }
                    None => {
                        frame.relation += 1;
                        frame.child = 0;
                    }
                },
                None => {
                    self.frames.pop();
                }
            }
        }
        Ok(())
    }
}
fn denied(cause: Cause) -> WorthQueryApplicationReadExecutionDenial {
    super::super::super::denial::relational_read_denial(cause, "Source footprint traversal")
}
