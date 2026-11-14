//! A connection cannot omit a row from its declared prerequisite relation.
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
        let relation = row.relation(self)?;
        let valid = match C::CARDINALITY {
            ApplicationQueryCardinality::OptionalOne => relation.rows().len() <= 1,
            ApplicationQueryCardinality::ExactlyOne => relation.rows().len() == 1,
            ApplicationQueryCardinality::Many => true,
        };
        if !valid {
            return Err(relation_cardinality_denial(relation));
        }
        relation
            .rows()
            .iter()
            .map(|child| {
                V::project(&WorthQueryApplicationProjectionRow::new(
                    row.node.child(child),
                    row.governance,
                    row.decode,
                ))
            })
            .collect()
    }
}
