//! Compile the tracked full-Parent profile from actual installed graph meanings.
use super::Binding;
use crate::application_query::{
    WorthQueryInstalledGraphProjection as Projection,
    WorthQueryInstalledGraphReadContract as Graph, WorthQueryInstalledGraphRelation as Relation,
};
use worth_query_declaration::facade::application_query::{
    ApplicationManagedIndexBindingDescriptor, ApplicationManagedIndexMembership,
    ErasedApplicationQueryDefinition,
};
#[cfg(test)]
mod tests;

/// Installed matching posture. Both retain complete conservative Source dependencies.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryManagedIndexPredicateProfile {
    DirectSourceProjection,
    /// Candidate-related changes can deliver an unchanged full Parent result.
    /// Fresh Parent execution applies the tracked refinement; this is no exact
    /// refined change detector or narrowed influence certificate.
    ConservativeProjectedRefinement,
}

/// A descriptive refusal, never permission to substitute a conservative broadcast.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryManagedIndexPredicateProfileDenial {
    IncompleteMembership,
    UntrackedParameters,
    UntrackedSelection,
    DifferentProjection,
}

pub(super) fn compile(
    recipe: &ApplicationManagedIndexBindingDescriptor,
    source: &Binding,
    parent: &Binding,
) -> Result<WorthQueryManagedIndexPredicateProfile, WorthQueryManagedIndexPredicateProfileDenial> {
    use WorthQueryManagedIndexPredicateProfileDenial as Denial;
    if recipe.meaning().parts().membership == ApplicationManagedIndexMembership::Candidates {
        return Err(Denial::IncompleteMembership);
    }
    let query = parent.query();
    let definition = query.definition();
    let Some(selection) = definition.managed_index() else {
        return Err(Denial::UntrackedSelection);
    };
    if definition.continuation().is_some() || definition.live_cause().is_some() {
        return Err(Denial::UntrackedSelection);
    }
    let source_query = source.query();
    tracked_profile(
        definition,
        &selection.parts().probe_parameter,
        source_query.read_graph(),
        query.read_graph(),
    )
}

fn tracked_profile(
    definition: &ErasedApplicationQueryDefinition,
    probe_parameter: &str,
    source: &Graph,
    parent: &Graph,
) -> Result<WorthQueryManagedIndexPredicateProfile, WorthQueryManagedIndexPredicateProfileDenial> {
    use WorthQueryManagedIndexPredicateProfileDenial as Denial;
    projection_profile(source, parent)?;
    if !definition
        .parameters()
        .iter()
        .any(|p| p.name() == probe_parameter)
        || definition.parameters().iter().any(|p| {
            p.name() != probe_parameter
                && !parent
                    .predicates()
                    .iter()
                    .any(|predicate| predicate.parameter() == p.name())
        })
    {
        return Err(Denial::UntrackedParameters);
    }
    for predicate in parent.predicates() {
        let declared = definition.parameters().iter().any(|p| {
            p.name() == predicate.parameter() && p.scalar_family() == predicate.scalar_family()
        });
        let projected = source.projections().iter().any(|p| {
            p.parent_path() == "root"
                && p.entity() == source.root_entity()
                && (p.entity(), p.aspect(), p.field()) == predicate.field()
                && p.scalar_family() == predicate.scalar_family()
        });
        if !declared || !projected {
            return Err(Denial::UntrackedSelection);
        }
    }
    Ok(if parent.predicates().is_empty() {
        WorthQueryManagedIndexPredicateProfile::DirectSourceProjection
    } else {
        WorthQueryManagedIndexPredicateProfile::ConservativeProjectedRefinement
    })
}

fn projection_profile(
    source: &Graph,
    parent: &Graph,
) -> Result<(), WorthQueryManagedIndexPredicateProfileDenial> {
    use WorthQueryManagedIndexPredicateProfileDenial as Denial;
    if source.component_selection().is_some()
        || source.orbit().is_some()
        || parent.component_selection().is_some()
        || parent.orbit().is_some()
        || !parent.root_paths().is_empty()
        || !source.predicates().is_empty()
        || source
            .relations()
            .iter()
            .chain(parent.relations())
            .any(|r| r.predicate().is_some())
    {
        return Err(Denial::UntrackedSelection);
    }
    // Query and slot-key owners deliberately differ. Compare the compiled
    // physical routes, portable slots and output meaning, never query names.
    if source.schema_basis_digest() != parent.schema_basis_digest()
        || source.root_entity() != parent.root_entity()
        || source.projections().len() != parent.projections().len()
        || source.relations().len() != parent.relations().len()
        || !source
            .projections()
            .iter()
            .zip(parent.projections())
            .all(|(a, b)| same_projection(a, b))
        || !source
            .relations()
            .iter()
            .zip(parent.relations())
            .all(|(a, b)| same_relation(a, b))
    {
        return Err(Denial::DifferentProjection);
    }
    // Ordering was compiled against projected slots. Every continuing member's
    // projected changes will route too, even when candidate membership is stable.
    Ok(())
}

fn same_projection(a: &Projection, b: &Projection) -> bool {
    a.parent_path() == b.parent_path()
        && a.result_path() == b.result_path()
        && a.slot_type() == b.slot_type()
        && a.entity() == b.entity()
        && a.aspect() == b.aspect()
        && a.field() == b.field()
        && a.output_name() == b.output_name()
        && a.scalar_family() == b.scalar_family()
        && a.value_type() == b.value_type()
        && a.presence() == b.presence()
}

fn same_relation(a: &Relation, b: &Relation) -> bool {
    a.parent_path() == b.parent_path()
        && a.result_path() == b.result_path()
        && a.slot_type() == b.slot_type()
        && a.relation() == b.relation()
        && a.from() == b.from()
        && a.to() == b.to()
        && a.direction() == b.direction()
        && a.output_name() == b.output_name()
        && a.cardinality() == b.cardinality()
        && a.depth() == b.depth()
}
