//! Genuine nested occurrence trees through installed Native preparation.
use super::*;
use worth_foundational::facade::AspectValue;
mod court;
mod fixture;
mod registration;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(super) struct Record {
    path: Vec<RelationId>,
    entity: EntityId,
    value: Option<String>,
}

fn field(
    rows: &ManagedIndexSourceRows<'_>,
    read: &PreparedManagedIndexProjectionRead,
    occurrence: &ManagedIndexSourceProjectionOccurrence,
) -> Result<Option<String>, DerivedRelationReadDenial> {
    rows.with_projected_field(read, occurrence, 0, |value| match value.value() {
        Some(AspectValue::String(worth_foundational::facade::InternedString::Raw(value))) => {
            Some(value.clone())
        }
        None => None,
        other => panic!("actual declared String field: {other:?}"),
    })
}

fn tree(
    rows: &ManagedIndexSourceRows<'_>,
    root: &ManagedIndexSourceOccurrence,
    court: Option<&owner::Court>,
) -> Result<Vec<Record>, DerivedRelationReadDenial> {
    assert!(matches!(
        rows.prepare_projection(root, 1),
        Err(DerivedRelationReadDenial::Definition(
            DerivedRelationDenial::InvalidExpression
        ))
    ));
    let owners = rows.prepare_projection(root, 0)?;
    let mut output = Vec::new();
    let mut previous = None::<(
        EntityId,
        PreparedManagedIndexProjectionRead,
        Vec<ManagedIndexSourceProjectionOccurrence>,
    )>;
    for owner in rows.projection_occurrences(&owners)? {
        output.push(Record {
            path: vec![owner.relation().relation_id],
            entity: owner.entity(),
            value: field(rows, &owners, &owner)?,
        });
        let observed = rows.prepare_nested_projection(&owners, &owner, 1)?;
        assert!(
            !rows
                .projection_support(&observed)?
                .adjacency_dependencies()
                .is_empty(),
            "even empty actual child adjacency remains support"
        );
        let children = rows.projection_occurrences(&observed)?;
        for child in &children {
            let path = vec![owner.relation().relation_id, child.relation().relation_id];
            output.push(Record {
                path: path.clone(),
                entity: child.entity(),
                value: field(rows, &observed, child)?,
            });
            let leaves = rows.prepare_nested_projection(&observed, child, 2)?;
            let occurrences = rows.projection_occurrences(&leaves)?;
            if let Some((anchor, prior, occurrences)) = &previous
                && *anchor == child.entity()
                && let Some(occurrence) = occurrences.first()
            {
                let entered = std::cell::Cell::new(false);
                assert!(matches!(
                    rows.with_projected_field(&leaves, occurrence, 0, |_| entered.set(true)),
                    Err(DerivedRelationReadDenial::Definition(
                        DerivedRelationDenial::ForeignObservation
                    ))
                ));
                assert!(
                    !entered.get(),
                    "convergent-parent lineage cannot graft a child field"
                );
                assert!(rows.projection_support(prior).is_ok());
                if let Some(court) = court {
                    court.nested_checks.fetch_add(1, Ordering::SeqCst);
                }
            }
            for leaf in &occurrences {
                let mut path = path.clone();
                path.push(leaf.relation().relation_id);
                output.push(Record {
                    path,
                    entity: leaf.entity(),
                    value: field(rows, &leaves, leaf)?,
                });
                if let Some(cancel) =
                    court.and_then(|court| court.cancel_nested_field.lock().unwrap().take())
                {
                    rows.with_projected_field(&leaves, leaf, 0, |_| cancel.cancel())?;
                    let entered = std::cell::Cell::new(false);
                    let stopped =
                        rows.with_projected_field(&leaves, leaf, 0, |_| entered.set(true));
                    assert!(
                        !entered.get(),
                        "original cancellation precedes the next nested callback"
                    );
                    return Err(stopped.expect_err("original nested request is cancelled"));
                }
            }
            previous = Some((child.entity(), leaves, occurrences));
        }
    }
    output.sort();
    Ok(output)
}

pub(super) fn observe(
    source: &ManagedIndexSourcePreparation<'_>,
    court: &owner::Court,
) -> Result<(), ManagedIndexSourcePreparationDenial> {
    for scope in source.scopes() {
        if !scope.scope_exists()? {
            continue;
        }
        let rows = scope.proposed();
        let all = rows.acquire_all()?;
        for root in rows.occurrences(&all)? {
            let actual = tree(&rows, &root, Some(court))?;
            court
                .nested
                .lock()
                .unwrap()
                .insert((scope.scope(), root.entity()), actual);
            if scope.committed_scope_exists()? {
                let old = scope.committed()?;
                if let Some(old_root) = old.occurrence(root.entity())? {
                    court
                        .nested_before
                        .lock()
                        .unwrap()
                        .insert((scope.scope(), root.entity()), tree(&old, &old_root, None)?);
                    let parents = rows.prepare_projection(&root, 0)?;
                    if let Some(parent) = rows.projection_occurrences(&parents)?.first() {
                        assert!(matches!(
                            old.prepare_nested_projection(&parents, parent, 1),
                            Err(DerivedRelationReadDenial::Definition(
                                DerivedRelationDenial::ForeignObservation
                            ))
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}
