//! All original trees advance under one navigation and qualify at one endpoint.
use super::*;
use worth_query_execution::publication_boundary::document_history::{
    WorthQueryCompletedHistoryOutputs, WorthQueryHistoryNavigationPosture,
};

impl<S: ApplicationSchema + 'static, H: ApplicationHistoryBinding<S>>
    WorthQueryHistoryOutputsMode<S, H> for WorthQueryHistoryWithOutputTrees<S>
{
}

impl<S: ApplicationSchema + 'static, H: ApplicationHistoryBinding<S>> OutputDriver<S, H>
    for WorthQueryHistoryWithOutputTrees<S>
{
    fn with_recording<T>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        consume: impl FnOnce(Option<&mut RecordingForest<S>>) -> (T, bool),
    ) -> Result<T, WorthQueryHistoryOutputDenial> {
        self.with_recording_owner(phase, consume)
    }
    fn discard_recording(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        branch: WorthQueryProductBranch,
    ) -> Result<(), WorthQueryHistoryOutputDenial> {
        self.discard_recording_owner(request, branch)
    }
    fn release_for_branch(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        branch: WorthQueryProductBranch,
    ) -> Result<(), WorthQueryHistoryOutputDenial> {
        for entry in &self.trees {
            funding::checkpoint(phase, entry.tree.runtime())?;
            entry.tree.release(phase, branch)?;
        }
        Ok(())
    }
    fn close(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<(), WorthQueryHistoryOutputDenial> {
        self.close_recording_owner(phase)?;
        for entry in &mut self.trees {
            funding::checkpoint(phase, entry.tree.runtime())?;
            entry.tree.close(phase)?;
        }
        Ok(())
    }
    fn discard(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        navigation: &WorthQueryDocumentHistoryNavigation<'_, S>,
    ) -> Result<(), WorthQueryHistoryNavigationAdvanceDenial> {
        navigation
            .discard_after_output_cleanup(phase, &program_publication_access(), |key| {
                for entry in &self.trees {
                    funding::checkpoint(phase, entry.tree.runtime())?;
                    entry.tree.discard(phase, &key)?;
                }
                Ok(())
            })
            .map_err(WorthQueryHistoryNavigationAdvanceDenial::Navigation)?
            .map_err(WorthQueryHistoryNavigationAdvanceDenial::Outputs)
    }
    fn prepare<'runtime, N: ApplicationHistoryNavigationBinding<S, H>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        document: &WorthQueryDocumentHistory<'runtime, S, H>,
        history_access: &H::Access,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
        expected: WorthQueryHistorySelection,
        intent: WorthQueryHistoryNavigationIntent,
        key: u128,
    ) -> Result<
        WorthQueryDocumentHistoryNavigation<'runtime, S>,
        WorthQueryHistoryNavigationAdvanceDenial,
    >
    where
        Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
    {
        request
            .application
            .prepare_document_history_navigation_with_output_forest::<H, N>(
                phase,
                &program_publication_access(),
                document,
                history_access,
                read,
                expected,
                intent,
                key,
                self.profile.clone(),
            )
            .map_err(WorthQueryHistoryOutputDenial::from)
            .map_err(WorthQueryHistoryNavigationAdvanceDenial::Outputs)
    }
    fn advance<'runtime, N: ApplicationHistoryNavigationBinding<S, H>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        document: &WorthQueryDocumentHistory<'runtime, S, H>,
        history_access: &H::Access,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
        navigation: &WorthQueryDocumentHistoryNavigation<'runtime, S>,
        key: u128,
    ) -> Result<
        Option<WorthQueryHistoryNavigationCompletion>,
        WorthQueryHistoryNavigationAdvanceDenial,
    >
    where
        Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
    {
        let current = WorthQueryApplicationRequest {
            application: request.application,
            principal: request.principal,
            scope: request.scope,
            branch: document.branch(),
        };
        let mut slot = None;
        let result = request.application.advance_document_history_navigation_with_output_forest::<H, N, WorthQueryHistoryOutputDenial>(
            phase, &program_publication_access(), document, history_access, read,
            navigation, key, &self.profile, |registered, step| {
                slot = Some(registered);
                self.progress(phase, &current, registered, step)
            },
        ).map_err(WorthQueryHistoryOutputDenial::from).and_then(|result| result)
            .map_err(WorthQueryHistoryNavigationAdvanceDenial::Outputs);
        if navigation.posture() == WorthQueryHistoryNavigationPosture::Selected
            && let Some(key) = slot
        {
            // Selection already happened. Keep refused cleanup in each original
            // registry, exactly as the fixed-tree driver does.
            for entry in &self.trees {
                entry.tree.selected(phase, &key);
            }
        }
        result
    }
}

impl<S: ApplicationSchema + 'static> WorthQueryHistoryWithOutputTrees<S> {
    fn progress(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        key: [u8; 32],
        step: Step<'_, '_>,
    ) -> Result<Option<WorthQueryCompletedHistoryOutputs>, WorthQueryHistoryOutputDenial> {
        use WorthQueryHistoryOutputDenial as Denial;
        let (branch, source, authority, selecting) = match step {
            Step::Pending => {
                for entry in &self.trees {
                    funding::checkpoint(phase, entry.tree.runtime())?;
                    entry.tree.progress(phase, request, key, Step::Pending)?;
                }
                return Ok(None);
            }
            Step::Selected(completed) => {
                for entry in &self.trees {
                    funding::checkpoint(phase, entry.tree.runtime())?;
                    entry
                        .tree
                        .progress(phase, request, key, Step::Selected(completed.clone()))?;
                }
                return Ok(None);
            }
            Step::Outputs {
                branch,
                source,
                authority,
            } => (branch, source, authority, false),
            Step::Selecting {
                branch,
                source,
                authority,
            } => (branch, source, authority, true),
        };
        let execution = phase
            .checked_execution_request_for(request.application.product_runtime())
            .map_err(Denial::Execution)?;
        let count = if selecting { 0 } else { self.trees.len() };
        let _temporary = funding::reserve(
            &execution,
            count
                .checked_mul(std::mem::size_of::<Arc<WorthQueryRetainedHistoryOutputTree>>())
                .ok_or(Denial::CapacityOverflow)?,
        )?;
        let mut completed = Vec::with_capacity(count);
        let mut pending = false;
        for (index, entry) in self.trees.iter().enumerate() {
            funding::checkpoint(phase, entry.tree.runtime())?;
            let tree = authority
                .with_tree_authority(
                    phase,
                    &program_publication_access(),
                    index,
                    |tree_authority| {
                        if !entry.tree.has_original_attempt(&key)? {
                            if selecting
                                || request
                                    .application
                                    .can_verify_selected_history_leaf(
                                        phase,
                                        &program_publication_access(),
                                        tree_authority,
                                    )
                                    .map_err(Denial::GraphQualification)?
                            {
                                return Ok((true, None));
                            }
                        }
                        let step = if selecting {
                            Step::Selecting {
                                branch,
                                source: Arc::clone(&source),
                                authority: tree_authority,
                            }
                        } else {
                            Step::Outputs {
                                branch,
                                source: Arc::clone(&source),
                                authority: tree_authority,
                            }
                        };
                        entry
                            .tree
                            .progress(phase, request, key, step)
                            .map(|tree| (false, tree))
                    },
                )
                .map_err(Denial::Navigation)??;
            if tree.0 {
                continue;
            }
            match tree.1 {
                Some(tree) => completed.push(tree),
                None => pending = true,
            }
        }
        if selecting || pending {
            return Ok(None);
        }
        request
            .application
            .qualify_original_history_output_forest(
                phase,
                &program_publication_access(),
                authority,
                &completed,
            )
            .map(Some)
            .map_err(Denial::GraphQualification)
    }
}
