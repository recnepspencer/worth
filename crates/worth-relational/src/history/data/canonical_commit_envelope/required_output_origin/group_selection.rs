//! Exact route/member selection within a multi-route canonical retirement.
use super::*;
use crate::{history::data::CodecRetirementClosureStop as Stop, identity::data::RelationId};
use std::sync::Arc;
use worth_execution::MapKernelContext;

impl CanonicalCommitEnvelopeBody {
    pub(crate) fn output_origins(
        &self,
    ) -> impl Iterator<Item = &Arc<CanonicalRequiredOutputOrigin>> {
        self.required_output_origin
            .iter()
            .chain(
                self.historical_output_selection
                    .iter()
                    .flat_map(|event| event.publications.iter()),
            )
            .chain(
                self.producer_cascade_retirement
                    .iter()
                    .flat_map(|event| event.publications.iter()),
            )
            .chain(
                self.authored_cascade_retirement
                    .iter()
                    .flat_map(|event| event.publications.iter()),
            )
            .chain(
                self.authored_cascade_restoration
                    .iter()
                    .flat_map(|event| event.publications.iter()),
            )
            .chain(
                self.coupled_authored_publication
                    .iter()
                    .flat_map(|event| event.output_origins()),
            )
    }

    fn validate_cascade_header(&self, context: &mut MapKernelContext<'_, '_>) -> Result<(), Stop> {
        if let Some(event) = &self.coupled_authored_publication {
            return self.validate_coupled_authored_header(event, context);
        }
        self.validate_historical_selection_header(context)?;
        self.validate_restoration_header(context)?;
        self.validate_producer_retirement_header(context)?;
        let Some(event) = &self.authored_cascade_retirement else {
            return Ok(());
        };
        self.validate_authored_retirement_components(event, false, context)
    }

    pub(super) fn validate_authored_retirement_components(
        &self,
        event: &CanonicalAuthoredRetirement,
        allow_empty: bool,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<(), Stop> {
        use crate::{
            mvcc::RequiredOutputRolePosture as Posture,
            publication::patch::data::RecordStructuralChange as Change,
            transactions::data::RecordRef,
        };
        let charge = super::super::super::codec_retirement_closure::charge;
        charge(context, self.patch.authoritative_record_patches.len() + 1)?;
        if self.producer_cascade_retirement.is_some()
            || self.required_output_origin.is_some()
            || (!allow_empty && event.publications.is_empty())
            || !self.patch.authoritative_record_patches.iter().any(|patch| {
                patch.target == RecordRef::Entity(event.anchor)
                    && patch.structural_change == Change::Deleted
            })
        {
            return Err("copied or mixed canonical authored cascade".into());
        }
        for (index, anchor) in event.anchors().enumerate() {
            charge(
                context,
                index + self.patch.authoritative_record_patches.len() + 1,
            )?;
            if event.anchors().take(index).any(|prior| prior == anchor)
                || !self.patch.authoritative_record_patches.iter().any(|patch| {
                    patch.target == RecordRef::Entity(anchor)
                        && patch.structural_change == Change::Deleted
                })
            {
                return Err("canonical cascade repeats or omits an authored anchor".into());
            }
        }
        for (index, origin) in event.publications.iter().enumerate() {
            charge(
                context,
                origin
                    .members
                    .len()
                    .saturating_mul(event.co_anchors.len() + 1)
                    + 1,
            )?;
            self.validate_output_origin_header(origin)?;
            if !origin.source_scope_owned
                || origin.publication.predecessor().is_none()
                || origin.members.iter().any(|member| {
                    member.posture != Posture::Retire || event.contains_anchor(member.id)
                })
            {
                return Err("canonical cascade contains an unowned or live publication".into());
            }
            for prior in &event.publications[..index] {
                charge(
                    context,
                    origin.meaning.family.len()
                        + prior.meaning.family.len()
                        + 64
                        + origin.members.len().saturating_mul(prior.members.len()),
                )?;
                if prior.meaning.same_route(&origin.meaning)
                    || origin
                        .members
                        .iter()
                        .any(|member| prior.outputs.binary_search(&member.id).is_ok())
                {
                    return Err("canonical cascade repeats a route or member".into());
                }
            }
            let mut source = origin.meaning.source_scope;
            for _ in 0..event.publications.len() {
                charge(context, event.co_anchors.len() + 1)?;
                if event.contains_anchor(source) {
                    break;
                }
                let mut parent = None;
                for candidate in &event.publications {
                    charge(context, candidate.members.len() + 1)?;
                    if candidate.outputs.binary_search(&source).is_ok() {
                        parent = Some(candidate.meaning.source_scope);
                    }
                }
                source = parent.ok_or("canonical cascade has a different source owner")?;
            }
            if !event.contains_anchor(source) {
                return Err("canonical cascade source ownership is cyclic".into());
            }
        }
        Ok(())
    }
}

/// The envelope owner alone mints this immutable, envelope-bound validation result.
#[derive(Clone, Copy)]
pub(crate) struct ValidatedOutputOrigins<'a> {
    envelope: &'a CanonicalCommitEnvelopeBody,
}

impl CanonicalCommitEnvelopeBody {
    pub(crate) fn validate_output_origins(
        &self,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<ValidatedOutputOrigins<'_>, Stop> {
        self.validate_cascade_header(context)?;
        // Group validators certify their complete origin rosters. A standalone
        // publication has no component validator to establish that fact.
        if self.producer_cascade_retirement.is_none() {
            if let Some(origin) = &self.required_output_origin {
                super::super::super::codec_retirement_closure::charge(
                    context,
                    origin.members.len() + 1,
                )?;
                self.validate_output_origin_header(origin)?;
            }
        }
        Ok(ValidatedOutputOrigins { envelope: self })
    }
}

impl<'a> ValidatedOutputOrigins<'a> {
    pub(crate) fn output_origin_for_entity(
        &self,
        id: EntityId,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<&'a Arc<CanonicalRequiredOutputOrigin>, Stop> {
        for origin in self.envelope.output_origins() {
            super::super::super::codec_retirement_closure::charge(
                context,
                (usize::BITS - origin.outputs.len().leading_zeros()) as usize + 1,
            )?;
            if origin.outputs.binary_search(&id).is_ok() {
                return Ok(origin);
            }
        }
        Err("exact member has no canonical publication".into())
    }

    pub(crate) fn output_origin_for_relation(
        &self,
        id: RelationId,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<&'a Arc<CanonicalRequiredOutputOrigin>, Stop> {
        for origin in self.envelope.output_origins() {
            super::super::super::codec_retirement_closure::charge(
                context,
                origin.relations.len() + 1,
            )?;
            if origin.relations.iter().any(|edge| edge.id == id) {
                return Ok(origin);
            }
        }
        Err("incident relation is outside complete publication membership".into())
    }

    pub(crate) fn output_origin_for_route(
        &self,
        meaning: &crate::mvcc::RequiredOutputPublicationMeaning,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<&'a Arc<CanonicalRequiredOutputOrigin>, Stop> {
        for origin in self.envelope.output_origins() {
            super::super::super::codec_retirement_closure::charge(
                context,
                origin.meaning.family.len() + meaning.family.len() + 65,
            )?;
            if origin.meaning.same_route(meaning) {
                return Ok(origin);
            }
        }
        Err("exact route has no canonical publication".into())
    }
}

impl<'a> ValidatedOutputOrigins<'a> {
    pub(crate) fn envelope(&self) -> &'a CanonicalCommitEnvelopeBody {
        self.envelope
    }
}

/// Owns the exact immutable envelope whose header was admitted. No raw-envelope
/// constructor or mutation surface can separate the header proof from its data.
#[derive(Clone)]
pub(crate) struct OwnedValidatedOutputOrigins {
    envelope: Arc<CanonicalCommitEnvelope>,
}

impl OwnedValidatedOutputOrigins {
    pub(crate) fn validate(
        envelope: Arc<CanonicalCommitEnvelope>,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<Self, Stop> {
        envelope.validate_output_origins(context)?;
        Ok(Self { envelope })
    }

    pub(crate) fn as_view(&self) -> ValidatedOutputOrigins<'_> {
        ValidatedOutputOrigins {
            envelope: &self.envelope,
        }
    }
}

impl std::ops::Deref for OwnedValidatedOutputOrigins {
    type Target = Arc<CanonicalCommitEnvelope>;
    fn deref(&self) -> &Self::Target {
        &self.envelope
    }
}

impl OwnedValidatedOutputOrigins {
    pub(crate) fn validate_output_membership_header(
        &self,
    ) -> Result<&CanonicalRequiredOutputOrigin, Stop> {
        self.as_view().validate_output_membership_header()
    }

    pub(crate) fn output_origin_for_entity(
        &self,
        id: EntityId,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<&Arc<CanonicalRequiredOutputOrigin>, Stop> {
        self.as_view().output_origin_for_entity(id, context)
    }
    pub(crate) fn output_origin_for_relation(
        &self,
        id: RelationId,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<&Arc<CanonicalRequiredOutputOrigin>, Stop> {
        self.as_view().output_origin_for_relation(id, context)
    }
    pub(crate) fn output_origin_for_route(
        &self,
        meaning: &crate::mvcc::RequiredOutputPublicationMeaning,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<&Arc<CanonicalRequiredOutputOrigin>, Stop> {
        self.as_view().output_origin_for_route(meaning, context)
    }
}
