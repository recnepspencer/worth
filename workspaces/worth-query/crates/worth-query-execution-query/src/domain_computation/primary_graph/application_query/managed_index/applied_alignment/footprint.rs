//! Everything a managed Parent reads beside its Source program, by installed
//! kind, aspect and field: the alignment proof must cover all of it.
use super::Denial;
use crate::domain_computation::primary_graph::schema_layout::WorthQueryPrimaryGraphLayout;
use worth_query_declaration::facade::application_query::ApplicationQueryBindingReference;
use worth_query_installation::facade::{
    WorthQueryInstalledGraphReadContract, WorthQueryInstalledGraphRelationPredicate as Predicate,
};
use worth_relational::facade::mvcc::ManagedIndexParentReadFootprint;

/// The Parent's projected, filtered and ordered fields, its nested result
/// relations, their endpoint kinds and filters, and its scope-key field. A
/// read the installed layout cannot name refuses rather than shrinking it.
pub(super) fn parent_footprint(
    layout: &WorthQueryPrimaryGraphLayout,
    reads: &WorthQueryInstalledGraphReadContract,
    parent: &ApplicationQueryBindingReference,
) -> Result<ManagedIndexParentReadFootprint, Denial> {
    // Managed Parents cannot declare a where-filter or related predicate today;
    // one that could would read fields or relations this footprint does not name.
    if reads.where_filter().is_some() || !reads.related_predicates().is_empty() {
        return Err(Denial::ForeignAssociation);
    }
    let mut footprint = ManagedIndexParentReadFootprint::default();
    let mut field = |footprint: &mut ManagedIndexParentReadFootprint,
                     (entity, aspect, field): (&str, &str, &str),
                     lifecycle: bool| {
        let installed = layout
            .field(entity, aspect, field)
            .ok_or(Denial::ForeignAssociation)?;
        let key = installed
            .locator
            .field_path()
            .fields()
            .first()
            .cloned()
            .ok_or(Denial::ForeignAssociation)?;
        footprint.fields.push((
            installed.entity_kind,
            installed.locator.aspect().aspect_key().clone(),
            key,
        ));
        if lifecycle {
            footprint.entities.push(installed.entity_kind);
        }
        Ok::<_, Denial>(())
    };
    for projection in reads.projections() {
        field(
            &mut footprint,
            (projection.entity(), projection.aspect(), projection.field()),
            true,
        )?;
    }
    for predicate in reads.predicates() {
        field(&mut footprint, predicate.field(), true)?;
    }
    for ordering in reads.ordering() {
        field(&mut footprint, ordering.field(), true)?;
    }
    for relation in reads.relations() {
        let installed = layout
            .relation(relation.relation())
            .ok_or(Denial::ForeignAssociation)?;
        footprint.relations.push(installed.kind);
        footprint.entities.extend([installed.from, installed.to]);
        match relation.predicate() {
            None | Some(Predicate::EntityEqualParameter { .. }) => {}
            Some(Predicate::EqualParameter(equal)) => field(&mut footprint, equal.field(), true)?,
            Some(Predicate::FieldAbsent {
                entity,
                aspect,
                field,
                ..
            }) => {
                if entity != relation.child_entity() {
                    return Err(Denial::ForeignAssociation);
                }
                let child = child_kind(relation.from(), relation.child_entity(), installed);
                footprint
                    .fields
                    .push((child, aspect.clone(), field.clone()));
                footprint.entities.push(child);
            }
        }
    }
    // The key that resolved the probed scope at the before basis: a write
    // to it on any scope entity could name another scope in the candidate.
    let parts = parent.parts();
    field(
        &mut footprint,
        (&parts.scope_entity, &parts.scope_aspect, &parts.scope_field),
        false,
    )?;
    // The field just recorded is the scope key itself.
    footprint.scope_key = footprint.fields.last().cloned();
    Ok(footprint)
}

/// The kind a nested relation's filter reads: its child endpoint, which is
/// the relation's `from` when the traversal is reversed.
fn child_kind(
    from: &str,
    child: &str,
    installed: &crate::domain_computation::primary_graph::schema_layout::WorthQueryPrimaryRelationLayout,
) -> worth_relational::facade::identity::KindId {
    if child == from {
        installed.from
    } else {
        installed.to
    }
}

#[cfg(test)]
mod tests {
    use super::child_kind;
    use crate::domain_computation::primary_graph::schema_layout::WorthQueryPrimaryRelationLayout;
    use worth_relational::facade::identity::KindId;

    #[test]
    fn a_reversed_relation_filters_its_from_endpoint() {
        let contains = WorthQueryPrimaryRelationLayout {
            kind: KindId(9),
            from: KindId(1),
            to: KindId(2),
            source_max: None,
            orbit: false,
        };
        // Forward Directory -Contains-> Item filters the Item; reversed from
        // Item, the child is the Directory.
        assert_eq!(child_kind("Directory", "Item", &contains), KindId(2));
        assert_eq!(child_kind("Directory", "Directory", &contains), KindId(1));
    }
}
