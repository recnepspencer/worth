//! Capture only the definition lane and allocator structure, never warm or hot state.
use super::SignalGraph;
use crate::durability::{SignalAuthoredImage, SignalAuthoredImageDenial};

impl SignalGraph {
    /// Capture authored node declarations and static edges in Signal's byte format.
    /// Cold capture costs O(node slots + authored declaration bytes).
    pub fn capture_authored_image(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<SignalAuthoredImage, SignalAuthoredImageDenial> {
        crate::durability::authored_image::encode(execution, &mut |out| {
            out.count(self.arena.nodes.len())?;
            for (index, slot) in self.arena.nodes.iter().enumerate() {
                out.small_number(slot.generation)?;
                out.boolean(slot.retired)?;
                out.boolean(slot.occupied)?;
                if slot.occupied {
                    crate::durability::authored_image::write_definition(
                        out,
                        &self.arena.definitions[index],
                    )?;
                }
            }
            out.count(self.arena.free_list.len())?;
            for index in self.arena.free_list.iter() {
                out.small_number(*index)?;
            }
            Ok(())
        })
    }
}
