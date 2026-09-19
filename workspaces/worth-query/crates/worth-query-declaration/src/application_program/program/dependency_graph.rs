use std::collections::BTreeMap;

use super::{
    denial, ApplicationConnectionDeclaration, ApplicationFeatureDeclaration,
    ApplicationProgramValidationDenial, ApplicationProgramValidationDenialKind,
};

pub(super) fn require_acyclic_connections(
    features: &[ApplicationFeatureDeclaration],
    connections: &[ApplicationConnectionDeclaration],
) -> Result<(), ApplicationProgramValidationDenial> {
    let mut incoming = BTreeMap::new();
    let mut outgoing = BTreeMap::<_, Vec<_>>::new();
    for feature in features {
        incoming.insert((feature.composition_instance(), feature.identity()), 0usize);
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
        return Ok(());
    }
    Err(denial(
        ApplicationProgramValidationDenialKind::CyclicConnection,
        "application feature dependency graph",
    ))
}
