use std::collections::{BTreeMap, BTreeSet};

use super::order::OrderNode;
use super::planner::UiOverlayCompositionDenial;

pub(super) fn unique_topological_order(
    nodes: &BTreeSet<OrderNode>,
    edges: &BTreeSet<(OrderNode, OrderNode)>,
) -> Result<Vec<OrderNode>, UiOverlayCompositionDenial> {
    let mut incoming = nodes
        .iter()
        .copied()
        .map(|node| (node, 0_usize))
        .collect::<BTreeMap<_, _>>();
    let mut outgoing = BTreeMap::<OrderNode, Vec<OrderNode>>::new();
    for (lower, upper) in edges {
        *incoming
            .get_mut(upper)
            .ok_or(UiOverlayCompositionDenial::Cycle)? += 1;
        outgoing.entry(*lower).or_default().push(*upper);
    }
    let mut selected = BTreeSet::new();
    let mut order = Vec::with_capacity(nodes.len());
    while order.len() < nodes.len() {
        let available = incoming
            .iter()
            .filter_map(|(node, count)| (*count == 0 && !selected.contains(node)).then_some(*node))
            .collect::<Vec<_>>();
        let Some(node) = available.first().copied() else {
            return Err(UiOverlayCompositionDenial::Cycle);
        };
        if available.len() != 1 {
            return Err(UiOverlayCompositionDenial::AmbiguousOrder);
        }
        selected.insert(node);
        order.push(node);
        for successor in outgoing.get(&node).into_iter().flatten() {
            *incoming
                .get_mut(successor)
                .expect("edge has incoming entry") -= 1;
        }
    }
    Ok(order)
}
