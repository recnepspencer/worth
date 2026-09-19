use super::*;

impl WorthQueryRuntime {
    pub fn public_authoritative_mutation_evidence_support_for_posture(
        posture: WorthQueryRuntimeBackendPosture,
    ) -> WorthQueryAuthoritativeMutationEvidenceSupport {
        WorthQueryAuthoritativeMutationEvidenceSupport::derive(
            &WorthQueryRuntimeSupportProfile::scaffold_backend_profile().with_posture(posture),
        )
    }

    pub fn public_authoritative_mutation_evidence_support_for_support_profile(
        support_profile: &WorthQueryRuntimeSupportProfile,
    ) -> WorthQueryAuthoritativeMutationEvidenceSupport {
        WorthQueryAuthoritativeMutationEvidenceSupport::derive(support_profile)
    }

    pub fn public_authoritative_mutation_evidence_closeout_for_support_profile(
        support_profile: &WorthQueryRuntimeSupportProfile,
    ) -> WorthQueryAuthoritativeMutationEvidenceCloseout {
        let public_api_contract =
            WorthQueryRuntimePublicApiContract::from_support_profile(support_profile);
        let support_matrix =
            WorthQueryRuntimePublicSupportMatrix::from_public_api_contract(&public_api_contract);
        let naming_contract = Self::public_api_naming_contract();
        let mutation_surface = WorthQueryMutationSurfaceReport::derive(
            public_api_contract.backend_posture(),
            &support_matrix,
            &naming_contract,
        );
        let query_support =
            Self::public_authoritative_mutation_evidence_support_for_support_profile(
                support_profile,
            );
        let bridge_support =
            worth_runtime_bridge::facade::RuntimeBridge::public_authoritative_mutation_evidence_support();
        let bridge_closeout =
            worth_runtime_bridge::facade::RuntimeBridge::public_authoritative_mutation_evidence_closeout();
        WorthQueryAuthoritativeMutationEvidenceCloseout::derive(
            public_api_contract.backend_posture(),
            &support_matrix,
            &mutation_surface,
            &naming_contract,
            &query_support,
            &bridge_support,
            &bridge_closeout,
        )
    }

    pub fn builder(
        product_world_resources: worth_query_execution::facade::integration::WorthQueryProductWorldResources,
    ) -> WorthQueryRuntimeBuilder {
        WorthQueryRuntimeBuilder::new(product_world_resources)
    }

    pub fn workspace(
        self,
        name: impl Into<String>,
    ) -> Result<WorthQueryWorkspace, WorthQueryRuntimeError> {
        WorthQueryWorkspace::new(name, self)
    }

    pub fn public_api_naming_contract() -> WorthQueryRuntimePublicApiNamingContract {
        WorthQueryRuntimePublicApiNamingContract::standard()
    }

    pub fn public_api_contract(&self) -> WorthQueryRuntimePublicApiContract {
        WorthQueryRuntimePublicApiContract::from_support_profile(&self.backend.support_profile())
    }

    pub fn public_handle_contract(&self) -> WorthQueryHandleContract {
        WorthQueryHandleContract::from_public_api_contract(&self.public_api_contract())
    }

    pub fn public_downstream_delivery_contract(
        &self,
    ) -> WorthQueryRuntimeDownstreamDeliveryContract {
        WorthQueryRuntimeDownstreamDeliveryContract::from_support_profile(
            &self.backend.support_profile(),
        )
    }

    pub fn public_support_matrix(&self) -> WorthQueryRuntimePublicSupportMatrix {
        WorthQueryRuntimePublicSupportMatrix::from_public_api_contract(&self.public_api_contract())
    }

    pub fn public_mutation_surface_report(&self) -> WorthQueryMutationSurfaceReport {
        WorthQueryMutationSurfaceReport::derive(
            self.public_api_contract().backend_posture(),
            &self.public_support_matrix(),
            &Self::public_api_naming_contract(),
        )
    }

    pub fn public_authoritative_mutation_evidence_support(
        &self,
    ) -> WorthQueryAuthoritativeMutationEvidenceSupport {
        Self::public_authoritative_mutation_evidence_support_for_support_profile(
            &self.backend.support_profile(),
        )
    }

    pub fn public_aspect_api_finalization_closeout(
        &self,
    ) -> WorthQueryAspectApiFinalizationCloseout {
        WorthQueryAspectApiFinalizationCloseout::derive(
            self.public_api_contract().backend_posture(),
            &self.public_support_matrix(),
            &self.public_mutation_surface_report(),
            &Self::public_api_naming_contract(),
        )
    }

    pub fn public_authoritative_mutation_evidence_closeout(
        &self,
    ) -> WorthQueryAuthoritativeMutationEvidenceCloseout {
        Self::public_authoritative_mutation_evidence_closeout_for_support_profile(
            &self.backend.support_profile(),
        )
    }

    pub fn downstream_delivery<T>(
        &self,
        view: &WorthQueryLiveView<T>,
    ) -> Result<Option<WorthQueryRuntimeDownstreamDelivery>, WorthQueryRuntimeError> {
        let target = WorthQueryLiveArtifactTarget::from_subscription_installation(
            view.subscription_installation(),
        );
        let state = self.live_subscriptions.get(&target).ok_or_else(|| {
            WorthQueryRuntimeError::MissingLiveSubscription(view.name().to_string())
        })?;
        Ok(project_downstream_delivery(
            &self.public_downstream_delivery_contract(),
            state,
        ))
    }

    pub fn admit_public_api_family(
        &self,
        family: WorthQueryRuntimeFacadeFamily,
    ) -> Result<WorthQueryRuntimePublicApiFamilyContract, WorthQueryRuntimeError> {
        let contract = self.public_api_contract();
        let row = contract.family(family).cloned().ok_or_else(|| {
            WorthQueryRuntimeError::UnsupportedFacadeFamily(
                WorthQueryRuntimeSupportDenial::unsupported(
                    family,
                    "runtime support matrix does not declare this public API family",
                ),
            )
        })?;
        self.admit_facade_family(family)?;
        if row.admission_fail_closed() {
            let reason = row
                .reason()
                .unwrap_or_else(|| match row.teaching_posture() {
                    WorthQueryRuntimeFamilyTeachingPosture::SupportGateOnly => {
                        "public runtime DX for this facade family remains support-gated"
                    }
                    WorthQueryRuntimeFamilyTeachingPosture::VisibleButDeferred => {
                        "public runtime DX for this facade family remains deferred"
                    }
                    WorthQueryRuntimeFamilyTeachingPosture::VisibleVocabularyOnly => {
                        "public runtime DX for this facade family remains vocabulary-only"
                    }
                    WorthQueryRuntimeFamilyTeachingPosture::OrdinaryRuntimeDx => {
                        "public runtime DX for this facade family is not admitted"
                    }
                });
            return Err(WorthQueryRuntimeError::UnsupportedFacadeFamily(
                WorthQueryRuntimeSupportDenial::new(
                    family,
                    row.status(),
                    Some(row.teaching_posture()),
                    reason,
                ),
            ));
        }
        Ok(row)
    }
}
