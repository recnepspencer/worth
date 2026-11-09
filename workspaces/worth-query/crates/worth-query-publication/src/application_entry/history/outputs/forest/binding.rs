//! Configure original trees before navigation; additions preserve one opaque mode.
use super::*;
use crate::application_entry::mutation::program_output_continuation::ProgramOutputContinuationFactory;
use worth_query_declaration::facade::application_query::ApplicationQueryIntent;

impl<'runtime, S, H, Payload> WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload>
where
    S: ApplicationSchema + 'static,
    H: ApplicationHistoryBinding<S>,
    Payload: WorthQueryHistoryPayloadMode<S, H>,
{
    /// Configure a graph document's first original tree. Further roots and source
    /// demands can be appended at runtime with `with_output_tree`.
    pub fn with_output_trees<P, R>(
        self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        application: &WorthQueryProgramApplicationRuntime<S, P>,
        demand: Demand<S, R>,
        controls: crate::application_entry::WorthQueryOutputDemandControls,
    ) -> Result<
        WorthQueryApplicationHistoryDocument<
            'runtime,
            S,
            H,
            Payload,
            WorthQueryHistoryWithOutputTrees<S>,
        >,
        (Self, WorthQueryHistoryOutputDenial),
    >
    where
        P: ApplicationProgramDefinition<S>,
        R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
        Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
        Demand<S, R>: Clone,
        Value<S, R>: WorthQueryApplicationProjection<S, RootQuery<S, R>> + Clone,
        <Source<S, R> as ApplicationQueryBinding<S>>::Input:
            ApplicationQueryIntent<S, Binding = Source<S, R>>,
        <Source<S, R> as ApplicationQueryBinding<S>>::ScopeBinding: ApplicationQueryScopeResolution<
            S,
            <Source<S, R> as ApplicationQueryBinding<S>>::PrincipalIdentity,
        >,
        R::Dependents: ProgramOutputContinuationFactory<S, P, Demand<S, R>>,
    {
        let tree = match super::super::binding::capture_tree::<S, H, P, R>(
            &self.owner,
            phase,
            request,
            application,
            demand,
            controls,
        ) {
            Ok(tree) => tree,
            Err(cause) => return Err((self, cause)),
        };
        if let Err(cause) = R::Dependents::capture_history_root_recipes(
            &self.owner,
            phase,
            application,
            &tree.profile,
            &tree.demand,
        ) {
            return Err((self, cause));
        }
        let outputs = match new_collection(&self.owner, phase, tree) {
            Ok(outputs) => outputs,
            Err(cause) => return Err((self, cause)),
        };
        let Self {
            owner,
            access,
            payload,
            ..
        } = self;
        Ok(WorthQueryApplicationHistoryDocument {
            owner,
            access,
            payload,
            outputs,
        })
    }
}

impl<'runtime, S, H, Payload>
    WorthQueryApplicationHistoryDocument<
        'runtime,
        S,
        H,
        Payload,
        WorthQueryHistoryWithOutputTrees<S>,
    >
where
    S: ApplicationSchema + 'static,
    H: ApplicationHistoryBinding<S>,
    Payload: WorthQueryHistoryPayloadMode<S, H>,
{
    /// Append another actual typed program/root/source demand. This consumes no
    /// producer and retains all existing attempt and disposal owners on refusal.
    pub fn with_output_tree<P, R>(
        mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        application: &WorthQueryProgramApplicationRuntime<S, P>,
        demand: Demand<S, R>,
        controls: crate::application_entry::WorthQueryOutputDemandControls,
    ) -> Result<Self, (Self, WorthQueryHistoryOutputDenial)>
    where
        P: ApplicationProgramDefinition<S>,
        R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
        Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
        Demand<S, R>: Clone,
        Value<S, R>: WorthQueryApplicationProjection<S, RootQuery<S, R>> + Clone,
        <Source<S, R> as ApplicationQueryBinding<S>>::Input:
            ApplicationQueryIntent<S, Binding = Source<S, R>>,
        <Source<S, R> as ApplicationQueryBinding<S>>::ScopeBinding: ApplicationQueryScopeResolution<
            S,
            <Source<S, R> as ApplicationQueryBinding<S>>::PrincipalIdentity,
        >,
        R::Dependents: ProgramOutputContinuationFactory<S, P, Demand<S, R>>,
    {
        let tree = match super::super::binding::capture_tree::<S, H, P, R>(
            &self.owner,
            phase,
            request,
            application,
            demand,
            controls,
        ) {
            Ok(tree) => tree,
            Err(cause) => return Err((self, cause)),
        };
        if let Err(cause) = R::Dependents::capture_history_root_recipes(
            &self.owner,
            phase,
            application,
            &tree.profile,
            &tree.demand,
        ) {
            return Err((self, cause));
        }
        if let Err(cause) = append(&self.owner, phase, &mut self.outputs, tree) {
            return Err((self, cause));
        }
        Ok(self)
    }
}

fn new_collection<S, H, T>(
    document: &WorthQueryDocumentHistory<'_, S, H>,
    phase: &WorthQueryAdvancementPhase<'_>,
    tree: T,
) -> Result<WorthQueryHistoryWithOutputTrees<S>, WorthQueryHistoryOutputDenial>
where
    S: ApplicationSchema + 'static,
    H: ApplicationHistoryBinding<S>,
    T: OriginalTree<S> + 'static,
{
    use WorthQueryHistoryOutputDenial as Denial;
    let execution = phase
        .checked_execution_request_for(tree.runtime().product_runtime())
        .map_err(Denial::Execution)?;
    let memory = funding::reserve(
        &execution,
        std::mem::size_of::<T>()
            .checked_add(std::mem::size_of::<TreeEntry<S>>())
            .ok_or(Denial::CapacityOverflow)?,
    )?;
    funding::checkpoint(phase, tree.runtime())?;
    let profile = document
        .bind_history_output_forest(
            phase,
            &program_publication_access(),
            &[tree.profile().clone()],
        )
        .map_err(Denial::Navigation)?;
    Ok(WorthQueryHistoryWithOutputTrees {
        profile,
        recording: std::sync::Mutex::new(None),
        trees: vec![TreeEntry {
            tree: Box::new(tree),
            _memory: memory,
        }],
    })
}

fn append<S, H, T>(
    document: &WorthQueryDocumentHistory<'_, S, H>,
    phase: &WorthQueryAdvancementPhase<'_>,
    collection: &mut WorthQueryHistoryWithOutputTrees<S>,
    tree: T,
) -> Result<(), WorthQueryHistoryOutputDenial>
where
    S: ApplicationSchema + 'static,
    H: ApplicationHistoryBinding<S>,
    T: OriginalTree<S> + 'static,
{
    use WorthQueryHistoryOutputDenial as Denial;
    let execution = phase
        .checked_execution_request_for(tree.runtime().product_runtime())
        .map_err(Denial::Execution)?;
    let next = collection
        .trees
        .len()
        .checked_add(1)
        .ok_or(Denial::CapacityOverflow)?;
    let capacity = if next > collection.trees.capacity() {
        next.checked_next_power_of_two()
            .ok_or(Denial::CapacityOverflow)?
    } else {
        collection.trees.capacity()
    };
    let bytes = capacity
        .checked_sub(collection.trees.capacity())
        .and_then(|growth| growth.checked_mul(std::mem::size_of::<TreeEntry<S>>()))
        .and_then(|bytes| bytes.checked_add(std::mem::size_of::<T>()))
        .ok_or(Denial::CapacityOverflow)?;
    let memory = funding::reserve(&execution, bytes)?;
    let _temporary = funding::reserve(
        &execution,
        next.checked_mul(std::mem::size_of::<WorthQueryHistoryOutputProfile>())
            .ok_or(Denial::CapacityOverflow)?,
    )?;
    let mut profiles = Vec::with_capacity(next);
    for entry in &collection.trees {
        funding::checkpoint(phase, entry.tree.runtime())?;
        profiles.push(entry.tree.profile().clone());
    }
    profiles.push(tree.profile().clone());
    let profile = document
        .bind_history_output_forest(phase, &program_publication_access(), &profiles)
        .map_err(Denial::Navigation)?;
    if capacity > collection.trees.capacity() {
        collection
            .trees
            .reserve_exact(capacity - collection.trees.len());
    }
    collection.trees.push(TreeEntry {
        tree: Box::new(tree),
        _memory: memory,
    });
    collection.profile = profile;
    Ok(())
}
