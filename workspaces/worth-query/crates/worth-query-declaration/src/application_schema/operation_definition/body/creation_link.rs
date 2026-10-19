//! Links connect this body's creation to its admitted scope, never arbitrary IDs.
use super::{
    ApplicationOperationBodyAuthoringDenial as Denial, ApplicationOperationBodyCreationBuilder,
};
use crate::application_schema::{
    ApplicationOperationDecisionReadTarget, ApplicationOperationProgramTarget,
    ApplicationRelationRef, OperationLinks, OperationReads, OperationUnlinks,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationOperationBodyCreationLinkDirection {
    FromScope,
    ToScope,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationOperationBodyCreationLinkPosture {
    Append,
    ReplaceSingle,
    ReplaceAndRetireSingle,
}
impl ApplicationOperationBodyCreationLinkPosture {
    pub fn replaces_selection(self) -> bool {
        self != Self::Append
    }
    pub fn retires_predecessor(self) -> bool {
        self == Self::ReplaceAndRetireSingle
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationOperationBodyCreationLink {
    pub relation: String,
    pub from: String,
    pub to: String,
    pub key: String,
    pub direction: ApplicationOperationBodyCreationLinkDirection,
    pub posture: ApplicationOperationBodyCreationLinkPosture,
}
impl ApplicationOperationBodyCreationLink {
    pub fn replacement_target(&self) -> Option<ApplicationOperationProgramTarget> {
        self.posture
            .replaces_selection()
            .then(|| ApplicationOperationProgramTarget::Unlink {
                relation: self.relation.clone(),
                from: self.from.clone(),
                to: self.to.clone(),
            })
    }
    pub fn retirement_target(&self) -> Option<ApplicationOperationProgramTarget> {
        self.posture
            .retires_predecessor()
            .then(|| ApplicationOperationProgramTarget::Delete {
                entity: self.created_entity().into(),
            })
    }
    pub fn replacement_reads(&self) -> Vec<ApplicationOperationDecisionReadTarget> {
        if !self.posture.replaces_selection() {
            return Vec::new();
        }
        vec![
            ApplicationOperationDecisionReadTarget::Relation {
                relation: self.relation.clone(),
                from: self.from.clone(),
                to: self.to.clone(),
            },
            ApplicationOperationDecisionReadTarget::Entity {
                entity: self.from.clone(),
            },
            ApplicationOperationDecisionReadTarget::Entity {
                entity: self.to.clone(),
            },
        ]
    }
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
        let mut record = String::from(match self.posture {
            ApplicationOperationBodyCreationLinkPosture::Append => "creation-link-v1;",
            ApplicationOperationBodyCreationLinkPosture::ReplaceSingle => {
                "creation-link-replace-single-v1;"
            }
            ApplicationOperationBodyCreationLinkPosture::ReplaceAndRetireSingle => {
                "creation-link-replace-and-retire-single-v1;"
            }
        });
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
    /// Atomically replaces an at-most-one outgoing selection with this exact creation.
    /// The installed edge contract, not a distinct-target count, bounds retirement.
    ///
    /// Unlink permission is mandatory even when the first observed selection is empty.
    ///
    /// ```compile_fail,E0277
    /// use worth_query_declaration::facade::application_schema::*;
    /// fn missing_unlink<S, O, I, E, Root, R>(
    ///     creation: ApplicationOperationBodyCreationBuilder<S, O, I, E>,
    ///     relation: ApplicationRelationRef<S, R, Root, E>,
    /// ) where R: OperationReads<O> + OperationLinks<O> {
    ///     let _ = creation.replace_from_scope(relation, "current");
    /// }
    /// ```
    ///
    /// ```compile_fail,E0308
    /// use worth_query_declaration::facade::application_schema::*;
    /// fn wrong_endpoint<S, O, I, E, Other, Root, R>(
    ///     creation: ApplicationOperationBodyCreationBuilder<S, O, I, E>,
    ///     relation: ApplicationRelationRef<S, R, Root, Other>,
    /// ) where R: OperationReads<O> + OperationLinks<O> + OperationUnlinks<O> {
    ///     let _ = creation.replace_from_scope(relation, "current");
    /// }
    /// ```
    pub fn replace_from_scope<Relation, Scope>(
        mut self,
        relation: ApplicationRelationRef<Schema, Relation, Scope, Entity>,
        key: &str,
    ) -> Result<Self, Denial>
    where
        Relation:
            OperationReads<Operation> + OperationLinks<Operation> + OperationUnlinks<Operation>,
    {
        if relation.integrity().cardinality.source_max != Some(1)
            || self
                .links
                .iter()
                .any(|link| link.relation == relation.name())
        {
            return Err(Denial::InvalidCreationLink);
        }
        self = self.link_from_scope(relation, key)?;
        self.links.last_mut().expect("newly declared link").posture =
            ApplicationOperationBodyCreationLinkPosture::ReplaceSingle;
        Ok(self)
    }
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
            || self.links.iter().any(|link| {
                link.relation == relation && (link.key == key || link.posture.replaces_selection())
            })
        {
            return Err(Denial::InvalidCreationLink);
        }
        self.links.push(ApplicationOperationBodyCreationLink {
            relation: relation.to_owned(),
            from: from.to_owned(),
            to: to.to_owned(),
            key: key.to_owned(),
            direction,
            posture: ApplicationOperationBodyCreationLinkPosture::Append,
        });
        Ok(self)
    }
}
