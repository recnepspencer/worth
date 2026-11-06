//! A graph selection restores producer custody; its rows move with it, never the head's.
use super::*;
mod accepted;
pub use accepted::HistoricalAcceptedOrigin;
use worth_relational::facade::mvcc::{
    CompanionPreflightStop as Stop, RelationalRequiredOutputPublication,
};
mod denial;
pub use denial::{HistoricalTransportDenial, WorthQueryHistoryPublicationCorrespondenceMismatch};
mod coordinate;
pub use coordinate::HistoricalSelectionCoordinate;

/// The selected step's own row for one route that Native restores.
struct SelectedRow {
    cell: Arc<OnceLock<RecordedOutput>>,
    route: RelationalRequiredOutputPublication,
    locator: Option<native_prior_checkpoint::NativePriorCheckpointLocator>,
    witness: Option<Arc<OnceLock<SealedNativeOutputWitness>>>,
}

/// Every restored route resolved to its exact selected row before World moves.
/// Recording after the selection commit allocates only what this funds.
pub struct PreparedHistoricalTransport {
    rows: Vec<SelectedRow>,
    selected: worth_relational::facade::mvcc::RelationalBranchObservation,
    _funding: Option<worth_execution::ExecutionMemoryReservation>,
}

impl WorthQueryApplicationOutputLineage {
    /// Resolve each restored route at the selected step's coordinate. The row
    /// must hold that route's own Native publication (or its recovered
    /// locator); the current head's row, a fact-less prior or an ambiguous
    /// pair is a typed stop. O(routes × (bindings + fork depth)).
    pub(in crate::domain_computation::primary_graph) fn prepare_historical_transport<'a>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime_authority: u64,
        schema: &ApplicationSchemaBindingIdentity,
        layout: &crate::domain_computation::primary_graph::schema_layout::WorthQueryPrimaryGraphLayout,
        selected: &worth_runtime_world::facade::ProductBranchObservation,
        routes: impl ExactSizeIterator<Item = &'a RelationalRequiredOutputPublication>,
    ) -> Result<PreparedHistoricalTransport, HistoricalTransportDenial> {
        let count = routes.len();
        let bytes = (count as u64).saturating_mul(
            (std::mem::size_of::<RecordedOutput>() + std::mem::size_of::<SelectedRow>() + 256)
                as u64,
        );
        let funding = crate::domain_computation::primary_graph::structural_walk::memory::reserve(
            execution,
            execution.has_accounting(),
            bytes,
        )
        .map_err(Stop::Execution)?;
        let mut rows = Vec::with_capacity(count);
        for route in routes {
            let meaning = route.meaning();
            let bindings = self
                .output_families
                .get(meaning.family.as_str())
                .ok_or(Stop::SelectedSourceMismatch)?
                .iter()
                .map(|(binding, _)| *binding)
                .collect::<Vec<_>>();
            let scope = crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(meaning.source_scope);
            let candidates = self.retained_output_candidates_metered(
                runtime_authority,
                schema,
                scope,
                selected.lifecycle_incarnation(),
                selected.reference_generation().get(),
                &bindings,
                meaning.source_partition,
                execution,
            )?;
            let mut found = None;
            for candidate in candidates {
                execution
                    .consult()
                    .map_err(|stop| Stop::Execution(stop.into()))?;
                let pin = &candidate.settlement_identity;
                let coordinate = pin.coordinate();
                let cell = self
                    .by_source
                    .get(pin.source())
                    .and_then(|rows| rows.get(&coordinate.occurrence))
                    .and_then(|rows| rows.get(&coordinate.generation))
                    .and_then(|rows| rows.get(pin.slot()))
                    .expect("a retained candidate owns its exact row");
                let row = cell.get().ok_or(Stop::SelectedSourceMismatch)?;
                let origin = match &row.performed_origin {
                    Some(alias) => alias.get().ok_or(Stop::SelectedSourceMismatch)?,
                    None => row,
                };
                let owns_route = origin
                    .required_publication
                    .as_ref()
                    .is_some_and(|publication| publication.same_origin(route))
                    || origin
                        .native_prior_checkpoint
                        .as_ref()
                        .is_some_and(|prior| {
                            prior.producer() == route.producer()
                                && prior.source() == meaning.source_epoch
                        });
                if owns_route {
                    if !origin
                        .correspondence
                        .matches_native_publication(execution, route)?
                    {
                        return Err(HistoricalTransportDenial::PublicationCorrespondence(
                            WorthQueryHistoryPublicationCorrespondenceMismatch::new(
                                execution, route,
                            )?,
                        ));
                    }
                    if found.is_some() {
                        return Err(Stop::SelectedSourceMismatch.into());
                    }
                    let locator = match &origin.native_prior_checkpoint {
                        Some(locator) => Some(
                            locator
                                .try_retain()
                                .ok_or(Stop::PreparationMemoryCounterOverflow)?,
                        ),
                        None => None,
                    };
                    let witness = match origin
                        .native_output_witness_cell()
                        .filter(|cell| cell.get().is_some())
                    {
                        Some(witness) => Some(Arc::clone(witness)),
                        None if matches!(
                            origin.source_identity,
                            Some(RecordedSourceIdentity::Checkpoint(_))
                        ) =>
                        {
                            match origin
                                .observed_source_facts()
                                .and_then(|facts| facts.for_comparison())
                            {
                                Some(facts) => {
                                    SealedNativeOutputWitness::from_historical_checkpoint_facts(
                                        execution,
                                        &origin.correspondence,
                                        layout,
                                        &facts,
                                    )?
                                }
                                None => None,
                            }
                        }
                        None => None,
                    };
                    found = Some(SelectedRow {
                        witness,
                        cell: Arc::clone(cell),
                        route: route.clone(),
                        locator,
                    });
                }
            }
            rows.push(found.ok_or(Stop::SelectedSourceMismatch)?);
        }
        Ok(PreparedHistoricalTransport {
            rows,
            selected: selected.basis().relational_basis().observation(),
            _funding: funding,
        })
    }

    /// Record each selected row again at the selection commit's coordinate.
    /// The row keeps its facts, witness, implementation, correspondence and
    /// idempotency, and names the committed HistoricalSelection publication,
    /// so a later Preserve succeeds that publication. Its new settlement has
    /// no mark row yet: the first reader verifies it in full.
    pub(in crate::domain_computation::primary_graph) fn record_historical_transport(
        &mut self,
        prepared: PreparedHistoricalTransport,
        restored: &worth_runtime_world::facade::ProductBranchObservation,
        publications: impl Iterator<Item = RelationalRequiredOutputPublication>,
    ) {
        let mut publications = publications.collect::<Vec<_>>();
        let coordinate = ProductCoordinate {
            occurrence: restored.lifecycle_incarnation(),
            generation: restored.reference_generation().get(),
        };
        self.invalidate_source_publication_coverage();
        let restored_basis = restored.basis().relational_basis().observation();
        for selected in prepared.rows {
            let row = selected.cell.get().expect("prepared rows are filled");
            let historical = Arc::new(HistoricalSelectionCoordinate {
                selected: prepared.selected.clone(),
                restored: restored_basis.clone(),
                prior: row.historical_selection.as_ref().map(Arc::clone),
            });
            let position = publications
                .iter()
                .position(|publication| {
                    publication.is_historical_selection()
                        && publication.producer() == selected.route.producer()
                        && publication.meaning() == selected.route.meaning()
                })
                .expect("the selection commit republished every prepared restored route");
            let publication = publications.swap_remove(position);
            let origin = row
                .performed_origin
                .as_ref()
                .and_then(|alias| alias.get())
                .unwrap_or(row);
            let source = row.settlement_identity.source().clone();
            let partition = row.source_partition_identity;
            assert!(
                partition.is_none_or(|partition| {
                    self.partition_index
                        .at_generation(
                            &source,
                            coordinate.occurrence,
                            coordinate.generation,
                            partition,
                        )
                        .is_none()
                }),
                "one selection commit records one row per restored route"
            );
            let generation = self
                .by_source
                .entry(source.clone())
                .or_default()
                .entry(coordinate.occurrence)
                .or_default()
                .entry(coordinate.generation)
                .or_default();
            let slot = generation.len();
            let transported = RecordedOutput {
                coupled_completion: None,
                producing_implementation: origin.producing_implementation,
                required_publication: Some(publication),
                computation_source: origin.computation_source,
                native_prior_checkpoint: selected.locator,
                performed_origin: None,
                historical_selection: Some(historical),
                consumed_outputs: Arc::clone(&origin.consumed_outputs),
                completed_handler_facts: None,
                completed_decision_reuse: None,
                prepared_input_reuse_key: None,
                native_output_witness: selected.witness.map(OnceLock::from).unwrap_or_default(),
                mutable: std::sync::Mutex::new(RecordedOutputMutable::new(
                    None,
                    row.observed_source_facts(),
                    row.mutable
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .computation
                        .shared(),
                )),
                settlement_identity: RecordedSettlementIdentity::retain(&source, coordinate, slot),
                correspondence: Arc::clone(&row.correspondence),
                source_identity: row.source_identity,
                source_partition_identity: partition,
                producer_dependency_identity: row.producer_dependency_identity,
                idempotency_key_identity: row.idempotency_key_identity,
            };
            let cell = Arc::new(OnceLock::new());
            assert!(cell.set(transported).is_ok());
            generation.push(cell);
            self.partition_index.insert(
                source,
                coordinate.occurrence,
                coordinate.generation,
                partition,
                slot,
            );
        }
        self.live_occurrences.insert(coordinate.occurrence);
    }
}
