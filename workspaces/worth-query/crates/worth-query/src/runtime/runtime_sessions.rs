use super::evidence_identities::runtime_live_view_consumer_attachment_identity;
use super::live_subscription::live_subscription_source_identity;
use super::runtime_session_lowering::{
    install_live_subscription_activation, lower_runtime_live_subscription_read_binding,
    lower_runtime_live_subscription_request, LoweredRuntimeLiveSubscriptionRequest,
};
use super::*;

impl WorthQueryRuntime {
    pub fn preview<'a>(
        &'a mut self,
        label: WorthQuerySessionLabel,
    ) -> Result<WorthQueryPreviewSession<'a>, WorthQueryRuntimeError> {
        self.preview_with_options(label, WorthQueryPreviewOptions::default())
    }

    pub fn branch<'a>(
        &'a mut self,
        label: WorthQuerySessionLabel,
    ) -> Result<WorthQueryBranchSession<'a>, WorthQueryRuntimeError> {
        self.branch_with_options(label, WorthQueryBranchOptions::default())
    }

    pub fn branch_with_options<'a>(
        &'a mut self,
        label: WorthQuerySessionLabel,
        options: WorthQueryBranchOptions,
    ) -> Result<WorthQueryBranchSession<'a>, WorthQueryRuntimeError> {
        self.try_branch_with_options(label, options)
    }

    pub fn try_branch<'a>(
        &'a mut self,
        label: WorthQuerySessionLabel,
    ) -> Result<WorthQueryBranchSession<'a>, WorthQueryRuntimeError> {
        self.try_branch_with_options(label, WorthQueryBranchOptions::default())
    }

    pub fn try_branch_with_options<'a>(
        &'a mut self,
        label: WorthQuerySessionLabel,
        options: WorthQueryBranchOptions,
    ) -> Result<WorthQueryBranchSession<'a>, WorthQueryRuntimeError> {
        self.admit_facade_family(WorthQueryRuntimeFacadeFamily::BranchPreview)?;
        self.admit_branch_session_label(&label)?;
        let basis_admission = self.branch_basis_admission(
            label.clone(),
            options.effect_policy(),
            "runtime-branch-basis-admission",
        );
        Ok(WorthQueryBranchSession::new(
            label,
            self,
            options,
            basis_admission,
        )?)
    }

    pub(crate) fn capture_branch_comparison_basis(
        &self,
        label: WorthQuerySessionLabel,
    ) -> Result<WorthQueryRuntimeBranchComparisonBasis, WorthQueryRuntimeError> {
        self.admit_facade_family(WorthQueryRuntimeFacadeFamily::BranchPreview)?;
        let admission = self.branch_basis_admission(
            label,
            WorthQueryEffectPolicy::DeriveOnly,
            "runtime-branch-comparison-basis-admission",
        );
        Ok(WorthQueryRuntimeBranchComparisonBasis::new(
            admission,
            self.current_snapshot_identity()?,
        ))
    }

    fn branch_basis_admission(
        &self,
        label: WorthQuerySessionLabel,
        effect_policy: WorthQueryEffectPolicy,
        evidence_tag: &'static str,
    ) -> WorthQueryBranchBasisAdmission {
        let branch_support_evidence = self
            .backend
            .support_profile()
            .support_for(WorthQueryRuntimeFacadeFamily::BranchPreview)
            .map(|support| support.evidence().to_vec())
            .unwrap_or_default();
        let evidence_rows = std::iter::once(WorthQueryBasisAdmissionEvidenceRow::tagged(
            evidence_tag,
            evidence_tag,
        ))
        .chain(
            branch_support_evidence
                .into_iter()
                .map(WorthQueryBasisAdmissionEvidenceRow::support_profile_token),
        )
        .collect::<Vec<_>>();
        WorthQueryBranchBasisAdmission::new(
            &self.evidence_authority,
            label,
            effect_policy,
            evidence_rows,
        )
    }

    pub fn preview_with_options<'a>(
        &'a mut self,
        label: WorthQuerySessionLabel,
        options: WorthQueryPreviewOptions,
    ) -> Result<WorthQueryPreviewSession<'a>, WorthQueryRuntimeError> {
        self.try_preview_with_options(label, options)
    }

    pub fn try_preview<'a>(
        &'a mut self,
        label: WorthQuerySessionLabel,
    ) -> Result<WorthQueryPreviewSession<'a>, WorthQueryRuntimeError> {
        self.try_preview_with_options(label, WorthQueryPreviewOptions::default())
    }

    pub fn try_preview_with_options<'a>(
        &'a mut self,
        label: WorthQuerySessionLabel,
        options: WorthQueryPreviewOptions,
    ) -> Result<WorthQueryPreviewSession<'a>, WorthQueryRuntimeError> {
        self.admit_facade_family(WorthQueryRuntimeFacadeFamily::BranchPreview)?;
        self.admit_preview_session_label(&label)?;
        let basis_admission = self.backend.admit_preview_basis(
            &label,
            options.effect_policy(),
            &self.evidence_authority,
        )?;
        Ok(WorthQueryPreviewSession::new(
            label,
            self,
            options.effect_policy(),
            basis_admission,
        )?)
    }

    pub(in crate::runtime) fn open_preview_with_admitted_basis<'a>(
        &'a mut self,
        basis_admission: WorthQueryPreviewBasisAdmission,
    ) -> Result<WorthQueryPreviewSession<'a>, WorthQueryRuntimeError> {
        self.admit_facade_family(WorthQueryRuntimeFacadeFamily::BranchPreview)?;
        let label = basis_admission.session_label().clone();
        let effect_policy = basis_admission.effect_policy();
        self.admit_preview_session_label(&label)?;
        Ok(WorthQueryPreviewSession::new(
            label,
            self,
            effect_policy,
            basis_admission,
        )?)
    }

    pub fn support_profile(&self) -> WorthQueryRuntimeSupportProfile {
        self.backend.support_profile()
    }

    pub fn graph_index_inventory(&self) -> WorthQueryGraphIndexInventory {
        self.backend.support_profile().graph_index_inventory()
    }

    pub(crate) fn admit_graph_read_access_for_family(
        &self,
        family: &WorthQueryReadFamily,
    ) -> Result<WorthQueryGraphReadAccessAdmission, WorthQueryGraphReadAccessShapeExplanationError>
    {
        let authority = WorthQueryGraphReadAccessAuthorityContext::runtime_current_compatibility();
        crate::runtime::admit_graph_read_access_for_family_in_authority_with_inventory_and_lookup(
            family,
            &authority,
            self.graph_index_inventory(),
            self.installed_domain_execution_index(),
        )
    }

    pub(crate) fn admit_graph_read_access_for_family_in_authority(
        &self,
        family: &WorthQueryReadFamily,
        authority: &WorthQueryGraphReadAccessAuthorityContext,
    ) -> Result<WorthQueryGraphReadAccessAdmission, WorthQueryGraphReadAccessShapeExplanationError>
    {
        crate::runtime::admit_graph_read_access_for_family_in_authority_with_inventory_and_lookup(
            family,
            authority,
            self.graph_index_inventory(),
            self.installed_domain_execution_index(),
        )
    }

    pub(super) fn admit_facade_family(
        &self,
        family: WorthQueryRuntimeFacadeFamily,
    ) -> Result<(), WorthQueryRuntimeError> {
        self.backend
            .support_profile()
            .admit(family)
            .map_err(WorthQueryRuntimeError::UnsupportedFacadeFamily)
    }

    pub(in crate::runtime) fn admit_facade_family_lane(
        &self,
        family: WorthQueryRuntimeFacadeFamily,
        authority_lane: WorthQueryAuthorityLane,
    ) -> Result<(), WorthQueryRuntimeError> {
        self.admit_facade_family(family)?;
        let support_profile = self.backend.support_profile();
        let Some(row) = support_profile.support_for(family) else {
            return Err(WorthQueryRuntimeError::UnsupportedFacadeFamily(
                WorthQueryRuntimeSupportDenial::unsupported(
                    family,
                    "backend support profile does not declare this facade family",
                ),
            ));
        };
        if row.authority_lanes().contains(&authority_lane) {
            Ok(())
        } else {
            Err(WorthQueryRuntimeError::UnsupportedFacadeFamily(
                WorthQueryRuntimeSupportDenial::new(
                    family,
                    row.status(),
                    Some(row.teaching_posture()),
                    format!(
                        "backend support profile does not admit `{}` lane for `{}` facade family",
                        authority_lane, family
                    ),
                ),
            ))
        }
    }

    fn admit_preview_session_label(
        &mut self,
        label: &WorthQuerySessionLabel,
    ) -> Result<(), WorthQueryRuntimeError> {
        admit_session_label_for_lane(
            &mut self.preview_session_labels,
            WorthQueryAuthorityLane::PreviewTruth,
            label,
        )
    }

    fn admit_branch_session_label(
        &mut self,
        label: &WorthQuerySessionLabel,
    ) -> Result<(), WorthQueryRuntimeError> {
        admit_session_label_for_lane(
            &mut self.branch_session_labels,
            WorthQueryAuthorityLane::BranchLocalTruth,
            label,
        )
    }

    pub(super) fn install_live_subscription_for_request(
        &mut self,
        view_name: &str,
        request: &DeclarativeLiveQueryRequest,
        schema_view: QuerySchemaView,
        future_selection: crate::subscription::QuerySubscriptionFutureSelection,
    ) -> Result<WorthQueryRuntimeLiveSubscriptionActivation, WorthQueryRuntimeError> {
        let lowered_subscription = lower_runtime_live_subscription_request(
            &*self.backend,
            view_name,
            request,
            schema_view,
            future_selection,
        )?;
        self.install_lowered_live_subscription(view_name, request, lowered_subscription, None)
    }

    pub(super) fn install_live_subscription_for_read_binding(
        &mut self,
        view_name: &str,
        binding: WorthQueryReadExecutionBinding,
    ) -> Result<WorthQueryRuntimeLiveSubscriptionActivation, WorthQueryRuntimeError> {
        let lowered_subscription =
            lower_runtime_live_subscription_read_binding(&*self.backend, view_name, &binding)?;
        let request = binding
            .read_family()
            .read_graph()
            .declarative_request()
            .clone();
        self.install_lowered_live_subscription(
            view_name,
            &request,
            lowered_subscription,
            Some(binding),
        )
    }

    fn install_lowered_live_subscription(
        &mut self,
        view_name: &str,
        request: &DeclarativeLiveQueryRequest,
        lowered_subscription: LoweredRuntimeLiveSubscriptionRequest,
        read_authority_binding: Option<WorthQueryReadExecutionBinding>,
    ) -> Result<WorthQueryRuntimeLiveSubscriptionActivation, WorthQueryRuntimeError> {
        let activation = lowered_subscription.activation.clone();
        let counters = activation.counters().clone();
        let activation_receipt =
            install_live_subscription_activation(&mut *self.backend, view_name, &activation)?;
        let remask_posture = activation_receipt.remask_posture().cloned();
        let active_lane_admission =
            admit_active_subscription_lane(activation.clone(), runtime_active_lifecycle_budget())
                .map_err(
                |error| WorthQueryRuntimeError::LiveSubscriptionInstallation {
                    view_name: view_name.to_string(),
                    stage: "active-lane-admission",
                    message: format!("{error:?}"),
                },
            )?;
        let active_lane_handle =
            open_active_subscription_lane(&mut self.active_subscriptions, active_lane_admission)
                .map_err(
                    |error| WorthQueryRuntimeError::LiveSubscriptionInstallation {
                        view_name: view_name.to_string(),
                        stage: "active-lane-open",
                        message: format!("{error:?}"),
                    },
                )?;
        let active_lane_counters = self.active_subscriptions.counters().clone();
        let consumer_attachment = attach_subscription_consumer(
            &mut self.active_subscriptions,
            &active_lane_handle,
            SubscriptionConsumerAttachmentRequest::from_consumer_identity(
                runtime_live_view_consumer_attachment_identity(view_name),
                activation_receipt.activation_identity().clone(),
            ),
            runtime_consumer_attachment_budget(),
        )
        .map_err(
            |error| WorthQueryRuntimeError::LiveSubscriptionInstallation {
                view_name: view_name.to_string(),
                stage: "consumer-attachment",
                message: format!("{error:?}"),
            },
        )?;
        let consumer_attachment_counters = self.active_subscriptions.counters().clone();

        let installation = WorthQueryRuntimeLiveSubscriptionInstallation::new(
            view_name,
            lowered_subscription.query_identity,
            lowered_subscription.canonical_query_digest,
            lowered_subscription.live_view_identity,
            lowered_subscription.canonical_result_shape_digest,
            lowered_subscription.subscription_family,
            lowered_subscription.subscription_declaration_identity,
            lowered_subscription.bridge_declaration_identity,
            lowered_subscription.admission_identity,
            live_subscription_source_identity(
                "activation",
                activation_receipt.activation_identity(),
            ),
            lowered_subscription.basis_binding_identity,
            lowered_subscription.signal_strategy_identity,
            live_subscription_source_identity(
                "active_lane",
                active_lane_handle.lane_digest().evidence_identity(),
            ),
            &consumer_attachment,
            runtime_subscription_budget_policy(),
            runtime_active_lifecycle_budget_policy(),
            runtime_consumer_attachment_budget_policy(),
            active_lane_counters,
            consumer_attachment_counters,
            live_subscription_source_identity("support", activation_receipt.support_identity()),
            counters,
        );

        Ok(WorthQueryRuntimeLiveSubscriptionActivation {
            installation,
            active_lane_handle,
            consumer_attachment,
            request: request.clone(),
            remask_posture,
            read_authority_binding,
        })
    }
}

fn admit_session_label_for_lane(
    admitted_labels: &mut std::collections::BTreeSet<WorthQuerySessionLabel>,
    authority_lane: WorthQueryAuthorityLane,
    label: &WorthQuerySessionLabel,
) -> Result<(), WorthQueryRuntimeError> {
    if admitted_labels.contains(label) {
        return Err(WorthQueryRuntimeError::SessionLabelCollision {
            authority_lane,
            label: label.clone(),
        });
    }
    admitted_labels.insert(label.clone());
    Ok(())
}
