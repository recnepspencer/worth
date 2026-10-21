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
            && c.peers().is_empty()
            && c.path_links().is_empty()
            && c.signing().is_none()
            && c.signing_lifecycle().is_none()
            && c.enrollment().is_none()
            && c.subject_impact_bound().is_none()
            && !c.reuses_current_if_unchanged()
            && !c.key().is_empty()
            && c.key().trim() == c.key()
            && !c.links().is_empty()
            && c.links()[0].direction == Direction::FromScope
            && c.links()[0].posture == Posture::Append
            && c.links()[0].key == c.key()
            && c.links()[0].created_entity() == c.entity()
            && c.links().iter().enumerate().all(|(index, link)| {
                link.direction == Direction::FromScope
                    && matches!(link.source, crate::application_schema::ApplicationOperationBodyCreationLinkSource::Scope)
                    && link.posture == Posture::Append
                    && link.scope_entity() == c.links()[0].scope_entity()
                    && link.created_entity() == c.entity()
                    && !link.key.is_empty()
                    && link.key.trim() == link.key
                    && !c.links()[..index]
                        .iter()
                        .any(|prior| prior.key == link.key || prior.relation == link.relation)
            })
            && c.fields().iter().all(|f| {
                matches!(
                    f,
                    Field::Literal { .. } | Field::Input { .. } | Field::OptionalInput { .. }
                )
            })
    }
}
impl Body {
    /// Multiple parent associations require their own canonical and portable profile.
    pub fn has_multiple_child_links(&self) -> bool {
        self.creation()
            .into_iter()
            .flat_map(|c| c.creation_roots())
            .flat_map(|c| c.children())
            .any(|child| child.creation.links().len() > 1)
    }
    pub fn has_supported_child_creations(&self) -> bool {
        for root in self
            .creation()
            .into_iter()
            .flat_map(|c| c.creation_roots())
            .filter(|c| !c.children().is_empty())
        {
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
        }
        true
    }
}
pub(super) fn validate(operation: &str, body: &Body, members: &[Member]) -> Result<(), Denial> {
    if !body.has_supported_child_creations() {
        return Err(Denial::program_body_dependency(operation));
    }
    for child in body
        .creation()
        .into_iter()
        .flat_map(|c| c.creation_roots())
        .flat_map(|c| c.children())
    {
        for link in child.creation.links() {
            if !members.iter().any(|m|matches!(m,Member::Relation{relation,from,to,integrity}
            if relation==&link.relation && from==&link.from && to==&link.to
                && integrity.cardinality.target_max==Some(1) && integrity.cross_context_policy()==ApplicationRelationCrossContextPolicy::Forbid))
            || !members.iter().any(|m|matches!(m,Member::OperationProgram{operation:op,target} if op==operation && target==&link.program_target())) {
            return Err(Denial::program_body_dependency(operation));
        }
        }
    }
    Ok(())
}
