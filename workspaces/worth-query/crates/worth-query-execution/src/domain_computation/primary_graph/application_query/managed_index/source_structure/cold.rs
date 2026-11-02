//! One invocation-local DFS over the immutable functional parent graph.
use super::{maps, Control, ExecutionRequest, Map, Stop};
#[cfg(test)]
mod tests;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Color {
    Gray,
    Black,
}

/// The caller supplies every live node as a seed and only actual live parents.
/// This numeric traversal creates no Source, node-identity or root authority.
pub(super) fn acyclic(
    seeds: impl IntoIterator<Item = usize>,
    node_count: usize,
    execution: &ExecutionRequest<'_, '_>,
    control: &Control<'_>,
    mut parent: impl FnMut(usize) -> Result<Option<usize>, Stop>,
) -> Result<(), Stop> {
    let mut colors = Map::default();
    for seed in seeds {
        let mut current = Some(seed);
        while let Some(ordinal) = current {
            match color(&colors, ordinal, control)? {
                Some(Color::Black) => break,
                Some(Color::Gray) => return Err(control.refuse(Stop::SourceStructureCycle)),
                None => {
                    if colors.len() >= node_count {
                        return Err(control.refuse(Stop::SourceStructureIncomplete));
                    }
                    maps::insert(&mut colors, &ordinal, &Color::Gray, execution, control)?;
                    current = parent(ordinal)?;
                }
            }
        }
        // Earlier successful walks are entirely black. Only this walk can
        // be gray, so each newly visited node's parent is read a second time.
        current = Some(seed);
        while let Some(ordinal) = current {
            if color(&colors, ordinal, control)? != Some(Color::Gray) {
                break;
            }
            maps::insert(&mut colors, &ordinal, &Color::Black, execution, control)?;
            current = parent(ordinal)?;
        }
    }
    control.poll()
}
fn color(
    colors: &Map<usize, Color>,
    ordinal: usize,
    control: &Control<'_>,
) -> Result<Option<Color>, Stop> {
    maps::lookup(control, colors.len(), size_of::<usize>())?;
    Ok(colors.get(&ordinal).copied())
}
