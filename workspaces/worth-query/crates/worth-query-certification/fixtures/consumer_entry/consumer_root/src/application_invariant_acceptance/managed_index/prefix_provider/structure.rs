//! The prefix adapter exposes actual topology; Query supplies structural admission.
use super::*;
use crate::application_invariant_acceptance::managed_index::structure_fault::Fault;

pub(super) fn describe(
    calls: &Calls,
    node: Node,
    control: &WorthQueryManagedIndexControl<'_>,
) -> Result<WorthQueryManagedIndexNodeRegion, Stop> {
    control.checkpoint(1)?;
    let region = WorthQueryManagedIndexNodeRegion::new(Some(node.source), [node.left, node.right]);
    #[cfg(test)]
    {
        let fault = *calls.structure_fault.lock().unwrap();
        Ok(match fault {
            Fault::OmitSource => WorthQueryManagedIndexNodeRegion::new(None, region.children()),
            Fault::DuplicateSource => WorthQueryManagedIndexNodeRegion::new(
                Some(
                    calls
                        .first_source
                        .lock()
                        .unwrap()
                        .expect("the actual build observed its first genuine slot"),
                ),
                region.children(),
            ),
            Fault::DuplicateChild => match region.children().into_iter().flatten().next() {
                Some(child) => WorthQueryManagedIndexNodeRegion::new(
                    region.source(),
                    [Some(child), Some(child)],
                ),
                None => region,
            },
            _ => region,
        })
    }
    #[cfg(not(test))]
    {
        let _ = calls;
        Ok(region)
    }
}

#[cfg(test)]
pub(super) fn cyclic_sources<S: ApplicationSchema, I: PrefixSourceIndex<S>>(
    sources: &WorthQueryManagedIndexSourceRows<'_, S, I>,
    nodes: &mut WorthQueryManagedIndexNodeWriter<'_, Node>,
) -> Result<Option<WorthQueryManagedIndexNodeId>, Stop> {
    let mut root = None;
    let mut cycle = None;
    let mut previous = None;
    for slot in sources.slots() {
        let slot = slot?;
        let id = nodes.push(Node {
            source: slot,
            bounds: worth_query_consumer_values::PrefixSourceBounds::point(slot),
            left: None,
            right: None,
            height: 1,
        })?;
        if root.is_none() {
            root = Some(id);
            continue;
        }
        if cycle.is_none() {
            cycle = Some(id);
        }
        if let Some(previous) = previous {
            let mut node = nodes.get(previous)?;
            node.left = Some(id);
            nodes.replace(previous, node)?;
        }
        previous = Some(id);
    }
    if let (Some(last), Some(first)) = (previous, cycle) {
        let mut node = nodes.get(last)?;
        node.left = Some(first);
        nodes.replace(last, node)?;
    }
    Ok(root)
}
