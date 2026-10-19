//! Typed child sites are inert authored meaning, not execution callbacks or context authority.
use super::{
    ApplicationOperationBodyAuthoringDenial as Denial,
    ApplicationOperationBodyCreation as Creation, ApplicationOperationBodyCreationBuilder,
    ApplicationOperationBodyCreationLink as Link,
    ApplicationOperationBodyCreationLinkDirection as Direction,
    ApplicationOperationBodyCreationLinkPosture as Posture,
};
use crate::application_schema::{
    ApplicationFieldRef, ApplicationFieldUnit, ApplicationRelationRef,
    DeclaredApplicationFieldValue, OperationCreates, OperationLinks, OperationWrites,
    OptionalApplicationFieldValue,
};
use std::marker::PhantomData;
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationOperationBodyChildCreation {
    pub parent_key: String,
    pub creation: Creation,
}
pub struct ApplicationOperationBodyChildCreationBuilder<S, O, I, E> {
    creation: Creation,
    marker: PhantomData<fn() -> (S, O, I, E)>,
}
impl<S, O, I, E> ApplicationOperationBodyChildCreationBuilder<S, O, I, E> {
    pub fn initialize<A, F, V, W, Q, U>(
        mut self,
        field: ApplicationFieldRef<S, E, A, F, V, W, Q, U>,
        value: V,
    ) -> Result<Self, Denial>
    where
        F: OperationWrites<O> + DeclaredApplicationFieldValue<Value = V>,
        U: ApplicationFieldUnit,
    {
        self.creation
            .fields
            .push(super::initialization::literal::<S, O, E, A, F, V, W, Q, U>(
                field, value,
            )?);
        Ok(self)
    }
    pub fn initialize_input<A, F, V, W, Q, U>(
        mut self,
        field: ApplicationFieldRef<S, E, A, F, V, W, Q, U>,
        member: &str,
    ) -> Result<Self, Denial>
    where
        F: OperationWrites<O> + DeclaredApplicationFieldValue<Value = V>,
        U: ApplicationFieldUnit,
    {
        self.creation
            .fields
            .push(super::initialization::input::<S, O, E, A, F, V, W, Q, U>(
                field, member, false,
            )?);
        Ok(self)
    }
    pub fn initialize_optional_input<A, F, V, W, Q, U>(
        mut self,
        field: ApplicationFieldRef<S, E, A, F, V, W, Q, U>,
        member: &str,
    ) -> Result<Self, Denial>
    where
        F: OperationWrites<O> + OptionalApplicationFieldValue<Value = V>,
        U: ApplicationFieldUnit,
    {
        self.creation
            .fields
            .push(super::initialization::input::<S, O, E, A, F, V, W, Q, U>(
                field, member, true,
            )?);
        Ok(self)
    }
    pub fn with_child<R, Child, F>(
        mut self,
        relation: ApplicationRelationRef<S, R, E, Child>,
        key: &str,
        build: F,
    ) -> Result<Self, Denial>
    where
        Child: OperationCreates<O>,
        R: OperationLinks<O>,
        F: FnOnce(
            ApplicationOperationBodyChildCreationBuilder<S, O, I, Child>,
        )
            -> Result<ApplicationOperationBodyChildCreationBuilder<S, O, I, Child>, Denial>,
    {
        self.creation
            .children
            .extend(build_child(&self.creation.key, relation, key, build)?);
        Ok(self)
    }
}
impl<S, O, I, E> ApplicationOperationBodyCreationBuilder<S, O, I, E> {
    /// Atomically creates a child and its incoming-exclusive same-context link.
    /// The closure runs during authoring only; Query executes the resulting sites.
    ///
    /// ```compile_fail,E0277
    /// use worth_query_declaration::facade::application_schema::*;
    /// fn missing_create<S,O,I,E,R,Child>(creation:ApplicationOperationBodyCreationBuilder<S,O,I,E>,relation:ApplicationRelationRef<S,R,E,Child>)
    /// where R:OperationLinks<O> {
    ///     let _ = creation.with_child(relation,"child",Ok);
    /// }
    /// ```
    ///
    /// ```no_run
    /// use worth_query_declaration::facade::application_schema::*;
    /// fn declared_create<S,O,I,E,R,Child>(creation:ApplicationOperationBodyCreationBuilder<S,O,I,E>,relation:ApplicationRelationRef<S,R,E,Child>)
    /// where R:OperationLinks<O>,Child:OperationCreates<O> {
    ///     let _ = creation.with_child(relation,"child",Ok);
    /// }
    /// ```
    pub fn with_child<R, Child, F>(
        mut self,
        relation: ApplicationRelationRef<S, R, E, Child>,
        key: &str,
        build: F,
    ) -> Result<Self, Denial>
    where
        Child: OperationCreates<O>,
        R: OperationLinks<O>,
        F: FnOnce(
            ApplicationOperationBodyChildCreationBuilder<S, O, I, Child>,
        )
            -> Result<ApplicationOperationBodyChildCreationBuilder<S, O, I, Child>, Denial>,
    {
        self.children
            .extend(build_child(&self.key, relation, key, build)?);
        Ok(self)
    }
}
fn build_child<S, O, I, E, R, Child, F>(
    parent: &str,
    relation: ApplicationRelationRef<S, R, E, Child>,
    key: &str,
    build: F,
) -> Result<Vec<ApplicationOperationBodyChildCreation>, Denial>
where
    Child: OperationCreates<O>,
    R: OperationLinks<O>,
    F: FnOnce(
        ApplicationOperationBodyChildCreationBuilder<S, O, I, Child>,
    ) -> Result<ApplicationOperationBodyChildCreationBuilder<S, O, I, Child>, Denial>,
{
    use crate::application_schema::ApplicationRelationCrossContextPolicy;
    if key.is_empty()
        || key.trim() != key
        || relation.integrity().cardinality.target_max != Some(1)
        || relation.integrity().cross_context_policy()
            != ApplicationRelationCrossContextPolicy::Forbid
    {
        return Err(Denial::InvalidCreationLink);
    }
    let link = Link {
        relation: relation.name().to_owned(),
        from: relation.from().to_owned(),
        to: relation.to().to_owned(),
        key: key.to_owned(),
        direction: Direction::FromScope,
        posture: Posture::Append,
    };
    let mut child = build(ApplicationOperationBodyChildCreationBuilder {
        creation: Creation::from_untrusted(relation.to().to_owned(), key.to_owned(), vec![])
            .with_untrusted_links(vec![link]),
        marker: PhantomData,
    })?
    .creation;
    let descendants = std::mem::take(&mut child.children);
    Ok(std::iter::once(ApplicationOperationBodyChildCreation {
        parent_key: parent.to_owned(),
        creation: child,
    })
    .chain(descendants)
    .collect())
}
