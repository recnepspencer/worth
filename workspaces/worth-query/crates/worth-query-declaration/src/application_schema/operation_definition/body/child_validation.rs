//! One flat, prior-parent inventory closes every child create/write/link before execution.
use super::{
    ApplicationOperationBodyChildCreation as Child, ApplicationOperationBodyCreatedField as Field,
    ApplicationOperationBodyCreationLinkDirection as Direction,
    ApplicationOperationBodyCreationLinkPosture as Posture,
    ApplicationOperationBodyProgram as Body,
};
use crate::application_schema::{
    ApplicationRelationCrossContextPolicy, ApplicationSchemaDeclarationDenial as Denial,
    ApplicationSchemaMember as Member,
};
impl Child {
    /// Structural profile; schema contracts and field codecs still require fresh admission.
    pub fn has_supported_shape(&self) -> bool {
        let c = &self.creation;
        c.children().is_empty()
            && c.signing().is_none()
            && c.signing_lifecycle().is_none()
            && c.enrollment().is_none()
            && c.subject_impact_bound().is_none()
            && !c.reuses_current_if_unchanged()
            && !c.key().is_empty()
            && c.key().trim() == c.key()
            && c.links().len() == 1
            && c.links()[0].direction == Direction::FromScope
            && c.links()[0].posture == Posture::Append
            && c.links()[0].key == c.key()
            && c.links()[0].created_entity() == c.entity()
            && c.fields().iter().all(|f| {
                matches!(
                    f,
                    Field::Literal { .. } | Field::Input { .. } | Field::OptionalInput { .. }
                )
            })
    }
}
impl Body {
    pub fn has_supported_child_creations(&self) -> bool {
        let Some(root) = self.creation().filter(|c| !c.children().is_empty()) else {
            return true;
        };
        if root.signing().is_some()
            || root.signing_lifecycle().is_some()
            || root.enrollment().is_some()
            || root.subject_impact_bound().is_some()
            || root.reuses_current_if_unchanged()
            || self.succession().is_some()
            || root.links().iter().any(|l| l.posture != Posture::Append)
        {
            return false;
        }
        let mut parents = std::collections::BTreeMap::from([(root.key(), root.entity())]);
        for child in root.children() {
            if !child.has_supported_shape()
                || parents.get(child.parent_key.as_str()).copied()
                    != Some(child.creation.links()[0].scope_entity())
                || parents
                    .insert(child.creation.key(), child.creation.entity())
                    .is_some()
            {
                return false;
            }
        }
        true
    }
}
pub(super) fn validate(operation: &str, body: &Body, members: &[Member]) -> Result<(), Denial> {
    if !body.has_supported_child_creations() {
        return Err(Denial::MissingOperationProgramDependency);
    }
    for child in body.creation().into_iter().flat_map(|c| c.children()) {
        let link = &child.creation.links()[0];
        if !members.iter().any(|m|matches!(m,Member::Relation{relation,from,to,integrity}
            if relation==&link.relation && from==&link.from && to==&link.to
                && integrity.cardinality.target_max==Some(1) && integrity.cross_context_policy()==ApplicationRelationCrossContextPolicy::Forbid))
            || !members.iter().any(|m|matches!(m,Member::OperationProgram{operation:op,target} if op==operation && target==&link.program_target())) {
            return Err(Denial::MissingOperationProgramDependency);
        }
    }
    Ok(())
}
