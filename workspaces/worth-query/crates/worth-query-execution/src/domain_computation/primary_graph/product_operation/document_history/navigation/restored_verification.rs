//! Healthy owned publications need no source disclosure or original SDK tree.
use super::*;
use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use worth_query_installation::facade::ApplicationSchema;

impl<S: ApplicationSchema> NavigationProgress<'_, S> {
    /// Only the untouched committed restoration is eligible. A fallback's
    /// later endpoint still requires its complete original tree qualification.
    pub(super) fn accept_verified_restoration(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<bool, WorthQueryHistoryNavigationDenial> {
        if self.ownership.is_none()
            || !matches!(
                self.phase,
                NavigationPhase::AwaitingOutputs { .. }
                    | NavigationPhase::Selecting { _outputs: None, .. }
            )
        {
            return Ok(false);
        }
        let current = self.restoration_is_current(phase)?;
        if current {
            if matches!(self.phase, NavigationPhase::AwaitingOutputs { .. }) {
                let NavigationPhase::AwaitingOutputs {
                    candidate,
                    exact,
                    _publication,
                } = std::mem::replace(&mut self.phase, NavigationPhase::Discarded)
                else {
                    unreachable!()
                };
                self.phase = NavigationPhase::Selecting {
                    candidate,
                    exact,
                    _publication,
                    _outputs: None,
                };
            }
        } else if matches!(
            self.phase,
            NavigationPhase::Selecting { _outputs: None, .. }
        ) {
            let NavigationPhase::Selecting {
                candidate,
                exact,
                _publication,
                ..
            } = std::mem::replace(&mut self.phase, NavigationPhase::Discarded)
            else {
                unreachable!()
            };
            self.phase = NavigationPhase::AwaitingOutputs {
                candidate,
                exact,
                _publication,
            };
        }
        Ok(current)
    }

    fn restoration_is_current(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<bool, WorthQueryHistoryNavigationDenial> {
        use WorthQueryHistoryNavigationDenial as Denial;
        let restored = self.output_source.as_ref().ok_or(Denial::IntentDrift)?;
        let original = restored.observation();
        let selected = self
            .application
            .on_branch(crate::basis::WorthQueryProductBranch::from_occurrence(
                original.product_observation().lifecycle_incarnation(),
            ))
            .select(phase)
            .map_err(Denial::ProductAdmission)?;
        if original
            .product_observation()
            .compare(selected.product().observation())
            .is_err()
        {
            return if matches!(self.phase, NavigationPhase::Selecting { .. }) {
                Err(Denial::HeadChanged)
            } else {
                Ok(false)
            };
        }
        let execution = phase
            .checked_execution_request_for(&self.application.runtime.product_runtime)
            .map_err(Denial::Stopped)?;
        let historical = self.source.as_ref().ok_or(Denial::IntentDrift)?;
        let current = self
            .application
            .runtime
            .primary_provider
            .graph
            .with_runtime(&execution, |native| {
                let evidence = restored
                    .native(&execution, native)
                    .map_err(Denial::RestoredOutputRead)?;
                // An independent/non-restored root must keep its ordinary admission.
                // Profiles describe fallback correspondence; they establish no ownership.
                if let Some(profile) = &self.output_profile {
                    for tree in profile.tree_profiles() {
                        // Native route membership is not a complete discovery fact.
                        // A declared non-leaf retains its whole-set owner.
                        if !tree.leaf {
                            return Ok(false);
                        }
                        if evidence
                            .selected_output_route(
                                &execution,
                                tree.family,
                                tree.source_intent().source_subject(),
                                worth_foundational::facade::CanonicalDigestId::new(
                                    tree.source_intent().source_partition(),
                                ),
                            )
                            .map_err(Denial::RestoredOutputRead)?
                            .is_none()
                        {
                            return Ok(false);
                        }
                    }
                }
                let mut count = 0;
                for route in evidence
                    .selected_output_routes(&execution)
                    .map_err(Denial::RestoredOutputRead)?
                {
                    let route = route.map_err(Denial::RestoredOutputRead)?;
                    count += 1;
                    if !selected
                        .selected_route_is_current(&execution, native, route)
                        .map_err(Denial::RestoredOutputCurrentness)?
                    {
                        return Ok(false);
                    }
                }
                Ok(count != 0 || historical.root_posture.is_none())
            })?;
        drop(selected);
        Ok(current)
    }
}
