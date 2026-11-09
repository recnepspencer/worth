//! Installed authored families select Native's roster check before relation validation.
use super::WorthQueryStagedApplicationAttempt;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationOutputPosture as Posture, WorthQueryPrimaryGraphLayout,
};
use crate::domain_computation::WorthQueryInvariantExecutionFailure as Failure;
use worth_query_declaration::facade::application_schema::ApplicationSchemaMember;
use worth_relational::facade::{
    identity::EntityId,
    mvcc::{AuthoredOwnedMemberKind, RequiredOutputRoleTarget},
    transactions::EntityReference,
};

impl WorthQueryStagedApplicationAttempt<'_> {
    pub(in crate::domain_computation::primary_graph::provider) fn owned_member_rosters(
        &self,
        layout: &WorthQueryPrimaryGraphLayout,
    ) -> Result<
        Vec<(
            EntityId,
            Vec<AuthoredOwnedMemberKind>,
            Vec<RequiredOutputRoleTarget>,
        )>,
        Failure,
    > {
        let inverse = self.attempt.aftermath_causality().is_some();
        let identity = self.attempt.authored_correction_identity();
        let Some(identity) = identity else {
            return Ok(Vec::new());
        };
        let failure = super::super::invariant_execution::touch_failure;
        let kinds = self.attempt.authored_member_kinds(layout)?;
        if kinds.is_empty() {
            return Ok(Vec::new());
        }
        let (roles, _) = super::super::native_publication_roles::collect(
            layout,
            self.attempt.native_publication_roles(),
        )
        .ok_or_else(failure)?;
        let mut rosters = Vec::new();
        for anchor in std::iter::once(identity)
            .chain(identity.co_anchors())
            .filter(|anchor| anchor.member_owner().is_none())
        {
            let ApplicationSchemaMember::Field {
                entity: anchor_entity,
                ..
            } = anchor.parts().logical_key.as_ref()
            else {
                return Err(failure());
            };
            let anchor_kinds = kinds
                .iter()
                .filter(|kind| {
                    identity.co_anchors().iter().any(|member| {
                        member.member_owner().is_some_and(|owner| {
                            owner.to() == anchor_entity
                                && layout
                                    .relation(owner.relation())
                                    .is_some_and(|relation| relation.kind == kind.owner_relation)
                        })
                    })
                })
                .copied()
                .collect::<Vec<_>>();
            if anchor_kinds.is_empty() {
                continue;
            }
            let name = if inverse {
                &anchor.parts().inverse_role
            } else {
                &anchor.parts().source_role
            };
            for (_, posture, _, target) in self
                .attempt
                .native_publication_roles()
                .filter(|(role, _, _, _)| role == name)
            {
                if let (Posture::Retire, EntityReference::Existing(id)) = (posture, target) {
                    rosters.push((*id, anchor_kinds.clone(), roles.clone()));
                }
            }
        }
        Ok(rosters)
    }
}

impl super::super::WorthQueryPrimaryGraphApplicationAttempt {
    pub(in crate::domain_computation::primary_graph::provider) fn authored_member_kinds(
        &self,
        layout: &WorthQueryPrimaryGraphLayout,
    ) -> Result<Vec<AuthoredOwnedMemberKind>, Failure> {
        let Some(identity) = self.authored_correction_identity() else {
            return Ok(Vec::new());
        };
        let failure = super::super::invariant_execution::touch_failure;
        let mut kinds = Vec::new();
        for member in identity.co_anchors() {
            let Some(owner) = member.member_owner() else {
                continue;
            };
            let ApplicationSchemaMember::Field { entity, .. } = member.parts().logical_key.as_ref()
            else {
                return Err(failure());
            };
            let kind = AuthoredOwnedMemberKind {
                member_kind: layout.entity_kind(entity).ok_or_else(failure)?,
                owner_relation: layout.relation(owner.relation()).ok_or_else(failure)?.kind,
            };
            if !kinds.iter().any(|prior: &AuthoredOwnedMemberKind| {
                prior.member_kind == kind.member_kind && prior.owner_relation == kind.owner_relation
            }) {
                kinds.push(kind);
            }
        }
        Ok(kinds)
    }
}
