use super::*;

struct HostNeutralFreeze {
    prepared: crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
    mounted_frame_retention_budget: crate::mounting::UiMountedFrameRetentionBudget,
    host_observation_capacity: crate::facade::observation_report::UiHostObservationCapacity,
    font_collection: std::sync::Arc<worth_ui_text::UiGlobalFontCollection>,
}

impl WorthUiApplicationBuilder<UiChangeProfileInstalled, UiIntentWiringSatisfied> {
    fn freeze_host_neutral(self) -> Result<HostNeutralFreeze, WorthUiApplicationPreparationDenial> {
        let capability_snapshot = self
            .inner
            .freeze_with_registration_report()
            .into_accepted_snapshot();
        let intent_execution_bindings = self
            .intent_execution_bindings
            .freeze(capability_snapshot.intent_definitions())
            .map_err(|denial| {
                WorthUiApplicationPreparationDenial::IntentExecutionBinding(Box::new(denial))
            })?;
        let preparation_source = match self.preparation_source {
            WorthUiApplicationBuilderPreparationSource::RustAuthored(input) => {
                WorthUiApplicationPreparationSource::rust_authored(&input, &capability_snapshot)?
            }
            WorthUiApplicationBuilderPreparationSource::Watched(submission) => {
                WorthUiApplicationPreparationSource::watched_submission(
                    *submission,
                    capability_snapshot.digest(),
                )?
            }
        };
        let service_support = intent_execution_bindings
            .runtime_service_support()
            .union(preparation_source.runtime_service_support())
            .union(capability_snapshot.commands().runtime_service_support())
            .union(
                capability_snapshot
                    .mosaic_regions()
                    .runtime_service_support(),
            );
        let service_policy_plan = crate::declaration::UiNormalizedServicePolicyPlan::normalize(
            self.service_policy_defaults,
            preparation_source.authored_service_policy_defaults(),
            service_support,
        )
        .map_err(WorthUiApplicationPreparationDenial::ServicePolicyNormalization)?;
        let prepared = prepare_application_authority(WorthUiApplicationPreparationInput {
            capability_snapshot,
            preparation_source,
            visual_inspection_policy: self.visual_inspection_policy,
            graph_world_profile: self.graph_world_profile,
            runtime_instance_basis_admissions: self
                .runtime_instance_basis_admissions
                .into_boxed_slice(),
            measurement_inspection_evidence: self
                .measurement_inspection_evidence
                .into_boxed_slice(),
            query_binding_plan: self.query_binding_plan,
            intent_application_facts: self.intent_application_facts,
            intent_execution_bindings,
            service_policy_defaults: self.service_policy_defaults,
            service_policy_plan,
            change_profile: self.change_profile.profile,
        })?;
        Ok(HostNeutralFreeze {
            prepared,
            mounted_frame_retention_budget: self.mounted_frame_retention_budget,
            host_observation_capacity: self.host_observation_capacity,
            font_collection: self.font_collection.unwrap_or_else(|| {
                std::sync::Arc::new(
                    worth_ui_text::UiGlobalFontCollection::admit_qualified_profile()
                        .expect("embedded qualified text profile")
                        .0,
                )
            }),
        })
    }
}

impl WorthUiApplicationBuilder<UiChangeProfileInstalled, UiIntentWiringSatisfied> {
    pub fn freeze(
        self,
    ) -> Result<crate::facade::entry::WorthUiHostNeutralApp, WorthUiApplicationPreparationDenial>
    {
        let frozen = self.freeze_host_neutral()?;
        Ok(crate::facade::entry::WorthUiHostNeutralApp::new(
            frozen.prepared,
            frozen.mounted_frame_retention_budget,
            frozen.host_observation_capacity,
            frozen.font_collection,
        ))
    }
}
