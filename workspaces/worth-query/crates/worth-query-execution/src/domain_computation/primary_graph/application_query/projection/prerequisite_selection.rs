//! A connection cannot omit a leaf from its declared prerequisite relation path.
use super::*;
use worth_query_declaration::facade::application_query::ApplicationQueryCardinality;
mod sealed {
    pub trait Selection {}
}
pub trait WorthQueryPrerequisiteSelection<S, Q>: sealed::Selection + 'static {
    fn project_all<T: WorthQueryApplicationProjection<S, Q>>(
        &self,
        row: &WorthQueryApplicationProjectionRow<'_, S, Q>,
    ) -> Result<Vec<T>, WorthQueryApplicationProjectionDenial>;
}
impl<Q, Slot, S, R, F, T, D, C> sealed::Selection
    for ApplicationQueryResultRelationRef<Q, Slot, S, R, F, T, D, C>
{
}
impl<Q, Slot, S, R, F, T, D, C> WorthQueryPrerequisiteSelection<S, Q>
    for ApplicationQueryResultRelationRef<Q, Slot, S, R, F, T, D, C>
where
    Q: ApplicationQueryMarkerIdentity<S> + 'static,
    Slot: WorthQueryPortableType + 'static,
    S: worth_query_installation::facade::ApplicationSchema,
    R: 'static,
    F: 'static,
    T: 'static,
    D: ApplicationQueryResultTraversal + 'static,
    C: ApplicationQueryResultRelationCardinality + 'static,
{
    fn project_all<V: WorthQueryApplicationProjection<S, Q>>(
        &self,
        row: &WorthQueryApplicationProjectionRow<'_, S, Q>,
    ) -> Result<Vec<V>, WorthQueryApplicationProjectionDenial> {
        let mut values = Vec::new();
        visit_relation(self, row, |child| {
            values.push(V::project(child)?);
            Ok(())
        })?;
        Ok(values)
    }
}

impl<Q, Slot, S, R, F, T, D, C, Tail> sealed::Selection
    for (
        ApplicationQueryResultRelationRef<Q, Slot, S, R, F, T, D, C>,
        Tail,
    )
where
    Tail: sealed::Selection,
{
}
impl<Q, Slot, S, R, F, T, D, C, Tail> WorthQueryPrerequisiteSelection<S, Q>
    for (
        ApplicationQueryResultRelationRef<Q, Slot, S, R, F, T, D, C>,
        Tail,
    )
where
    Q: ApplicationQueryMarkerIdentity<S> + 'static,
    Slot: WorthQueryPortableType + 'static,
    S: worth_query_installation::facade::ApplicationSchema,
    R: 'static,
    F: 'static,
    T: 'static,
    D: ApplicationQueryResultTraversal + 'static,
    C: ApplicationQueryResultRelationCardinality + 'static,
    Tail: WorthQueryPrerequisiteSelection<S, Q>,
{
    fn project_all<V: WorthQueryApplicationProjection<S, Q>>(
        &self,
        row: &WorthQueryApplicationProjectionRow<'_, S, Q>,
    ) -> Result<Vec<V>, WorthQueryApplicationProjectionDenial> {
        let mut values = Vec::new();
        visit_relation(&self.0, row, |child| {
            values.extend(self.1.project_all::<V>(child)?);
            Ok(())
        })?;
        Ok(values)
    }
}

/// Only this owner visits children; applications supply the terminal projection.
fn visit_relation<Q, Slot, S, R, F, T, D, C>(
    selector: &ApplicationQueryResultRelationRef<Q, Slot, S, R, F, T, D, C>,
    row: &WorthQueryApplicationProjectionRow<'_, S, Q>,
    mut visit: impl FnMut(
        &WorthQueryApplicationProjectionRow<'_, S, Q>,
    ) -> Result<(), WorthQueryApplicationProjectionDenial>,
) -> Result<(), WorthQueryApplicationProjectionDenial>
where
    Q: ApplicationQueryMarkerIdentity<S>,
    Slot: WorthQueryPortableType,
    D: ApplicationQueryResultTraversal,
    C: ApplicationQueryResultRelationCardinality,
{
    let relation = row.relation(selector)?;
    let valid = match C::CARDINALITY {
        ApplicationQueryCardinality::OptionalOne => relation.rows().len() <= 1,
        ApplicationQueryCardinality::ExactlyOne => relation.rows().len() == 1,
        ApplicationQueryCardinality::Many => true,
    };
    if !valid {
        return Err(relation_cardinality_denial(relation));
    }
    for child in relation.rows() {
        visit(&WorthQueryApplicationProjectionRow::new(
            row.node.child(child),
            row.governance,
            row.decode,
        ))?;
    }
    Ok(())
}
