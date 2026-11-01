//! Fixed control work classifies alternatives independently of the requested page width.
use super::*;

impl PageSource<'_> {
    pub(super) fn child_summary(
        &self,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
        parent: WorthQueryHistoryNodeId,
    ) -> Result<(Choice, u64), Failure> {
        // Three index seeks/visits and two exact node lookups are a fixed bound.
        checkpoint(
            context,
            3 * (seek_work(self.children.len()) + 1) + 2 * (seek_work(self.nodes.len()) + 1),
        )?;
        let range = || {
            self.children.range((
                Included((parent.ordinal, 0)),
                Excluded((parent.ordinal, self.completed_end)),
            ))
        };
        let mut children = range();
        let mut first = None;
        let mut count = 0;
        for _ in 0..2 {
            if let Some(observation) = self.observation {
                observation.access(Access::Child);
            }
            let Some(((_, ordinal), _)) = children.next() else {
                break;
            };
            if let Some(observation) = self.observation {
                observation.access(Access::Node);
            }
            let node = self
                .nodes
                .get(ordinal)
                .expect("child index retains its journal node");
            let NodeBinding::Completed(node) = &node.binding else {
                unreachable!("pending reservations are beyond completed_end")
            };
            first.get_or_insert(node.step.id());
            count += 1;
        }
        // Reseek the ORIGINAL parent range; consuming first2 must not lose its end.
        let last = range().next_back();
        if let Some(observation) = self.observation {
            observation.access(Access::Child);
        }
        let end = last.map_or(0, |((_, ordinal), _)| ordinal + 1);
        let choice = match count {
            0 => Choice::NoChild,
            1 => Choice::Single(first.expect("one completed child")),
            _ => Choice::ChoiceRequired,
        };
        Ok((choice, end))
    }
}
