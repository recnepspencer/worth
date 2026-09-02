use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct ModelPortal {
    pub(crate) declaration: u64,
    pub(crate) instance: u64,
    pub(crate) ordinal: u64,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum ModelBackdropScope {
    Surface,
    PerPortalDeclaration(u64),
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum ModelPresence {
    Always,
    WhilePortalPresented(u64),
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum ModelPlacement {
    AboveContent,
    BeforePortal(u64),
    AfterPortal(u64),
    BeforeBackdrop(u64),
    AfterBackdrop(u64),
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct ModelBackdrop {
    pub(crate) declaration: u64,
    pub(crate) scope: ModelBackdropScope,
    pub(crate) presence: ModelPresence,
    pub(crate) placement: ModelPlacement,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum ModelNode {
    Content,
    Portal {
        declaration: u64,
        instance: u64,
    },
    Backdrop {
        declaration: u64,
        instance: Option<u64>,
    },
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum ModelInstanceScope {
    Surface,
    Portal(u64),
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct ModelBackdropInstance {
    declaration: u64,
    node: ModelNode,
    scope: ModelInstanceScope,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ModelDenial {
    MissingPortal,
    AmbiguousPortal,
    MissingBackdrop,
    AmbiguousBackdrop,
    CrossScopeBackdrop,
    AmbiguousOrder,
    Cycle,
}

pub(crate) fn model_order(
    source_portals: &[ModelPortal],
    declarations: &[ModelBackdrop],
) -> Result<Vec<ModelNode>, ModelDenial> {
    let mut portals = source_portals.to_vec();
    portals.sort_by_key(|portal| portal.ordinal);
    if portals
        .windows(2)
        .any(|window| window[0].ordinal >= window[1].ordinal)
    {
        return Err(ModelDenial::Cycle);
    }
    let mut portal_nodes = BTreeSet::new();
    for portal in &portals {
        if !portal_nodes.insert(ModelNode::Portal {
            declaration: portal.declaration,
            instance: portal.instance,
        }) {
            return Err(ModelDenial::Cycle);
        }
    }
    let instances = materialize_backdrops(&portals, declarations);
    let mut nodes = BTreeSet::from([ModelNode::Content]);
    nodes.extend(portal_nodes);
    nodes.extend(instances.iter().map(|instance| instance.node));
    let mut edges = BTreeSet::new();
    if let Some(first) = portals.first() {
        add_edge(&mut edges, ModelNode::Content, portal_node(*first))?;
    }
    for window in portals.windows(2) {
        add_edge(&mut edges, portal_node(window[0]), portal_node(window[1]))?;
    }
    let mut immediate = BTreeSet::new();
    for declaration in declarations {
        for instance in instances
            .iter()
            .filter(|instance| instance.declaration == declaration.declaration)
        {
            let (lower, upper, is_immediate) =
                placement_edge(declaration.placement, instance, &portals, &instances)?;
            add_edge(&mut edges, lower, upper)?;
            if is_immediate {
                immediate.insert((lower, upper));
            }
        }
    }
    let mut results = Vec::new();
    let mut path = Vec::new();
    let mut remaining = nodes.iter().copied().collect::<Vec<_>>();
    search(&mut path, &mut remaining, &edges, &immediate, &mut results);
    match results.as_slice() {
        [order] => Ok(order.clone()),
        [] => Err(ModelDenial::Cycle),
        _ => Err(ModelDenial::AmbiguousOrder),
    }
}

fn materialize_backdrops(
    portals: &[ModelPortal],
    declarations: &[ModelBackdrop],
) -> Vec<ModelBackdropInstance> {
    declarations
        .iter()
        .filter(|declaration| present(declaration.presence, portals))
        .flat_map(|declaration| match declaration.scope {
            ModelBackdropScope::Surface => vec![ModelBackdropInstance {
                declaration: declaration.declaration,
                node: ModelNode::Backdrop {
                    declaration: declaration.declaration,
                    instance: None,
                },
                scope: ModelInstanceScope::Surface,
            }],
            ModelBackdropScope::PerPortalDeclaration(target) => portals
                .iter()
                .filter(|portal| portal.declaration == target)
                .map(|portal| ModelBackdropInstance {
                    declaration: declaration.declaration,
                    node: ModelNode::Backdrop {
                        declaration: declaration.declaration,
                        instance: Some(portal.instance),
                    },
                    scope: ModelInstanceScope::Portal(portal.instance),
                })
                .collect(),
        })
        .collect()
}

fn present(presence: ModelPresence, portals: &[ModelPortal]) -> bool {
    match presence {
        ModelPresence::Always => true,
        ModelPresence::WhilePortalPresented(declaration) => portals
            .iter()
            .any(|portal| portal.declaration == declaration),
    }
}

fn placement_edge(
    placement: ModelPlacement,
    source: &ModelBackdropInstance,
    portals: &[ModelPortal],
    instances: &[ModelBackdropInstance],
) -> Result<(ModelNode, ModelNode, bool), ModelDenial> {
    match placement {
        ModelPlacement::AboveContent => Ok((ModelNode::Content, source.node, false)),
        ModelPlacement::BeforePortal(target) => Ok((
            source.node,
            portal_anchor(source.scope, target, portals)?,
            true,
        )),
        ModelPlacement::AfterPortal(target) => Ok((
            portal_anchor(source.scope, target, portals)?,
            source.node,
            true,
        )),
        ModelPlacement::BeforeBackdrop(target) => Ok((
            source.node,
            backdrop_anchor(source.scope, target, instances)?,
            true,
        )),
        ModelPlacement::AfterBackdrop(target) => Ok((
            backdrop_anchor(source.scope, target, instances)?,
            source.node,
            true,
        )),
    }
}

fn portal_anchor(
    scope: ModelInstanceScope,
    declaration: u64,
    portals: &[ModelPortal],
) -> Result<ModelNode, ModelDenial> {
    let mut candidates = portals
        .iter()
        .filter(|portal| portal.declaration == declaration);
    match scope {
        ModelInstanceScope::Portal(instance) => candidates
            .find(|portal| portal.instance == instance)
            .map(|portal| portal_node(*portal))
            .ok_or(ModelDenial::MissingPortal),
        ModelInstanceScope::Surface => match candidates.collect::<Vec<_>>().as_slice() {
            [] => Err(ModelDenial::MissingPortal),
            [portal] => Ok(portal_node(**portal)),
            _ => Err(ModelDenial::AmbiguousPortal),
        },
    }
}

fn backdrop_anchor(
    scope: ModelInstanceScope,
    declaration: u64,
    instances: &[ModelBackdropInstance],
) -> Result<ModelNode, ModelDenial> {
    let candidates = instances
        .iter()
        .filter(|instance| instance.declaration == declaration)
        .copied()
        .collect::<Vec<_>>();
    match scope {
        ModelInstanceScope::Surface => match candidates.as_slice() {
            [] => Err(ModelDenial::MissingBackdrop),
            [instance] => Ok(instance.node),
            _ => Err(ModelDenial::AmbiguousBackdrop),
        },
        ModelInstanceScope::Portal(source_instance) => candidates
            .iter()
            .find(|instance| instance.scope == ModelInstanceScope::Portal(source_instance))
            .map(|instance| instance.node)
            .ok_or(if candidates.is_empty() {
                ModelDenial::MissingBackdrop
            } else {
                ModelDenial::CrossScopeBackdrop
            }),
    }
}

fn add_edge(
    edges: &mut BTreeSet<(ModelNode, ModelNode)>,
    lower: ModelNode,
    upper: ModelNode,
) -> Result<(), ModelDenial> {
    if lower == upper {
        return Err(ModelDenial::Cycle);
    }
    edges.insert((lower, upper));
    Ok(())
}

fn portal_node(portal: ModelPortal) -> ModelNode {
    ModelNode::Portal {
        declaration: portal.declaration,
        instance: portal.instance,
    }
}

fn search(
    path: &mut Vec<ModelNode>,
    remaining: &mut Vec<ModelNode>,
    edges: &BTreeSet<(ModelNode, ModelNode)>,
    immediate: &BTreeSet<(ModelNode, ModelNode)>,
    results: &mut Vec<Vec<ModelNode>>,
) {
    if results.len() >= 2 {
        return;
    }
    if remaining.is_empty() {
        results.push(path.clone());
        return;
    }
    for candidate in remaining.clone() {
        if !eligible(candidate, path, edges, immediate) {
            continue;
        }
        let position = remaining
            .iter()
            .position(|node| *node == candidate)
            .expect("candidate remains available");
        remaining.remove(position);
        path.push(candidate);
        search(path, remaining, edges, immediate, results);
        path.pop();
        remaining.insert(position, candidate);
    }
}

fn eligible(
    candidate: ModelNode,
    path: &[ModelNode],
    edges: &BTreeSet<(ModelNode, ModelNode)>,
    immediate: &BTreeSet<(ModelNode, ModelNode)>,
) -> bool {
    if edges
        .iter()
        .any(|(lower, upper)| *upper == candidate && !path.contains(lower))
    {
        return false;
    }
    if immediate
        .iter()
        .find(|(_, upper)| *upper == candidate)
        .is_some_and(|(lower, _)| path.last() != Some(lower))
    {
        return false;
    }
    if path.last().is_some_and(|previous| {
        immediate
            .iter()
            .find(|(lower, _)| lower == previous)
            .is_some_and(|(_, upper)| upper != &candidate)
    }) {
        return false;
    }
    true
}
