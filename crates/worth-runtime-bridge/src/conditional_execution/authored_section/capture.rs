//! Capture installed declaration meaning, preserving performed allocation choices.
use super::{
    definition, dependency::BridgeAuthoredDependencyDescription, fields::Writer, framing, targets,
    BridgeAuthoredSection, BridgeAuthoredSectionDenial as D,
};
use crate::conditional_execution::BridgeInstalledConditionalLowering;
use crate::correspondence::{
    AllocationSlotIdentity, BridgeSemanticClaimOrigin, BridgeSignalSlotRequest,
};
use crate::facade::RuntimeBridge;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{
    BridgeSemanticClaimReference, DurableSignalBranchIdentity,
    DurableSignalConditionalDefinitionOccurrence, DurableSignalNodeIdentity,
    SignalDefinitionTransitionIdentity,
};
use worth_signal::facade::durability::SignalBridgeAuthoredSection;
use worth_signal::facade::{NodeId, SignalGraph};

pub(in crate::conditional_execution) struct Capture<'a> {
    pub(in crate::conditional_execution) bridge: &'a RuntimeBridge,
    pub(in crate::conditional_execution) graph: &'a SignalGraph,
    pub(in crate::conditional_execution) branch: DurableSignalBranchIdentity,
    pub(in crate::conditional_execution) transition: SignalDefinitionTransitionIdentity,
    pub(in crate::conditional_execution) occurrences:
        &'a [DurableSignalConditionalDefinitionOccurrence],
}
impl Capture<'_> {
    pub(in crate::conditional_execution) fn capture(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        definitions: &[&BridgeInstalledConditionalLowering],
    ) -> Result<BridgeAuthoredSection, D> {
        if self.transition.owner() != self.branch.owner() || self.transition.ordinal() == 0 {
            return Err(D::ForeignBranch);
        }
        let allocations = self
            .bridge
            .correspondence_allocations
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let source = framing::encode(execution, &mut |out| {
            out.branch(self.branch)?;
            out.byte(u8::from(self.bridge.authoritative_source_profile.is_some()))?;
            if let Some(profile) = &self.bridge.authoritative_source_profile {
                out.text(profile.adapter_semantic_identity())?;
            }
            out.record(&mut |out| {
                out.number(self.bridge.semantic_dependency_registry.admitted_count() as u64)?;
                for (key, registration) in
                    self.bridge.semantic_dependency_registry.admitted_entries()
                {
                    out.record(&mut |out| {
                        out.claim(self.reference_key(key)?)?;
                        let meaning = BridgeAuthoredDependencyDescription::capture(
                            execution,
                            registration.dependency(),
                        )?;
                        out.json(meaning.data())?;
                        out.record(&mut |out| {
                            out.number(registration.targets.len() as u64)?;
                            for target in &registration.targets {
                                out.record(&mut |out| {
                                    self.node(out, target.node)?;
                                    out.text(&target.partition.0)?;
                                    out.text(target.aspect_registration_id.as_str())?;
                                    targets::write_slot(
                                        out,
                                        match &target.slot {
                                            BridgeSignalSlotRequest::Allocate => {
                                                AllocationSlotIdentity::Allocate
                                            }
                                            BridgeSignalSlotRequest::Exact(aspect) => {
                                                AllocationSlotIdentity::Exact(aspect.aspect())
                                            }
                                        },
                                    )
                                })?;
                            }
                            Ok(())
                        })
                    })?;
                }
                Ok(())
            })?;
            out.record(&mut |out| {
                out.number(allocations.authored_allocations().count() as u64)?;
                for allocation in allocations.authored_allocations() {
                    if allocation.graph_instance
                        != self.graph.installed_graph_capability().graph_instance_id()
                    {
                        return Err(D::SignalDefinitionMismatch);
                    }
                    out.record(&mut |out| {
                        out.claim(self.reference(allocation.owner)?)?;
                        self.node(out, allocation.node)?;
                        out.text(&allocation.partition.0)?;
                        out.text(allocation.mapping.as_str())?;
                        targets::write_slot(out, allocation.slot)?;
                        out.byte(u8::try_from(allocation.aspect.index()).map_err(|_| D::Malformed)?)
                    })?;
                }
                Ok(())
            })?;
            out.record(&mut |out| {
                out.number(definitions.len() as u64)?;
                for lowering in definitions {
                    let occurrence = self
                        .occurrences
                        .iter()
                        .find(|row| {
                            row.node().branch() == self.branch
                                && row.node().index() == lowering.signal_node().index()
                                && row.node().generation() == lowering.signal_node().generation()
                        })
                        .copied()
                        .ok_or(D::SignalDefinitionMismatch)?;
                    out.record(&mut |out| {
                        let claims =
                            framing::raw(execution, &mut |claims| self.claims(claims, lowering))?;
                        definition::write(out, occurrence, lowering, claims.bytes())
                    })?;
                }
                Ok(())
            })
        })?;
        let source =
            SignalBridgeAuthoredSection::from_bridge_bytes(source).map_err(D::SignalCarrier)?;
        super::layout::Layout::read(source.as_bytes())?.inspect()?;
        Ok(BridgeAuthoredSection {
            source,
            branch: self.branch,
            definition_count: definitions.len(),
        })
    }
    fn node(&self, out: &mut Writer<'_>, node: NodeId) -> Result<(), D> {
        out.node(DurableSignalNodeIdentity::from_recorded(
            self.branch,
            node.index(),
            node.generation(),
        ))
    }
    fn reference(
        &self,
        origin: &BridgeSemanticClaimOrigin,
    ) -> Result<BridgeSemanticClaimReference, D> {
        match origin {
            BridgeSemanticClaimOrigin::Recorded(reference) => Ok(*reference),
            BridgeSemanticClaimOrigin::Memory(key) => self.reference_key(key),
        }
    }
    fn reference_key(&self, key: &str) -> Result<BridgeSemanticClaimReference, D> {
        let (ordinal, row) = self
            .bridge
            .semantic_dependency_registry
            .admitted_entry(key)
            .ok_or(D::SignalDefinitionMismatch)?;
        Ok(row
            .recorded_claim
            .unwrap_or(BridgeSemanticClaimReference::from_recorded(
                self.transition,
                ordinal.try_into().map_err(|_| D::SizeLimit)?,
            )))
    }
    fn claims(
        &self,
        out: &mut Writer<'_>,
        lowering: &BridgeInstalledConditionalLowering,
    ) -> Result<(), D> {
        out.number(lowering.correspondences.len() as u64)?;
        for correspondence in lowering.correspondences.iter() {
            out.record(&mut |out| {
                out.claim(self.correspondence_reference(correspondence)?)?;
                out.record(&mut |out| {
                    out.number(correspondence.authored_targets().len() as u64)?;
                    for target in correspondence.authored_targets() {
                        out.record(&mut |out| {
                            self.node(out, target.node)?;
                            out.text(&target.partition.0)?;
                            let mapping = self
                                .bridge
                                .aspect_registry
                                .registrations()
                                .iter()
                                .find(|mapping| {
                                    mapping.identity_basis() == &target.mapping_identity
                                })
                                .ok_or(D::SignalDefinitionMismatch)?;
                            out.text(mapping.registration_id().as_str())?;
                            out.text(&target.mapping_identity)?;
                            out.byte(
                                u8::try_from(target.aspect.index()).map_err(|_| D::Malformed)?,
                            )?;
                            targets::write_precision(out, target.precision)?;
                            targets::write_widening(out, target.admitted_source_widening)?;
                            out.record(&mut |out| {
                                for source in &target.allocation_sources {
                                    out.claim(self.reference(source)?)?;
                                }
                                Ok(())
                            })
                        })?;
                    }
                    Ok(())
                })
            })?;
        }
        Ok(())
    }
    fn correspondence_reference(
        &self,
        correspondence: &crate::correspondence::BridgeInstalledSemanticCorrespondence,
    ) -> Result<BridgeSemanticClaimReference, D> {
        let target = correspondence
            .authored_targets()
            .first()
            .ok_or(D::SignalDefinitionMismatch)?;
        for origin in &target.allocation_sources {
            let matches = match origin {
                BridgeSemanticClaimOrigin::Memory(key) => self
                    .bridge
                    .semantic_dependency_registry
                    .admitted_entry(key)
                    .is_some_and(|(_, row)| row.dependency() == correspondence.dependency()),
                BridgeSemanticClaimOrigin::Recorded(reference) => self
                    .bridge
                    .semantic_dependency_registry
                    .admitted_entries()
                    .any(|(_, row)| {
                        row.recorded_claim == Some(*reference)
                            && row.dependency() == correspondence.dependency()
                    }),
            };
            if matches {
                return self.reference(origin);
            }
        }
        Err(D::SignalDefinitionMismatch)
    }
}
