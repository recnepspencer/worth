//! A fresh created entity may link to itself only through a relation admitting self edges.
use super::*;
impl<Schema, Operation, Input, Entity>
    ApplicationOperationBodyCreationBuilder<Schema, Operation, Input, Entity>
{
    /// Links this exact created entity to itself in the same body, such as a
    /// project that watches itself. Both endpoints are the created type and Link
    /// membership is compiler-visible; a relation declaring `self_edges = false`
    /// is refused here, and cold descriptions are independently checked.
    ///
    /// ```compile_fail,E0308
    /// use worth_query_declaration::facade::application_schema::*;
    /// fn foreign_endpoint<S, O, I, C, Other, R>(
    ///     creation: ApplicationOperationBodyCreationBuilder<S, O, I, C>,
    ///     relation: ApplicationRelationRef<S, R, C, Other>,
    /// ) where R: OperationLinks<O> {
    ///     let _ = creation.link_to_self(relation, "self");
    /// }
    /// ```
    ///
    /// ```compile_fail,E0277
    /// use worth_query_declaration::facade::application_schema::*;
    /// fn missing_link_capability<S, O, I, C, R>(
    ///     creation: ApplicationOperationBodyCreationBuilder<S, O, I, C>,
    ///     relation: ApplicationRelationRef<S, R, C, C>,
    /// ) {
    ///     let _ = creation.link_to_self(relation, "self");
    /// }
    /// ```
    pub fn link_to_self<Relation>(
        self,
        relation: ApplicationRelationRef<Schema, Relation, Entity, Entity>,
        key: &str,
    ) -> Result<Self, Denial>
    where
        Relation: OperationLinks<Operation>,
    {
        if !relation.integrity().endpoints.self_edges_allowed {
            return Err(Denial::ForbiddenSelfEdge);
        }
        self.link(
            relation.name(),
            relation.from(),
            relation.to(),
            key,
            ApplicationOperationBodyCreationLinkDirection::SelfEdge,
        )
    }
}
