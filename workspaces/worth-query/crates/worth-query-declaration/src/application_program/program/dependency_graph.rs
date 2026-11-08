use std::collections::BTreeMap;

use super::{
    denial, ApplicationConnectionDeclaration, ApplicationFeatureDeclaration,
    ApplicationProgramValidationDenial, ApplicationProgramValidationDenialKind,
};

pub(super) fn order_connections(
    features: &[ApplicationFeatureDeclaration],
    connections: &[ApplicationConnectionDeclaration],
) -> Result<ApplicationProgramConnectionOrder, ApplicationProgramValidationDenial> {
    let mut incoming = BTreeMap::new();
    let mut ranks = BTreeMap::new();
    let mut outgoing = BTreeMap::<_, Vec<_>>::new();
    for feature in features {
        incoming.insert((feature.composition_instance(), feature.identity()), 0usize);
        ranks.insert((feature.composition_instance(), feature.identity()), 0usize);
    }
    for connection in connections {
        let source = (connection.source_instance(), connection.source_feature());
        let target = (connection.target_instance(), connection.target_feature());
        outgoing.entry(source).or_default().push(target);
        *incoming
            .get_mut(&target)
            .expect("connection target was proven present") += 1;
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
    if visited == features.len() {
        let mut ordered = connections
            .iter()
            .map(|connection| {
                (
                    key(connection),
                    ranks[&(connection.target_instance(), connection.target_feature())],
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
