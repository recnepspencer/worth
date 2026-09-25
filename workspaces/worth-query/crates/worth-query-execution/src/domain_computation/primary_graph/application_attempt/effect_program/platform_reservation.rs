//! Exact reservation for platform-owned effects that do not pass through an
//! application's typed candidate writer.

use worth_query_declaration::facade::{
    application_operation::{
        ApplicationCandidateCardinalityCeiling, ApplicationCandidateRequirements,
        ApplicationCandidateResourceCeiling,
    },
    domain_computation::{WorthQueryResourceDimension, WorthQuerySemanticScaleAxis},
};

use super::{
    candidate_retained_representation as representation, CandidateItemKind,
    WorthQueryCandidateReservation, WorthQueryCandidateValidatorWorkAdmission,
};
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationRealizedEffect, WorthQueryCompleteApplicationReadSet,
    WorthQueryProjectedApplicationMutation,
};

pub(in crate::domain_computation::primary_graph) fn admit_platform_effects<
    Schema,
    Operation,
    Input,
    Scope,
>(
    read_set: &WorthQueryCompleteApplicationReadSet<
        Schema,
        Operation,
        Input,
        Scope,
        WorthQueryProjectedApplicationMutation,
    >,
    demand: PlatformEffectDemand,
) -> Result<PlatformEffectReservation, WorthQueryApplicationAttemptDenial> {
    let validator_work = demand.validator_work()?;
    let requirements = ApplicationCandidateRequirements::fixed_shape(
        ApplicationCandidateCardinalityCeiling::fixed(
            demand.creates,
            demand.deletes,
            demand.links,
            demand.unlinks,
            demand.writes,
            demand.emits,
        ),
        ApplicationCandidateResourceCeiling::bounded(demand.retained_bytes, validator_work),
    );
    let envelope = read_set
        .admission
        .allowed_graph_contract()
        .execution_strategy()
        .expect("installed application operation has exactly one execution strategy")
        .envelope();
    let declared = read_set
        .admission
        .allowed_graph_contract()
        .platform_candidate_ceiling()
        .ok_or_else(|| {
            WorthQueryApplicationAttemptDenial::new(
                WorthQueryApplicationAttemptDenialKind::CandidateCapacityExceeded,
                "platform effect operation has no installed candidate contract",
            )
        })?;
    let reservation = WorthQueryCandidateReservation::admit(
        requirements,
        declared,
        envelope.scale_ceiling(WorthQuerySemanticScaleAxis::CandidateItems),
        envelope
            .resource_ceiling(WorthQueryResourceDimension::CandidateRetainedRepresentationBytes),
        envelope.scale_ceiling(WorthQuerySemanticScaleAxis::WorkItems),
    )?;
    Ok(PlatformEffectReservation { reservation })
}

#[derive(Default)]
pub(in crate::domain_computation::primary_graph) struct PlatformEffectDemand {
    creates: usize,
    deletes: usize,
    links: usize,
    unlinks: usize,
    writes: usize,
    emits: usize,
    retained_bytes: usize,
}

impl PlatformEffectDemand {
    pub(in crate::domain_computation::primary_graph) fn observe(
        &mut self,
        effect: &WorthQueryApplicationRealizedEffect,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        observe(self, effect)
    }

    fn total_items(&self) -> Result<usize, WorthQueryApplicationAttemptDenial> {
        [
            self.creates,
            self.deletes,
            self.links,
            self.unlinks,
            self.writes,
            self.emits,
        ]
        .into_iter()
        .try_fold(0, add)
    }

    fn validator_work(&self) -> Result<usize, WorthQueryApplicationAttemptDenial> {
        // Query-owned platform facts never execute application invariant slots.
        // Reserve the exact candidate breadth consumed by the provider's
        // relational validation rather than importing unrelated application
        // invariant budgets from the authorizing operation.
        self.total_items().map(|work| work.max(1))
    }
}

fn observe(
    demand: &mut PlatformEffectDemand,
    effect: &WorthQueryApplicationRealizedEffect,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    match effect {
        WorthQueryApplicationRealizedEffect::CreateEntity { key, fields, .. } => {
            demand.creates = add(demand.creates, 1)?;
            demand.writes = add(demand.writes, fields.len())?;
            demand.retained_bytes = add(
                demand.retained_bytes,
                representation::created_entity(key, "").ok_or_else(overflow)?,
            )?;
            for (locator, value) in fields {
                demand.retained_bytes = add(
                    demand.retained_bytes,
                    representation::field_entry(locator, value, true).ok_or_else(overflow)?,
                )?;
            }
        }
        WorthQueryApplicationRealizedEffect::UpdateEntity { entity, fields, .. } => {
            demand.writes = add(demand.writes, fields.len())?;
            if !fields.is_empty() {
                demand.retained_bytes = add(demand.retained_bytes, entity.len())?;
            }
            for (locator, value) in fields {
                demand.retained_bytes = add(
                    demand.retained_bytes,
                    representation::field_entry(locator, value, true).ok_or_else(overflow)?,
                )?;
            }
        }
        WorthQueryApplicationRealizedEffect::PatchOptionalEntityFields {
            entity, fields, ..
        } => {
            demand.writes = add(demand.writes, fields.len())?;
            if !fields.is_empty() {
                demand.retained_bytes = add(demand.retained_bytes, entity.len())?;
            }
            for (locator, write) in fields {
                demand.retained_bytes = add(
                    demand.retained_bytes,
                    representation::optional_field_entry(
                        locator,
                        &write.contract,
                        write.value.as_ref(),
                        true,
                    )
                    .ok_or_else(overflow)?,
                )?;
            }
        }
        WorthQueryApplicationRealizedEffect::DeleteEntity { .. } => {
            demand.deletes = add(demand.deletes, 1)?;
        }
        WorthQueryApplicationRealizedEffect::CreateRelation { key, from, to, .. } => {
            demand.links = add(demand.links, 1)?;
            demand.retained_bytes = add(
                demand.retained_bytes,
                representation::relation(key, from, to).ok_or_else(overflow)?,
            )?;
        }
        WorthQueryApplicationRealizedEffect::DeleteRelation { .. } => {
            demand.unlinks = add(demand.unlinks, 1)?;
        }
        WorthQueryApplicationRealizedEffect::Emit(emission) => {
            demand.emits = add(demand.emits, 1)?;
            demand.retained_bytes = add(
                demand.retained_bytes,
                emission
                    .candidate_retained_representation_bytes()
                    .ok_or_else(overflow)?,
            )?;
        }
    }
    Ok(())
}

pub(in crate::domain_computation::primary_graph) struct PlatformEffectReservation {
    reservation: WorthQueryCandidateReservation,
}

impl PlatformEffectReservation {
    pub(in crate::domain_computation::primary_graph) fn materialize(
        mut self,
        effects: &[WorthQueryApplicationRealizedEffect],
    ) -> Result<WorthQueryCandidateValidatorWorkAdmission, WorthQueryApplicationAttemptDenial> {
        for effect in effects {
            match effect {
                WorthQueryApplicationRealizedEffect::CreateEntity { key, fields, .. } => {
                    self.reservation.charge_retained_representation(
                        CandidateItemKind::Create,
                        representation::created_entity(key, "").ok_or_else(overflow)?,
                        0,
                    )?;
                    for (locator, value) in fields {
                        self.reservation.charge_retained_representation(
                            CandidateItemKind::Write,
                            representation::field_entry(locator, value, true)
                                .ok_or_else(overflow)?,
                            0,
                        )?;
                    }
                }
                WorthQueryApplicationRealizedEffect::UpdateEntity { entity, fields, .. } => {
                    let mut first = true;
                    for (locator, value) in fields {
                        let mut retained = representation::field_entry(locator, value, true)
                            .ok_or_else(overflow)?;
                        if first {
                            retained = add(retained, entity.len())?;
                            first = false;
                        }
                        self.reservation.charge_retained_representation(
                            CandidateItemKind::Write,
                            retained,
                            0,
                        )?;
                    }
                }
                WorthQueryApplicationRealizedEffect::PatchOptionalEntityFields {
                    entity,
                    fields,
                    ..
                } => {
                    let mut first = true;
                    for (locator, write) in fields {
                        let mut retained = representation::optional_field_entry(
                            locator,
                            &write.contract,
                            write.value.as_ref(),
                            true,
                        )
                        .ok_or_else(overflow)?;
                        if first {
                            retained = add(retained, entity.len())?;
                            first = false;
                        }
                        self.reservation.charge_retained_representation(
                            CandidateItemKind::Write,
                            retained,
                            0,
                        )?;
                    }
                }
                WorthQueryApplicationRealizedEffect::DeleteEntity { .. } => {
                    self.reservation.charge(CandidateItemKind::Delete)?;
                }
                WorthQueryApplicationRealizedEffect::CreateRelation { key, from, to, .. } => {
                    self.reservation.charge_retained_representation(
                        CandidateItemKind::Link,
                        representation::relation(key, from, to).ok_or_else(overflow)?,
                        0,
                    )?;
                }
                WorthQueryApplicationRealizedEffect::DeleteRelation { .. } => {
                    self.reservation.charge(CandidateItemKind::Unlink)?;
                }
                WorthQueryApplicationRealizedEffect::Emit(emission) => {
                    self.reservation.charge_retained_representation(
                        CandidateItemKind::Emit,
                        emission
                            .candidate_retained_representation_bytes()
                            .ok_or_else(overflow)?,
                        0,
                    )?;
                }
            }
        }
        Ok(self.reservation.validator_work_admission())
    }
}

fn add(left: usize, right: usize) -> Result<usize, WorthQueryApplicationAttemptDenial> {
    left.checked_add(right).ok_or_else(overflow)
}

fn overflow() -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(
        WorthQueryApplicationAttemptDenialKind::CandidateCapacityExceeded,
        "platform candidate resource demand overflowed",
    )
}
