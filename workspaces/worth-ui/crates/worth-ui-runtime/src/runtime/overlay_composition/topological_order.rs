use std::collections::{BTreeMap, BTreeSet};

use super::order::OrderNode;
use super::planner::UiOverlayCompositionDenial;

pub(super) fn unique_topological_order(
    nodes: &BTreeSet<OrderNode>,
    edges: &BTreeSet<(OrderNode, OrderNode)>,
    immediate: &BTreeSet<(OrderNode, OrderNode)>,
) -> Result<Vec<OrderNode>, UiOverlayCompositionDenial> {
    let mut predecessors = BTreeMap::new();
    let mut successors = BTreeMap::new();
    for (lower, upper) in immediate {
        if lower == upper || !nodes.contains(lower) || !nodes.contains(upper) {
            return Err(UiOverlayCompositionDenial::Cycle);
        }
        if successors.insert(*lower, *upper).is_some()
            || predecessors.insert(*upper, *lower).is_some()
        {
            return Err(UiOverlayCompositionDenial::ImmediateAdjacencyViolated);
        }
    }
    let mut component_for = BTreeMap::new();
    let mut components = BTreeMap::<OrderNode, Vec<OrderNode>>::new();
    for start in nodes
        .iter()
        .copied()
        .filter(|node| !predecessors.contains_key(node))
    {
        let mut chain = Vec::new();
        let mut current = start;
        loop {
            if component_for.insert(current, start).is_some() {
                return Err(UiOverlayCompositionDenial::Cycle);
            }
            chain.push(current);
            let Some(next) = successors.get(&current).copied() else {
                break;
            };
            current = next;
        }
        components.insert(start, chain);
    }
    if component_for.len() != nodes.len() {
        return Err(UiOverlayCompositionDenial::Cycle);
    }

    let mut positions = BTreeMap::new();
    for (component, chain) in &components {
        for (position, node) in chain.iter().copied().enumerate() {
            positions.insert(node, (*component, position));
        }
    }
    let mut component_edges = BTreeSet::new();
    for (lower, upper) in edges {
        let lower_component = component_for
            .get(lower)
            .copied()
            .ok_or(UiOverlayCompositionDenial::Cycle)?;
        let upper_component = component_for
            .get(upper)
            .copied()
            .ok_or(UiOverlayCompositionDenial::Cycle)?;
        if lower_component == upper_component {
            if positions[lower].1 >= positions[upper].1 {
                return Err(UiOverlayCompositionDenial::Cycle);
            }
        } else {
            component_edges.insert((lower_component, upper_component));
        }
    }
    let component_order = unique_component_order(&components, &component_edges)?;
    let mut order = Vec::with_capacity(nodes.len());
    for component in component_order {
        order.extend(
            components
                .get(&component)
                .expect("ordered component remains materialized")
                .iter()
                .copied(),
        );
    }
    Ok(order)
}

fn unique_component_order(
    components: &BTreeMap<OrderNode, Vec<OrderNode>>,
    edges: &BTreeSet<(OrderNode, OrderNode)>,
) -> Result<Vec<OrderNode>, UiOverlayCompositionDenial> {
    let mut incoming = components
        .keys()
        .copied()
        .map(|component| (component, 0_usize))
        .collect::<BTreeMap<_, _>>();
    let mut outgoing = BTreeMap::<OrderNode, Vec<OrderNode>>::new();
    for (lower, upper) in edges {
        *incoming
            .get_mut(upper)
            .ok_or(UiOverlayCompositionDenial::Cycle)? += 1;
        outgoing.entry(*lower).or_default().push(*upper);
    }
    let mut selected = BTreeSet::new();
    let mut order = Vec::with_capacity(components.len());
    while order.len() < components.len() {
        let available = incoming
            .iter()
            .filter_map(|(component, count)| {
                (*count == 0 && !selected.contains(component)).then_some(*component)
            })
            .collect::<Vec<_>>();
        let Some(component) = available.first().copied() else {
            return Err(UiOverlayCompositionDenial::Cycle);
        };
        if available.len() != 1 {
            return Err(UiOverlayCompositionDenial::AmbiguousOrder);
        }
        selected.insert(component);
        order.push(component);
        for successor in outgoing.get(&component).into_iter().flatten() {
            *incoming
                .get_mut(successor)
                .expect("edge has incoming entry") -= 1;
        }
    }
    Ok(order)
}
