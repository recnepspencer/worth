//! Derived numeric ancestry for one borrowed immutable tree, never a Source proof.
use super::*;
use std::sync::Arc;
mod ready;
use ready::Ready;
#[cfg(test)]
mod tests;

#[derive(Clone)]
enum Entry {
    Pending {
        parent: Option<usize>,
        child: Option<usize>,
    },
    Ready(Arc<Ready>),
}
#[derive(Default)]
pub(super) struct Ancestry {
    nodes: Map<usize, Entry>,
}
impl Ancestry {
    pub(super) fn contains(
        &mut self,
        region: usize,
        source: usize,
        root: Option<usize>,
        count: usize,
        execution: &ExecutionRequest<'_, '_>,
        control: &Control<'_>,
        mut parent: impl FnMut(usize) -> Result<Option<usize>, Stop>,
    ) -> Result<bool, Stop> {
        control.checkpoint(1)?;
        if source == region {
            return Ok(true);
        }
        self.ensure(source, root, count, execution, control, &mut parent)?;
        // Ready is ancestor-closed. Only AFTER ensure is absence proof that
        // this receiving region cannot lie on this Source's actual parent path.
        let Some(ancestor) = self.ready(region, control)? else {
            return Ok(false);
        };
        let mut current = self
            .ready(source, control)?
            .ok_or_else(|| control.refuse(Stop::IncompleteTraversal))?;
        let Some(mut difference) = current.depth.checked_sub(ancestor.depth) else {
            return Ok(false);
        };
        let mut ordinal = source;
        while difference != 0 {
            control.checkpoint(1)?;
            let bit = difference.trailing_zeros() as usize;
            ordinal = current.jump(bit, control)?;
            difference &= difference - 1;
            current = self
                .ready(ordinal, control)?
                .ok_or_else(|| control.refuse(Stop::IncompleteTraversal))?;
        }
        Ok(ordinal == region)
    }

    fn ensure(
        &mut self,
        source: usize,
        root: Option<usize>,
        count: usize,
        execution: &ExecutionRequest<'_, '_>,
        control: &Control<'_>,
        parent: &mut impl FnMut(usize) -> Result<Option<usize>, Stop>,
    ) -> Result<(), Stop> {
        let (mut ordinal, mut child, mut visited) = (source, None, 0usize);
        let mut unwind = loop {
            match self.entry(ordinal, control)? {
                Some(Entry::Ready(_)) => break child,
                Some(Entry::Pending { .. }) => {
                    return Err(control.refuse(Stop::SourceStructureCycle))
                }
                None => {}
            }
            if visited >= count {
                return Err(control.refuse(Stop::SourceStructureCycle));
            }
            visited += 1;
            let above = parent(ordinal)?;
            if (above.is_none()) != (root == Some(ordinal)) {
                return Err(control.refuse(Stop::SourceStructureIncomplete));
            }
            maps::insert(
                &mut self.nodes,
                &ordinal,
                &Entry::Pending {
                    parent: above,
                    child,
                },
                execution,
                control,
            )?;
            match above {
                Some(above) => {
                    child = Some(ordinal);
                    ordinal = above;
                }
                None => break Some(ordinal),
            }
        };
        while let Some(ordinal) = unwind {
            let Some(Entry::Pending { parent, child }) = self.entry(ordinal, control)? else {
                return Err(control.refuse(Stop::IncompleteTraversal));
            };
            let parent = parent
                .map(|parent| {
                    self.ready(parent, control)?
                        .map(|ready| (parent, ready))
                        .ok_or_else(|| control.refuse(Stop::IncompleteTraversal))
                })
                .transpose()?;
            let ready = Ready::derive(parent, execution, control, |ancestor| {
                self.ready(ancestor, control)?
                    .ok_or_else(|| control.refuse(Stop::IncompleteTraversal))
            })?;
            maps::insert(
                &mut self.nodes,
                &ordinal,
                &Entry::Ready(ready),
                execution,
                control,
            )?;
            unwind = child;
        }
        Ok(())
    }
    fn entry(&self, ordinal: usize, control: &Control<'_>) -> Result<Option<Entry>, Stop> {
        maps::lookup(control, self.nodes.len(), size_of::<usize>())?;
        Ok(self.nodes.get(&ordinal).cloned())
    }
    fn ready(&self, ordinal: usize, control: &Control<'_>) -> Result<Option<Arc<Ready>>, Stop> {
        match self.entry(ordinal, control)? {
            Some(Entry::Ready(ready)) => Ok(Some(ready)),
            Some(Entry::Pending { .. }) => Err(control.refuse(Stop::IncompleteTraversal)),
            None => Ok(None),
        }
    }
}
