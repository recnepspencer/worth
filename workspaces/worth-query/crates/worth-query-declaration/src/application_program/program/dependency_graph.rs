use std::collections::BTreeMap;

use super::{
    denial, ApplicationConnectionDeclaration, ApplicationFeatureDeclaration,
    ApplicationProgramValidationDenial, ApplicationProgramValidationDenialKind,
};

pub(super) fn order_connections(
    features: &[ApplicationFeatureDeclaration],
    connections: &[ApplicationConnectionDeclaration],
) -> Result<ApplicationProgramConnectionOrder, ApplicationProgramValidationDenial> {
    // Only declaration-owned explicit occurrence components are condensed.
    // Static edges and every edge crossing a component remain ordinary DAG arcs.
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
            if let Some(previous) = contexts.insert(group, context) {
                if previous != context {
                    return Err(denial(
                        ApplicationProgramValidationDenialKind::IncompleteOccurrenceComponent,
                        "place overlapping connection groups inside one ApplicationOutputOccurrences",
                    ));
                }
            }
        }
    }
    let mut incoming = BTreeMap::new();
    let mut ranks = BTreeMap::new();
    let mut outgoing = BTreeMap::<_, Vec<_>>::new();
    for group in groups.values() {
        incoming.insert(*group, 0usize);
        ranks.insert(*group, 0usize);
    }
    for connection in connections {
        let source = groups[&(connection.source_instance(), connection.source_feature())];
        let target = groups[&(connection.target_instance(), connection.target_feature())];
        if connection.is_occurrence_component() {
            continue;
        }
        if source == target {
            return Err(denial(
                ApplicationProgramValidationDenialKind::CyclicConnection,
                "static dependency inside an occurrence component",
            ));
        }
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
        let mut ordered = connections
            .iter()
            .map(|connection| {
                (
                    key(connection),
                    ranks[&groups[&(connection.target_instance(), connection.target_feature())]],
                )
            })
            .collect::<Vec<_>>();
        ordered.sort_unstable_by_key(|entry| entry.0);
        return Ok(ApplicationProgramConnectionOrder {
            ranks: ordered.into(),
        });
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
