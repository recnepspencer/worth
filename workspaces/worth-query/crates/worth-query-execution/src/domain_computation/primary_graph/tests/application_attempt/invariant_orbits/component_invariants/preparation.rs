//! Complete native collections prepare the same actual member/edge footprint.
use super::super::{
    probe::{Entity, Rule},
    schema::OrbitSchema,
};
use crate::facade::application_invariants::*;
pub(in super::super) fn nodes<const O: bool>(
    view: &WorthQueryApplicationInvariantReadView<'_, OrbitSchema<O>>,
    rule: &Rule<O>,
) -> Result<Vec<(u64, Entity<O>)>, WorthQueryApplicationInvariantPreparationError> {
    let failure = |e: WorthQueryInvariantAccessDenial| {
        WorthQueryApplicationInvariantPreparationError::new(e.to_string())
    };
    let components = view.components(&rule.shells).map_err(failure)?;
    let mut nodes = Vec::new();
    for component in components.iter() {
        for entity in component.members() {
            let key = view
                .field(&rule.key, &entity)
                .map_err(failure)?
                .expect("authored key");
            nodes.push((key, entity));
        }
    }
    // Even an empty real member kind is read in preparation and evaluation.
    view.components(&rule.spares).map_err(failure)?;
    Ok(nodes)
}
