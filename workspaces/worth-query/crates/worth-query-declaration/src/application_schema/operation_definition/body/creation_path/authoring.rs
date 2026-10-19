use super::ApplicationOperationBodyCreationPathLink as PathLink;
use crate::{
    application_query::ApplicationQueryRootPath,
    application_schema::{
        ApplicationOperationBodyAuthoringDenial as Denial, ApplicationOperationBodyCreationBuilder,
        ApplicationOperationBodyCreationLink as Link,
        ApplicationOperationBodyCreationLinkDirection as Direction,
        ApplicationOperationBodyCreationLinkPosture as Posture, ApplicationRelationRef,
        OperationLinks, OperationReads,
    },
};
impl<S, O, I, E> ApplicationOperationBodyCreationBuilder<S, O, I, E> {
    /// Append from one live existing anchor selected by the original-scope path.
    /// Missing or ambiguous anchors deny; callers supply no graph entity ID.
    ///
    /// ```compile_fail,E0277
    /// use worth_query_declaration::facade::{application_query::ApplicationQueryRootPath,application_schema::*};
    /// fn missing_link<S,O,I,E,Root:OperationReads<O>,Anchor:OperationReads<O>,R>(creation:ApplicationOperationBodyCreationBuilder<S,O,I,E>,path:ApplicationQueryRootPath<S,Root,Anchor>,relation:ApplicationRelationRef<S,R,Anchor,E>) {
    ///     let _=creation.link_from_path(path,relation,"anchor");
    /// }
    /// ```
    ///
    /// ```no_run
    /// use worth_query_declaration::facade::{application_query::ApplicationQueryRootPath,application_schema::*};
    /// fn declared_link<S,O,I,E,Root:OperationReads<O>,Anchor:OperationReads<O>,R:OperationLinks<O>>(creation:ApplicationOperationBodyCreationBuilder<S,O,I,E>,path:ApplicationQueryRootPath<S,Root,Anchor>,relation:ApplicationRelationRef<S,R,Anchor,E>) {
    ///     let _=creation.link_from_path(path,relation,"anchor");
    /// }
    /// ```
    pub fn link_from_path<Root, Anchor, R>(
        mut self,
        path: ApplicationQueryRootPath<S, Root, Anchor>,
        relation: ApplicationRelationRef<S, R, Anchor, E>,
        key: &str,
    ) -> Result<Self, Denial>
    where
        Root: OperationReads<O>,
        Anchor: OperationReads<O>,
        R: OperationLinks<O>,
    {
        let path = path.into_meaning();
        self.parent.in_named_scope(path.start_entity())?;
        let link = PathLink {
            path,
            link: Link {
                relation: relation.name().into(),
                from: relation.from().into(),
                to: relation.to().into(),
                key: key.into(),
                direction: Direction::FromScope,
                posture: Posture::Append,
            },
        };
        if !link.has_supported_shape()
            || self
                .links
                .iter()
                .any(|other| other.relation == link.link.relation && other.key == link.link.key)
            || self.path_links.iter().any(|other| {
                other.link.relation == link.link.relation && other.link.key == link.link.key
            })
        {
            return Err(Denial::InvalidCreationLink);
        }
        self.path_links.push(link);
        Ok(self)
    }
}
