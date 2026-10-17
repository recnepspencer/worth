//! Structural ceiling for charged reconstruction and currentness comparison.
use crate::domain_computation::primary_graph::application_checkpoint::facts::FactSequence;

use super::*;

impl SealedNativeOutputWitness {
    /// Derive from the complete original fact sequence and installed vocabulary.
    /// This does not reserve the worst case: a sparse indexed probe still spends
    /// only the entries actually examined. Calculation visits use the same meter
    /// but are outside this returned ceiling; restoration adds their own debit
    /// once. This bounds charged work, not CPU time or RSS.
    pub(in crate::domain_computation::primary_graph) fn checkpoint_comparison_work_bound(
        correspondence: &WorthQueryApplicationOutputCorrespondence,
        layout: &WorthQueryPrimaryGraphLayout,
        facts: &(impl FactSequence + ?Sized),
        admission: &mut InvalidationEditAdmission,
    ) -> Result<u64, CompanionPreflightStop> {
        let index = super::CheckpointRoleIndex::build(
            correspondence
                .native_witness_roles()
                .map(|(_, _, _, entity)| entity),
            admission,
        )?;
        let fan_out =
            u64::try_from(index.widest_entity(admission)?).map_err(|_| work_overflow())?;
        let role_count = correspondence.native_witness_roles().len();
        // Building the index, then two descents for every fact a role may name.
        let descent = super::fact_index::navigation(role_count)?;
        let roles = u64::try_from(role_count).map_err(|_| work_overflow())?;
        let lookup_descent = multiply(2, descent)?;
        let mut bound = add(5, multiply(roles, descent)?)?;
        let mut longest_aspect = 0u64;
        let mut widest_role = 0u64;
        for (role, _, name, _) in correspondence.native_witness_roles() {
            admission.charge_external_work(16)?;
            super::prepay_catalog(layout, name, admission)?;
            let lookup = layout
                .native_output_lookup_work(name)
                .ok_or_else(work_overflow)?;
            let names = u64::try_from(role.len())
                .ok()
                .and_then(|role| {
                    u64::try_from(name.len())
                        .ok()
                        .and_then(|name| role.checked_add(name))
                })
                .ok_or_else(work_overflow)?;
            // First counting visit + copy visit + currentness visit; two catalog
            // passes; owned names.
            bound = add(bound, add(add(3, names)?, multiply(2, add(3, lookup)?)?)?)?;
            let mut role_aspects = 0u64;
            for aspect in layout.native_output_aspects(name) {
                admission.charge_external_work(8)?;
                let bytes = u64::try_from(aspect.as_str().len()).map_err(|_| work_overflow())?;
                longest_aspect = longest_aspect.max(bytes);
                role_aspects = add(role_aspects, 1)?;
                // Counting + copy + currentness visits, UTF-8 name ownership and
                // currentness lookup, plus the final coverage visit.
                bound = add(bound, add(4, multiply(2, bytes)?)?)?;
            }
            widest_role = widest_role.max(role_aspects);
        }
        // The final coverage visit of every role.
        bound = add(bound, roles)?;
        let kind_match = (1 + size_of::<KindId>() * 2) as u64;
        let revision_match = (size_of::<Option<u64>>() * 2 + 1) as u64;
        for fact in facts.iter() {
            // The fixed inspection and checked arithmetic for one fact.
            admission.charge_external_work(8)?;
            // The fact's currentness probe, plus its entity lookup when it
            // carries an original output expectation.
            let work = match fact {
                Fact::SourceEntity { .. } => 2,
                Fact::Entity { .. } => {
                    add(add(2, lookup_descent)?, multiply(fan_out, kind_match)?)?
                }
                // Inspection, native revision probe, and a liveness probe when
                // the native revision has no answer.
                Fact::SourceFieldRevision { .. } | Fact::SourceAdjacencyRevision { .. } => 3,
                Fact::SourceAspectRevision { aspect, .. } => {
                    let bytes =
                        u64::try_from(aspect.as_str().len()).map_err(|_| work_overflow())?;
                    // Each role naming the entity compares every installed
                    // aspect it owns; the matching one compares revisions.
                    let comparisons = multiply(widest_role, add(add(bytes, longest_aspect)?, 1)?)?;
                    let per_role = add(add(1, comparisons)?, revision_match)?;
                    add(
                        add(add(bytes, 3)?, lookup_descent)?,
                        multiply(fan_out, per_role)?,
                    )?
                }
                Fact::IndexedEntitySelection {
                    candidate_limit, ..
                } => {
                    // Result cardinality cannot bound examination. The authentic
                    // retained lookup limit bounds entries even for sparse sets.
                    add(
                        u64::try_from(*candidate_limit).map_err(|_| work_overflow())?,
                        2,
                    )?
                }
                _ => 1, // Unsupported facts stop after their inspection charge.
            };
            // Its one-pass original-fact visit precedes the probe.
            bound = add(bound, add(1, work)?)?;
        }
        Ok(bound)
    }
}

fn add(left: u64, right: u64) -> Result<u64, CompanionPreflightStop> {
    left.checked_add(right).ok_or_else(work_overflow)
}

fn multiply(left: u64, right: u64) -> Result<u64, CompanionPreflightStop> {
    left.checked_mul(right).ok_or_else(work_overflow)
}

fn work_overflow() -> CompanionPreflightStop {
    CompanionPreflightStop::WorkCounterOverflow
}
