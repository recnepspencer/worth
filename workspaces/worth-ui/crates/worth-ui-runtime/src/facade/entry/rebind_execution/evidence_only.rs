use super::WorthUiActiveApplicationSession;

pub(crate) struct WorthUiPreparedEvidenceOnlyApplicationRebind<'session> {
    session: &'session mut WorthUiActiveApplicationSession,
    successor_authority:
        crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
    appearance_succession: super::super::UiPreparedAppearanceGenerationSuccession,
    overlay_bindings: crate::runtime::portal::UiPortalOverlayBindingLifecycle,
    occurrence_geometry: crate::mounting::UiMountedOccurrenceGeometryState,
    pointer_succession:
        crate::runtime::pointer_affordance::UiPreparedPointerAffordanceGenerationSuccession,
    owners: crate::runtime::appearance::UiPreparedRetainedAppearanceOwnerSuccession,
    _admitted_candidate: crate::runtime::WorthUiAdmittedReplacementCandidate,
    _comparison: crate::runtime::WorthUiRuntimeArtifactComparison,
}

impl<'session> WorthUiPreparedEvidenceOnlyApplicationRebind<'session> {
    pub(super) fn new(
        session: &'session mut WorthUiActiveApplicationSession,
        succession: crate::runtime::observation::UiAuthoredSourceSuccession,
        pointer_succession: crate::runtime::pointer_affordance::UiPreparedPointerAffordanceGenerationSuccession,
    ) -> Result<Self, crate::runtime::rebind::UiRebindPreparationDenial> {
        let crate::runtime::observation::UiAuthoredSourceSuccession::EvidenceOnly {
            successor_authority,
            admitted_candidate,
            comparison,
        } = succession
        else {
            return Err(crate::runtime::rebind::UiRebindPreparationDenial::InvalidSemanticProof);
        };
        let predecessor = session.active_generation_identity();
        let successor = crate::runtime::WorthUiActiveApplicationGenerationIdentity::current(
            session.session_identity(),
            successor_authority.generation_identity(),
        );
        let generation_succession = crate::facade::prepared_application_authority::
            WorthUiPreparedApplicationGenerationSuccession::new(
                predecessor.prepared_generation().clone(),
                successor.prepared_generation().clone(),
            );
        let appearance_succession = session
            .prepare_appearance_generation_succession(&generation_succession)
            .map_err(|denial| match denial {
                super::super::UiAppearanceGenerationSuccessionDenial::Theme(denial) => {
                    crate::runtime::rebind::UiRebindPreparationDenial::AppearanceThemeSuccession(
                        denial,
                    )
                }
                super::super::UiAppearanceGenerationSuccessionDenial::Inspection(denial) => {
                    crate::runtime::rebind::UiRebindPreparationDenial::AppearanceInspectionSuccession(
                        denial,
                    )
                }
            })?;
        let overlay_bindings = session
            .authored_overlay_bindings
            .prepare_application_replacement(
                session.application.prepared_authority(),
                &successor_authority,
                &[],
            )
            .map_err(|_| {
                crate::runtime::rebind::UiRebindPreparationDenial::CandidateCutoverPreparation
            })?;
        let occurrence_geometry = session
            .application
            .prepare_retained_layout_succession(
                &session.mounted,
                &successor_authority,
                &overlay_bindings,
            )
            .map_err(
                crate::runtime::rebind::UiRebindPreparationDenial::CandidateOccurrenceGeometry,
            )?;
        let owners = session
            .prepare_retained_appearance_owners(&successor_authority, &appearance_succession)?;
        Ok(Self {
            session,
            successor_authority,
            appearance_succession,
            overlay_bindings,
            occurrence_geometry,
            pointer_succession,
            owners,
            _admitted_candidate: admitted_candidate,
            _comparison: comparison,
        })
    }

    pub(crate) fn commit(
        self,
    ) -> Result<
        (
            crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity,
            crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity,
        ),
        Box<Self>,
    >{
        if self
            .pointer_succession
            .validate_predecessor(
                &self.session.active_generation_identity(),
                self.session.pointer_affordance_snapshot.as_ref(),
                self.session
                    .observation_clock
                    .as_ref()
                    .map(|clock| clock.sample_millis()),
                &self.session.mounted,
            )
            .is_err()
            || !self
                .owners
                .matches_projection(self.session.appearance_owner_snapshot.as_ref())
            || self
                .session
                .prepare_retained_appearance_owners(
                    &self.successor_authority,
                    &self.appearance_succession,
                )
                .is_err()
        {
            return Err(Box::new(self));
        }
        let Self {
            session,
            successor_authority,
            appearance_succession,
            overlay_bindings,
            occurrence_geometry,
            pointer_succession,
            owners,
            _admitted_candidate: _,
            _comparison: _,
        } = self;
        let generations = session
            .application
            .commit_evidence_only_rebind(successor_authority);
        session.commit_retained_appearance_succession(appearance_succession, owners);
        session.authored_overlay_bindings = overlay_bindings;
        session.pointer_affordance_snapshot = pointer_succession.into_snapshot();
        session
            .mounted
            .commit_retained_geometry_succession(occurrence_geometry);
        Ok(generations)
    }

    pub(crate) fn generation_identity(
        &self,
    ) -> &crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity
    {
        self.successor_authority.generation_identity()
    }
}
