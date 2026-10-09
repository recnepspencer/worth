use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::proof::SubscriberRepairBatch;

impl SignalGraph {
    pub(super) fn subscriber_repair_check_scope(
        &self,
        sources: &[NodeId],
        plan: &SubscriberRepairBatch,
    ) -> Result<Vec<NodeId>, SignalError> {
        let mut touched = sources.to_vec();
        for repair in plan.as_slice() {
            let current = self.raw_subscribers_of(repair.source)?;
            let desired = repair.subscribers.as_slice();
            let (mut left, mut right) = (0, 0);
            while left < current.len() && right < desired.len() {
                match current[left].cmp(&desired[right]) {
                    std::cmp::Ordering::Less => {
                        touched.push(current[left]);
                        left += 1;
                    }
                    std::cmp::Ordering::Greater => {
                        touched.push(desired[right]);
                        right += 1;
                    }
                    std::cmp::Ordering::Equal => {
                        left += 1;
                        right += 1;
                    }
                }
            }
            touched.extend_from_slice(&current[left..]);
            touched.extend_from_slice(&desired[right..]);
        }
        touched.sort_unstable();
        touched.dedup();
        Ok(touched)
    }
}
