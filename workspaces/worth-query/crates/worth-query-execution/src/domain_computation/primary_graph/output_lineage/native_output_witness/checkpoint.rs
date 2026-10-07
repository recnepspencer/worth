//! Recover the original output expectations, never expectations from a new head.

use super::*;

#[cfg(test)]
mod tests;

impl SealedNativeOutputWitness {
    /// The original output revisions and the decodable producer observations
    /// must both hold at the authentic recovered Native snapshot. Unsupported
    /// fact probes conservatively require ordinary Fresh execution.
    pub(in crate::domain_computation::primary_graph) fn checkpoint_facts_current_in(
        &self,
        relational: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        facts: &[Fact],
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, CompanionPreflightStop> {
        if !self.unchanged_in(relational, snapshot, admission)? {
            return Ok(false);
        }
        for fact in facts {
            admission.charge_external_work(1)?;
            if matches!(fact, Fact::IndexedEntitySelection { .. }) {
                let available = admission.remaining_work();
                let mut remaining = available;
                let current = crate::domain_computation::primary_graph::application_attempt::indexed_selection_currentness(
                    fact, relational, snapshot, &mut remaining,
                );
                // Debit native examination even on lookup denial. The owner
                // narrows the probe before executing it, so this stays bounded.
                admission.charge_external_work(
                    u64::try_from(available - remaining).map_err(|_| overflow())?,
                )?;
                match current {
                    Ok(current) => {
                        if !current {
                            return Ok(false);
                        }
                    }
                    Err(crate::domain_computation::primary_graph::application_attempt::WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded) => {
                        // A complete lookup requires at least one more unit
                        // than this admission can supply.
                        let maximum = admission.charged_work()
                            .checked_add(u64::try_from(admission.remaining_work()).map_err(|_| overflow())?)
                            .ok_or_else(overflow)?;
                        return Err(CompanionPreflightStop::WorkExhausted {
                            required: maximum.checked_add(1).ok_or_else(overflow)?,
                            maximum,
                        });
                    }
                    Err(_) => return Ok(false),
                }
                continue;
            }
            let work = match fact {
                Fact::SourceEntity { .. }
                | Fact::SourceFieldRevision { .. }
                | Fact::Entity { .. }
                | Fact::SourceAdjacencyRevision { .. } => 1,
                Fact::SourceAspectRevision { aspect, .. } => {
                    admission.charge_external_work(1)?;
                    aspect.as_str().len().checked_add(1).ok_or_else(overflow)?
                }
                _ => return Ok(false),
            };
            admission.charge_external_work(u64::try_from(work).map_err(|_| overflow())?)?;
            let current = fact.source_currentness_in(relational, snapshot, work);
            match current {
                Ok((true, actual)) if actual <= work => {}
                _ => return Ok(false),
            }
        }
        Ok(true)
    }

    /// Authenticated facts must cover every installed aspect of every exact
    /// output role. Producer-only legacy payloads cannot invent missing proof.
    pub(in crate::domain_computation::primary_graph) fn from_checkpoint_facts(
        correspondence: &WorthQueryApplicationOutputCorrespondence,
        layout: &WorthQueryPrimaryGraphLayout,
        facts: &[Fact],
        owner: &SourceInvalidationOwner,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<Arc<OnceLock<Self>>>, CompanionPreflightStop> {
        admission.charge_external_work(5)?;
        let role_count = correspondence.native_witness_roles().len();
        if role_count == 0 {
            return Ok(None);
        }
        let mut aspect_count = 0usize;
        let mut name_bytes = 0u64;
        for (role, posture, name, _) in correspondence.native_witness_roles() {
            admission.charge_external_work(1)?;
            if posture == WorthQueryApplicationOutputPosture::Retire {
                return Ok(None);
            }
            prepay_catalog(layout, name, admission)?;
            let Some(_) = layout.entity_kind(name) else {
                return Ok(None);
            };
            name_bytes = name_bytes
                .checked_add(role.len() as u64)
                .and_then(|n| n.checked_add(name.len() as u64))
                .ok_or_else(overflow)?;
            for aspect in layout.native_output_aspects(name) {
                admission.charge_external_work(1)?;
                aspect_count = aspect_count.checked_add(1).ok_or_else(overflow)?;
                name_bytes = name_bytes
                    .checked_add(aspect.as_str().len() as u64)
                    .ok_or_else(overflow)?;
            }
        }
        let backing = role_count
            .checked_mul(size_of::<RoleProbe>())
            .and_then(|n| n.checked_add(aspect_count.checked_mul(size_of::<AspectProbe>())?))
            .and_then(|n| u64::try_from(n).ok())
            .and_then(|n| n.checked_add(name_bytes))
            .ok_or_else(overflow)?;
        let visits = role_count.checked_add(aspect_count).ok_or_else(overflow)? as u64;
        let retained = owner.retain_native_output_witness::<Self>(
            backing,
            name_bytes.checked_add(visits).ok_or_else(overflow)?,
            admission,
        )?;
        let mut roles = Vec::with_capacity(role_count);
        let mut aspects = Vec::with_capacity(aspect_count);
        for (role, posture, name, entity) in correspondence.native_witness_roles() {
            prepay_catalog(layout, name, admission)?;
            let kind = layout
                .entity_kind(name)
                .expect("validated checkpoint output kind");
            if !checkpoint_entity_matches(facts, entity, kind, admission)? {
                return Ok(None);
            }
            let first_aspect = aspects.len();
            for aspect in layout.native_output_aspects(name) {
                let Some(revision) = checkpoint_aspect_revision(facts, entity, aspect, admission)?
                else {
                    return Ok(None);
                };
                aspects.push(AspectProbe {
                    aspect: aspect.clone(),
                    revision: Some(revision),
                });
            }
            roles.push(RoleProbe {
                role: role.to_owned(),
                entity_name: name.to_owned(),
                kind,
                posture,
                retirement: None,
                entity: Some(entity),
                first_aspect,
                end_aspect: aspects.len(),
            });
        }
        let cell = Arc::new(OnceLock::new());
        cell.set(Self {
            roles,
            aspects,
            _retained: retained,
        })
        .expect("new checkpoint witness cell is empty");
        Ok(Some(cell))
    }
}

fn prepay_catalog(
    layout: &WorthQueryPrimaryGraphLayout,
    name: &str,
    admission: &mut InvalidationEditAdmission,
) -> Result<(), CompanionPreflightStop> {
    admission.charge_external_work(3)?;
    let lookup = layout
        .native_output_lookup_work(name)
        .ok_or_else(overflow)?;
    admission.charge_external_work(lookup)
}

fn checkpoint_entity_matches(
    facts: &[Fact],
    entity: EntityId,
    expected: KindId,
    admission: &mut InvalidationEditAdmission,
) -> Result<bool, CompanionPreflightStop> {
    let mut found = false;
    for fact in facts {
        admission.charge_external_work(1)?;
        if let Fact::Entity { entity_id, kind } = fact {
            admission.charge_external_work((size_of::<EntityId>() * 2) as u64)?;
            if *entity_id == entity {
                admission.charge_external_work((size_of::<KindId>() * 2) as u64)?;
                if *kind != expected {
                    return Ok(false);
                }
                found = true;
            }
        }
    }
    Ok(found)
}

fn checkpoint_aspect_revision(
    facts: &[Fact],
    entity: EntityId,
    expected: &AspectKey,
    admission: &mut InvalidationEditAdmission,
) -> Result<Option<Option<u64>>, CompanionPreflightStop> {
    let mut found = None;
    for fact in facts {
        admission.charge_external_work(1)?;
        let Fact::SourceAspectRevision {
            entity_id,
            aspect,
            native_revision,
        } = fact
        else {
            continue;
        };
        admission.charge_external_work((size_of::<EntityId>() * 2) as u64)?;
        if *entity_id != entity {
            continue;
        }
        let comparison = aspect
            .as_str()
            .len()
            .checked_add(expected.as_str().len())
            .and_then(|n| n.checked_add(1))
            .ok_or_else(overflow)?;
        admission.charge_external_work(comparison as u64)?;
        if aspect == expected {
            admission.charge_external_work((size_of::<Option<u64>>() * 2 + 1) as u64)?;
            if found.is_some_and(|revision| revision != *native_revision) {
                return Ok(None);
            }
            found = Some(*native_revision);
        }
    }
    Ok(found)
}
