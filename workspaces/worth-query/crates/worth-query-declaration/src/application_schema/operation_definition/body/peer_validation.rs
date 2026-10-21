//! Every independent site retains full closure; unsupported site controls fail closed.
use super::{
    ApplicationOperationBodyCreationLinkPosture as Posture, ApplicationOperationBodyProgram as Body,
};
use crate::application_schema::{
    ApplicationSchemaDeclarationDenial as Denial, ApplicationSchemaMember as Member,
};
impl Body {
    pub fn has_supported_peer_creations(&self) -> bool {
        let Some(root) = self.creation() else {
            return true;
        };
        let mut keys = std::collections::BTreeSet::new();
        if !root.initialization_sites().all(|c| keys.insert(c.key())) {
            return false;
        }
        if !root.peers().is_empty() && (root.enrollment().is_some() || self.succession().is_some())
        {
            return false;
        }
        if root.initialization_sites().any(|c| {
            !c.path_links().is_empty()
                && (c.reuses_current_if_unchanged() || c.retires_predecessor())
                || c.path_links().iter().any(|p| {
                    !p.has_supported_shape()
                        || Some(p.path.start_entity()) != self.scope_entity()
                        || p.link.created_entity() != c.entity()
                })
        }) {
            return false;
        }
        root.peers().iter().all(|peer| {
            peer.peers().is_empty()
                && peer.signing().is_none()
                && peer.signing_lifecycle().is_none()
                && peer.subject_impact_bound().is_none()
                && peer.enrollment().is_none()
                && !peer.reuses_current_if_unchanged()
                && peer.links().iter().all(|l| l.posture == Posture::Append)
        })
    }
}
pub(super) fn validate(operation: &str, body: &Body, members: &[Member]) -> Result<(), Denial> {
    if !body.has_supported_peer_creations() {
        return Err(Denial::program_body_dependency(operation));
    }
    for site in body
        .creation()
        .into_iter()
        .flat_map(|c| c.initialization_sites())
    {
        let mut keys = std::collections::BTreeSet::new();
        if site
            .links()
            .iter()
            .chain(site.path_links().iter().map(|p| &p.link))
            .any(|link| !keys.insert((&link.relation, &link.key)))
        {
            return Err(Denial::DuplicateMember);
        }
        super::classified_capture::validate_creation(site)
            .map_err(|_| Denial::program_body_dependency(operation))?;
        for path in site.path_links() {
            if !path.path.steps().iter().all(|step| members.iter().any(|m|matches!(m,Member::Relation{relation,from,to,..}
                if relation==step.relation() && from==step.from() && to==step.to())))
                || !path.decision_reads().iter().all(|read|members.iter().any(|m|matches!(m,Member::OperationDecisionRead{operation:op,target} if op==operation && target==read)))
                || !members.iter().any(|m|matches!(m,Member::Relation{relation,from,to,..} if relation==&path.link.relation && from==&path.link.from && to==&path.link.to))
                || !members.iter().any(|m|matches!(m,Member::OperationProgram{operation:op,target} if op==operation && target==&path.link.program_target()))
            { return Err(Denial::program_body_dependency(operation)); }
        }
    }
    Ok(())
}
