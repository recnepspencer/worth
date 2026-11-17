use std::collections::{BTreeMap, BTreeSet};

use super::{
    denial, ApplicationConnectionDeclaration, ApplicationFeatureDeclaration,
    ApplicationProgramValidationDenial, ApplicationProgramValidationDenialKind,
};

pub(super) fn order_connections(
    features: &[ApplicationFeatureDeclaration],
    connections: &[ApplicationConnectionDeclaration],
) -> Result<ApplicationProgramConnectionOrder, ApplicationProgramValidationDenial> {
    let groups = occurrence_groups(features, connections)?;
    let vertices = groups.keys().copied().collect::<Vec<_>>();
    let static_edges = connections
        .iter()
        .filter(|connection| !connection.is_occurrence_component())
        .map(|connection| {
            (
                (connection.source_instance(), connection.source_feature()),
                (connection.target_instance(), connection.target_feature()),
            )
        })
        .collect::<Vec<_>>();
    // Prove every static arc before condensation can hide an internal cycle.
    let static_ranks = topological_ranks(&vertices, &static_edges)?;
    let group_vertices = groups.values().copied().collect::<Vec<_>>();
    let group_edges = static_edges
        .iter()
        .map(|(source, target)| (groups[source], groups[target]))
        .filter(|(source, target)| source != target)
        .collect::<Vec<_>>();
    let group_ranks = topological_ranks(&group_vertices, &group_edges)?;
    let priority = |connection: &ApplicationConnectionDeclaration| {
        let target = (connection.target_instance(), connection.target_feature());
        (group_ranks[&groups[&target]], static_ranks[&target])
    };
    // Compact lexicographic priorities into the existing lookup representation.
    // Actual occurrence prerequisites still decide whether a task is runnable.
    let priorities = connections.iter().map(priority).collect::<BTreeSet<_>>();
    let priorities = priorities
        .into_iter()
        .enumerate()
        .map(|(rank, pair)| (pair, rank))
        .collect::<BTreeMap<_, _>>();
    let mut ordered = connections
        .iter()
        .map(|connection| (key(connection), priorities[&priority(connection)]))
        .collect::<Vec<_>>();
    ordered.sort_unstable_by_key(|entry| entry.0);
    Ok(ApplicationProgramConnectionOrder {
        ranks: ordered.into(),
    })
}

type FeatureKey = (&'static str, &'static str);

fn occurrence_groups(
    features: &[ApplicationFeatureDeclaration],
    connections: &[ApplicationConnectionDeclaration],
) -> Result<BTreeMap<FeatureKey, FeatureKey>, ApplicationProgramValidationDenial> {
    let mut groups = features
        .iter()
        .map(|f| {
            let key = (f.composition_instance(), f.identity());
            (key, key)
        })
        .collect::<BTreeMap<_, _>>();
    for c in connections.iter().filter(|c| c.is_occurrence_component()) {
        let source = groups[&(c.source_instance(), c.source_feature())];
        let target = groups[&(c.target_instance(), c.target_feature())];
        let keep = source.min(target);
        let replace = source.max(target);
        for group in groups.values_mut() {
            if *group == replace {
                *group = keep;
            }
        }
    }
    // One connected occurrence component must carry one complete typed context.
    let mut contexts = BTreeMap::new();
    for connection in connections {
        if let Some(context) = connection.occurrence_component() {
            let group = groups[&(connection.source_instance(), connection.source_feature())];
            if let Some(previous) = contexts.insert(group, context)
                && previous != context {
                    return Err(denial(
                        ApplicationProgramValidationDenialKind::IncompleteOccurrenceComponent,
                        "place overlapping connection groups inside one ApplicationOutputOccurrences",
                    ));
                }
        }
    }
    Ok(groups)
}

fn topological_ranks(
    vertices: &[FeatureKey],
    edges: &[(FeatureKey, FeatureKey)],
) -> Result<BTreeMap<FeatureKey, usize>, ApplicationProgramValidationDenial> {
    let mut incoming = vertices
        .iter()
        .map(|key| (*key, 0usize))
        .collect::<BTreeMap<_, _>>();
    let mut ranks = incoming.clone();
    let mut outgoing = BTreeMap::<_, Vec<_>>::new();
    for &(source, target) in edges {
        outgoing.entry(source).or_default().push(target);
        *incoming.get_mut(&target).expect("validated target") += 1;
    }
    let mut ready: Vec<_> = incoming
        .iter()
        .filter_map(|(feature, count)| (*count == 0).then_some(*feature))
        .collect();
    let mut visited = 0usize;
    while let Some(feature) = ready.pop() {
        visited += 1;
        for target in outgoing.get(&feature).into_iter().flatten() {
            let rank = ranks[&feature] + 1;
            let target_rank = ranks.get_mut(target).expect("declared target");
            *target_rank = (*target_rank).max(rank);
            let count = incoming
                .get_mut(target)
                .expect("connection target was proven present");
            *count -= 1;
            if *count == 0 {
                ready.push(*target);
            }
        }
    }
    if visited == incoming.len() {
        return Ok(ranks);
    }
    Err(denial(
        ApplicationProgramValidationDenialKind::CyclicConnection,
        "application feature dependency graph",
    ))
}

/// A derived scheduling index of the already validated occurrence graph.
/// It carries no permission, producer admission or dependent-set evidence.
#[derive(Clone)]
pub struct ApplicationProgramConnectionOrder {
    ranks: std::sync::Arc<[(ConnectionKey, usize)]>,
}
type ConnectionKey = (&'static str, &'static str, &'static str);
fn key(connection: &ApplicationConnectionDeclaration) -> ConnectionKey {
    (
        connection.source_instance(),
        connection.target_instance(),
        connection.identity(),
    )
}
impl ApplicationProgramConnectionOrder {
    pub fn rank(&self, connection: &ApplicationConnectionDeclaration) -> Option<usize> {
        self.ranks
            .binary_search_by_key(&key(connection), |entry| entry.0)
            .ok()
            .map(|index| self.ranks[index].1)
    }
    /// Upper bound for comparisons in this point lookup, including key widths.
    pub fn lookup_work(&self, connection: &ApplicationConnectionDeclaration) -> Option<u64> {
        let width = connection
            .source_instance()
            .len()
            .checked_add(connection.target_instance().len())?
            .checked_add(connection.identity().len())?
            .checked_add(3)?;
        let levels = (usize::BITS - self.ranks.len().max(1).leading_zeros()) as usize + 1;
        u64::try_from(width.checked_mul(levels)?).ok()
    }
}
