//! The members one mutation moved across owners that stay live.
use super::{
    AuthoredCorrectionCandidateDenial, Denial, Posture, PreparedAuthoredIdentity,
    WorthQuerySelectedCorrectionSource,
};
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::{
    application_aftermath::{ApplicationCorrectionBinding, DeclaredAuthoredCorrection},
    application_schema::ApplicationSchema,
};
use worth_relational::facade::{
    identity::EntityId,
    transactions::{CreatedEntityRef, EntityReference, WorkerIntentBatch},
};

/// The source's authored inverse is one anchor lifetime, or zero to many
/// members of several families whose owners stay live. One declaration never
/// mixes the two.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in super::super) enum PreparedAuthoredCorrection {
    Anchored(PreparedAuthoredIdentity),
    LiveOwnerMembers {
        declaration: DeclaredAuthoredCorrection,
        members: Vec<PreparedAuthoredIdentity>,
    },
}
use PreparedAuthoredCorrection::{Anchored, LiveOwnerMembers};

impl PreparedAuthoredCorrection {
    /// Work is one witnessed-role scan per declared family and one key read
    /// per member the source moved; a source that moved none prepares none.
    pub(in super::super) fn prepare<S: ApplicationSchema, C: ApplicationCorrectionBinding<S>>(
        source: &WorthQuerySelectedCorrectionSource<'_, S>,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Option<Self>, Denial> {
        let Some(identity) = C::authored_identity() else {
            return Ok(None);
        };
        let declaration = identity
            .into_declaration()
            .map_err(|_| Denial::UnsupportedSourceMeaning)?;
        if declaration.live_member_owner().is_none() {
            return Ok(PreparedAuthoredIdentity::prepare_identity::<S, C>(
                source,
                request,
                declaration,
            )?
            .map(Anchored));
        }
        source
            .occurrence()
            .correspondence()
            .require_source_binding::<C::Source>()
            .map_err(|cause| {
                Denial::SourceOutput(
                    super::super::material::WorthQueryCorrectionOutputProjectionDenial::Output(
                        cause,
                    ),
                )
            })?;
        let mut members = Vec::new();
        for family in std::iter::once(&declaration).chain(declaration.co_anchors()) {
            PreparedAuthoredIdentity::prepare_members::<S, C>(
                source,
                request,
                family,
                &mut members,
            )?;
        }
        Ok(Some(LiveOwnerMembers {
            declaration,
            members,
        }))
    }

    pub(in super::super) fn declaration(&self) -> &DeclaredAuthoredCorrection {
        match self {
            Anchored(identity) => identity.declaration(),
            LiveOwnerMembers { declaration, .. } => declaration,
        }
    }

    /// Exact members under live owners; an anchored inverse has none.
    pub(in super::super) fn live_members(&self) -> &[PreparedAuthoredIdentity] {
        match self {
            Anchored(_) => &[],
            LiveOwnerMembers { members, .. } => members,
        }
    }

    pub(in super::super) fn retirement_anchor(&self) -> Option<EntityId> {
        match self {
            Anchored(identity) => identity.retirement_anchor(),
            LiveOwnerMembers { members, .. } => members
                .iter()
                .find_map(PreparedAuthoredIdentity::retirement_anchor),
        }
    }

    pub(in super::super) fn retirement_co_anchors(&self) -> Vec<EntityId> {
        match self {
            Anchored(identity) => identity.retirement_co_anchors(),
            LiveOwnerMembers { members, .. } => members
                .iter()
                .filter_map(PreparedAuthoredIdentity::retirement_anchor)
                .skip(1)
                .collect(),
        }
    }

    /// A member under a live owner restores by its logical key in Native's
    /// relation footprint, never as a restored anchor.
    pub(in super::super) fn restoration_anchor<'a>(
        &self,
        roles: impl Iterator<Item = (&'a str, Posture, &'a str, &'a EntityReference)>,
    ) -> Option<(EntityId, CreatedEntityRef)> {
        match self {
            Anchored(identity) => identity.restoration_anchor(roles),
            LiveOwnerMembers { .. } => None,
        }
    }

    pub(in super::super) fn restoration_co_anchors<'a>(
        &self,
        roles: impl Iterator<Item = (&'a str, Posture, &'a str, &'a EntityReference)> + Clone,
    ) -> Vec<(EntityId, CreatedEntityRef)> {
        match self {
            Anchored(identity) => identity.restoration_co_anchors(roles),
            LiveOwnerMembers { .. } => Vec::new(),
        }
    }

    /// Every recorded member answers through its own inverse role. A member
    /// the source did not record is Native's refusal: only its relation
    /// footprint sees every actual birth.
    pub(in super::super) fn validate_candidate<'a>(
        &self,
        roles: impl Iterator<Item = (&'a str, Posture, &'a str, &'a EntityReference)> + Clone,
        batch: &WorkerIntentBatch,
        request: &WorthQueryRequestScope,
    ) -> Result<(), AuthoredCorrectionCandidateDenial> {
        match self {
            Anchored(identity) => identity.validate_candidate(roles, batch, request),
            LiveOwnerMembers { members, .. } => {
                for member in members {
                    member.validate_candidate(roles.clone(), batch, request)?;
                }
                Ok(())
            }
        }
    }
}
