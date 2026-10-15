//! Links connect this body's creation to its admitted scope, never arbitrary IDs.
use super::{
    ApplicationOperationBodyAuthoringDenial as Denial, ApplicationOperationBodyCreationBuilder,
};
use crate::application_schema::{
    ApplicationOperationProgramTarget, ApplicationRelationRef, OperationLinks,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationOperationBodyCreationLinkDirection {
    FromScope,
    ToScope,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationOperationBodyCreationLink {
    pub relation: String,
    pub from: String,
    pub to: String,
    pub key: String,
    pub direction: ApplicationOperationBodyCreationLinkDirection,
}
impl ApplicationOperationBodyCreationLink {
    pub fn scope_entity(&self) -> &str {
        match self.direction {
            ApplicationOperationBodyCreationLinkDirection::FromScope => &self.from,
            ApplicationOperationBodyCreationLinkDirection::ToScope => &self.to,
        }
    }
    pub fn created_entity(&self) -> &str {
        match self.direction {
            ApplicationOperationBodyCreationLinkDirection::FromScope => &self.to,
            ApplicationOperationBodyCreationLinkDirection::ToScope => &self.from,
        }
    }
    pub fn program_target(&self) -> ApplicationOperationProgramTarget {
        ApplicationOperationProgramTarget::Link {
            relation: self.relation.clone(),
            from: self.from.clone(),
            to: self.to.clone(),
        }
    }
    pub fn canonical_record(&self) -> String {
        let direction = match self.direction {
            ApplicationOperationBodyCreationLinkDirection::FromScope => "from-scope",
            ApplicationOperationBodyCreationLinkDirection::ToScope => "to-scope",
        };
        let mut record = String::from("creation-link-v1;");
        for part in [&self.relation, &self.from, &self.to, &self.key, direction] {
            use std::fmt::Write;
            write!(record, "{}:{part};", part.len()).expect("String write");
        }
        record
    }
}
impl<Schema, Operation, Input, Entity>
    ApplicationOperationBodyCreationBuilder<Schema, Operation, Input, Entity>
{
    /// Connect the observed scope to this builder's exact created entity type.
    ///
    /// ```compile_fail,E0308
    /// use worth_query_declaration::facade::application_schema::*;
    /// fn wrong_created_endpoint<S, O, I, C, Other, Root, R>(
    ///     creation: ApplicationOperationBodyCreationBuilder<S, O, I, C>,
    ///     relation: ApplicationRelationRef<S, R, Root, Other>,
    /// ) where R: OperationLinks<O> {
    ///     let _ = creation.link_from_scope(relation, "link");
    /// }
    /// ```
    /// Declared Link capability remains a compiler requirement.
    ///
    /// ```compile_fail,E0277
    /// use worth_query_declaration::facade::application_schema::*;
    /// fn missing_link_capability<S, O, I, C, Root, R>(
    ///     creation: ApplicationOperationBodyCreationBuilder<S, O, I, C>,
    ///     relation: ApplicationRelationRef<S, R, Root, C>,
    /// ) {
    ///     let _ = creation.link_from_scope(relation, "link");
    /// }
    /// ```
    pub fn link_from_scope<Relation, Scope>(
        self,
        relation: ApplicationRelationRef<Schema, Relation, Scope, Entity>,
        key: &str,
    ) -> Result<Self, Denial>
    where
        Relation: OperationLinks<Operation>,
    {
        self.link(
            relation.name(),
            relation.from(),
            relation.to(),
            key,
            ApplicationOperationBodyCreationLinkDirection::FromScope,
        )
    }
    pub fn link_to_scope<Relation, Scope>(
        self,
        relation: ApplicationRelationRef<Schema, Relation, Entity, Scope>,
        key: &str,
    ) -> Result<Self, Denial>
    where
        Relation: OperationLinks<Operation>,
    {
        self.link(
            relation.name(),
            relation.from(),
            relation.to(),
            key,
            ApplicationOperationBodyCreationLinkDirection::ToScope,
        )
    }
    fn link(
        mut self,
        relation: &str,
        from: &str,
        to: &str,
        key: &str,
        direction: ApplicationOperationBodyCreationLinkDirection,
    ) -> Result<Self, Denial> {
        if key.is_empty()
            || key.trim() != key
            || self
                .links
                .iter()
                .any(|link| link.relation == relation && link.key == key)
        {
            return Err(Denial::InvalidCreationLink);
        }
        self.links.push(ApplicationOperationBodyCreationLink {
            relation: relation.to_owned(),
            from: from.to_owned(),
            to: to.to_owned(),
            key: key.to_owned(),
            direction,
        });
        Ok(self)
    }
}
