//! Typed coupling sees only the completed read owner's demanded original fields.
use super::super::{
    effect_program::output_correspondence::WorthQueryApplicationOutputCorrespondenceCandidate,
    read_set::{CompletedFactProjection, WorthQueryCompleteApplicationReadSet},
    WorthQueryApplicationObservedFact as Fact,
};
use crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload;
use worth_query_declaration::facade::{
    application_aftermath::{ApplicationCorrectionPreImage, DeclaredPreImageLocus},
    application_operation::ApplicationMutationOutputRoleDescriptor,
};
use worth_query_installation::facade::{InstalledCorrectionMechanism, InstalledPreImageDemand};
use worth_relational::facade::{identity::EntityId, transactions::EntityReference};

pub(in crate::domain_computation::primary_graph::application_attempt) struct CoupledPreImage<'a> {
    facts: &'a FactPayload,
    projection: Option<&'a CompletedFactProjection>,
    demand: Option<&'a InstalledPreImageDemand>,
    outputs: &'a WorthQueryApplicationOutputCorrespondenceCandidate,
}
impl<'a> CoupledPreImage<'a> {
    pub(in crate::domain_computation::primary_graph::application_attempt) fn new<
        Schema,
        Operation,
        Input,
        Scope,
        Phase,
    >(
        reads: &'a WorthQueryCompleteApplicationReadSet<Schema, Operation, Input, Scope, Phase>,
        outputs: &'a WorthQueryApplicationOutputCorrespondenceCandidate,
    ) -> Self {
        Self {
            facts: &reads.facts,
            projection: reads.application_projection.as_deref(),
            demand: reads
                .admission
                .allowed_graph_contract()
                .aftermath()
                .and_then(|aftermath| aftermath.mechanism())
                .and_then(|mechanism| match mechanism {
                    InstalledCorrectionMechanism::RecordedInverse(inverse) => {
                        Some(inverse.preimage_demand())
                    }
                    _ => None,
                }),
            outputs,
        }
    }
    fn field_for_record<Schema>(
        &self,
        target: EntityId,
        locus: &DeclaredPreImageLocus<Schema>,
    ) -> Option<&worth_foundational::facade::AspectValue> {
        let installed = self.demand?.loci().iter().find(|installed| {
            installed.entity() == locus.entity()
                && installed.aspect() == locus.aspect()
                && installed.field() == locus.field()
        })?;
        let [ordinal] = self.projection?.preimage_ordinals(target, installed) else {
            return None;
        };
        match self.facts.get(*ordinal)? {
            Fact::Field { value, .. } => Some(value),
            _ => None,
        }
    }
}
impl<Schema> ApplicationCorrectionPreImage<Schema> for CoupledPreImage<'_> {
    fn field(
        &self,
        locus: &DeclaredPreImageLocus<Schema>,
    ) -> Option<&worth_foundational::facade::AspectValue> {
        let mut selected = None;
        for (_, _, entity, reference) in self.outputs.native_publication_roles() {
            let EntityReference::Existing(id) = reference else {
                continue;
            };
            if entity != locus.entity() {
                continue;
            }
            if let Some(value) = self.field_for_record(*id, locus) {
                match selected {
                    Some((prior, _)) if prior != *id => return None,
                    _ => selected = Some((*id, value)),
                }
            }
        }
        selected.map(|(_, value)| value)
    }
    fn field_for_output(
        &self,
        role: &ApplicationMutationOutputRoleDescriptor,
        locus: &DeclaredPreImageLocus<Schema>,
    ) -> Option<&worth_foundational::facade::AspectValue> {
        if role.entity() != locus.entity() {
            return None;
        }
        let mut targets =
            self.outputs
                .native_publication_roles()
                .filter(|(name, posture, entity, _)| {
                    *name == role.name() && *posture == role.posture() && *entity == role.entity()
                });
        let (_, _, _, EntityReference::Existing(id)) = targets.next()? else {
            return None;
        };
        if targets.next().is_some() {
            return None;
        }
        self.field_for_record(*id, locus)
    }
}
