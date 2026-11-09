//! Complete native collections prepare the same actual member/edge footprint.
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::probe::Entity;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::probe::Rule;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::schema::OrbitSchema;
use worth_query_execution::facade::application_invariants::WorthQueryApplicationInvariantReadView;
use worth_query_execution::facade::application_invariants::WorthQueryInvariantAccessDenial;
pub fn nodes<const O: bool>(
    view: &WorthQueryApplicationInvariantReadView<'_, OrbitSchema<O>>,
    rule: &Rule<O>,
) -> Result<Vec<(u64, Entity<O>)>, WorthQueryInvariantAccessDenial> {
    let components = view.components(&rule.shells)?;
    let mut nodes = Vec::new();
    for component in components.iter() {
        for entity in component.members() {
            let key = view.field(&rule.key, &entity)?.expect("authored key");
            nodes.push((key, entity));
        }
    }
    // Even an empty real member kind is read in preparation and evaluation.
    view.components(&rule.spares)?;
    Ok(nodes)
}
