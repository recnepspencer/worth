//! Explicit unsigned disposal of one exclusive predecessor; ordinary replacement retains history.
use super::{
    ApplicationOperationBodyAuthoringDenial as Denial, ApplicationOperationBodyCreation,
    ApplicationOperationBodyCreationBuilder,
    ApplicationOperationBodyCreationLinkPosture as Posture,
    ApplicationOperationBodyProgram as Body,
};
use crate::application_schema::{
    ApplicationOperationBodyOutputTarget as Output, ApplicationRelationRef, OperationDeletes,
    OperationLinks, OperationReads, OperationUnlinks,
};
impl<S, O, I, E> ApplicationOperationBodyCreationBuilder<S, O, I, E> {
    /// Replace one outgoing current selection and delete its observed predecessor atomically.
    /// Incoming exclusivity is relation-local; native deletion invariants still apply.
    /// This unsigned profile uses one link and distinct scope/selected entity kinds.
    ///
    /// ```compile_fail,E0277
    /// use worth_query_declaration::facade::application_schema::*;
    /// fn missing_delete<S,O,I,E,Root,R>(creation: ApplicationOperationBodyCreationBuilder<S,O,I,E>, relation: ApplicationRelationRef<S,R,Root,E>)
    /// where Root: OperationReads<O>, E: OperationReads<O>, R: OperationReads<O> + OperationLinks<O> + OperationUnlinks<O> {
    ///     let _ = creation.replace_and_retire_from_scope(relation,"current");
    /// }
    /// ```
    ///
    /// ```no_run
    /// use worth_query_declaration::facade::application_schema::*;
    /// fn declared_delete<S,O,I,E,Root,R>(creation: ApplicationOperationBodyCreationBuilder<S,O,I,E>, relation: ApplicationRelationRef<S,R,Root,E>)
    /// where Root: OperationReads<O>, E: OperationReads<O> + OperationDeletes<O>, R: OperationReads<O> + OperationLinks<O> + OperationUnlinks<O> {
    ///     let _ = creation.replace_and_retire_from_scope(relation,"current");
    /// }
    /// ```
    pub fn replace_and_retire_from_scope<R, Root>(
        mut self,
        relation: ApplicationRelationRef<S, R, Root, E>,
        key: &str,
    ) -> Result<Self, Denial>
    where
        Root: OperationReads<O>,
        E: OperationReads<O> + OperationDeletes<O>,
        R: OperationReads<O> + OperationLinks<O> + OperationUnlinks<O>,
    {
        if relation.integrity().cardinality.target_max != Some(1)
            || relation.from() == relation.to()
        {
            return Err(Denial::InvalidCreationLink);
        }
        self = self.replace_from_scope(relation, key)?;
        self.links.last_mut().expect("new replacement").posture = Posture::ReplaceAndRetireSingle;
        Ok(self)
    }
}
impl ApplicationOperationBodyCreation {
    pub fn retires_predecessor(&self) -> bool {
        self.links()
            .iter()
            .any(|link| link.posture.retires_predecessor())
    }
}
impl Body {
    /// Inert profile validation, not deletion authority. Admission and installation remain mandatory.
    pub fn has_supported_retiring_replacement(&self) -> bool {
        let Some(creation) = self
            .creation()
            .filter(|creation| creation.retires_predecessor())
        else {
            return true;
        };
        creation.links().len() == 1
            && self.scope_entity() != Some(creation.entity())
            && creation.signing().is_none()
            && !self.outputs().iter().any(
                |output| matches!(&output.target,Output::Related{to,..} if to==creation.entity()),
            )
            && !self
                .path_value_assignments()
                .iter()
                .any(|step| step.target.field.entity() == creation.entity())
            && !self
                .retirements()
                .iter()
                .any(|rule| rule.selected == creation.entity())
    }
}
