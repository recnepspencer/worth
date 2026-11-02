//! Bounded positive facts for one immutable tree's one receiving region.
use super::*;
#[cfg(test)]
mod tests;

pub(super) struct Ancestry {
    region: usize,
    positive: [Option<usize>; 4],
}
impl Ancestry {
    pub(super) fn new(region: usize) -> Self {
        Self {
            region,
            positive: [None; 4],
        }
    }
    pub(super) fn contains(
        &mut self,
        source: usize,
        root: Option<usize>,
        count: usize,
        control: &Control<'_>,
        mut parent: impl FnMut(usize) -> Result<Option<usize>, Stop>,
    ) -> Result<bool, Stop> {
        control.checkpoint(1)?;
        if source == self.region {
            return Ok(true);
        }
        let mut current = source;
        for _ in 0..count {
            control.checkpoint(size_of_val(&self.positive) as u64)?;
            if current == self.region || self.positive.contains(&Some(current)) {
                if self.positive[0] != Some(source) {
                    control.checkpoint(size_of_val(&self.positive) as u64)?;
                    if let Some(position) = self
                        .positive
                        .iter()
                        .position(|entry| *entry == Some(source))
                    {
                        self.positive[..=position].rotate_right(1);
                    } else {
                        self.positive.copy_within(0..3, 1);
                        self.positive[0] = Some(source);
                    }
                }
                return Ok(true);
            }
            match parent(current)? {
                Some(above) => current = above,
                None if root == Some(current) => return Ok(false),
                None => return Err(control.refuse(Stop::SourceStructureIncomplete)),
            }
        }
        Err(control.refuse(Stop::SourceStructureCycle))
    }
}
