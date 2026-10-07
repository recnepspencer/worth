//! Structural ceiling for charged reconstruction and currentness comparison.

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
        facts: &[Fact],
        admission: &mut InvalidationEditAdmission,
    ) -> Result<u64, CompanionPreflightStop> {
        let mut source_work = 0u64;
        let mut longest_aspect = 0u64;
        for fact in facts {
            // The fixed inspection and checked arithmetic for one fact.
            admission.charge_external_work(8)?;
            let work = match fact {
                Fact::SourceEntity { .. }
                | Fact::SourceFieldRevision { .. }
                | Fact::Entity { .. }
                | Fact::SourceAdjacencyRevision { .. } => 2,
                Fact::SourceAspectRevision { aspect, .. } => {
                    let bytes =
                        u64::try_from(aspect.as_str().len()).map_err(|_| work_overflow())?;
                    longest_aspect = longest_aspect.max(bytes);
                    add(bytes, 3)?
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
            source_work = add(source_work, work)?;
        }
        let fact_count = u64::try_from(facts.len()).map_err(|_| work_overflow())?;
        let entity_scan = multiply(
            fact_count,
            1 + (size_of::<EntityId>() * 2 + size_of::<KindId>() * 2) as u64,
        )?;
        // Every aspect scan visits ALL facts, including contradictory duplicates.
        let aspect_scan_base = add(
            1 + (size_of::<EntityId>() * 2 + size_of::<Option<u64>>() * 2 + 2) as u64,
            longest_aspect,
        )?;
        let mut bound = add(5, source_work)?;
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
            // passes; owned names; complete entity-kind fact scan.
            bound = add(
                bound,
                add(
                    add(add(3, names)?, multiply(2, add(3, lookup)?)?)?,
                    entity_scan,
                )?,
            )?;
            for aspect in layout.native_output_aspects(name) {
                admission.charge_external_work(8)?;
                let bytes = u64::try_from(aspect.as_str().len()).map_err(|_| work_overflow())?;
                // Counting + copy + currentness visits, UTF-8 name ownership and
                // currentness lookup, plus the complete original revision scan.
                let visits_and_names = add(3, multiply(2, bytes)?)?;
                let scan = multiply(fact_count, add(aspect_scan_base, bytes)?)?;
                bound = add(bound, add(visits_and_names, scan)?)?;
            }
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
