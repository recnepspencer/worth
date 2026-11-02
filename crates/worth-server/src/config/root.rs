use super::{
    WorthServerBindAddress, WorthServerMiddlewareConfig, WorthServerMiddlewareConfigBuilder,
    WorthServerOperatorEvidenceConfig, WorthServerOperatorEvidenceConfigBuilder,
    WorthServerQueryHandoffConfig, WorthServerQueryHandoffConfigBuilder,
    WorthServerRequestContextConfig, WorthServerRequestContextConfigBuilder,
    WorthServerResponseConfig, WorthServerResponseConfigBuilder,
};

#[derive(Clone, Debug)]
pub struct WorthServerConfig {
    bind_address: WorthServerBindAddress,
    middleware: WorthServerMiddlewareConfig,
    operator_evidence: WorthServerOperatorEvidenceConfig,
    query_handoff: WorthServerQueryHandoffConfig,
    response: WorthServerResponseConfig,
    request_context: WorthServerRequestContextConfig,
}

impl WorthServerConfig {
    pub fn builder() -> WorthServerConfigBuilder {
        WorthServerConfigBuilder::default()
    }

    pub fn bind_address(&self) -> WorthServerBindAddress {
        self.bind_address
    }

    pub fn middleware(&self) -> &WorthServerMiddlewareConfig {
        &self.middleware
    }

    pub fn operator_evidence(&self) -> &WorthServerOperatorEvidenceConfig {
        &self.operator_evidence
    }

    pub fn query_handoff(&self) -> &WorthServerQueryHandoffConfig {
        &self.query_handoff
    }

    pub fn response(&self) -> &WorthServerResponseConfig {
        &self.response
    }

    pub fn request_context(&self) -> &WorthServerRequestContextConfig {
        &self.request_context
    }
}

#[derive(Clone, Debug)]
pub struct WorthServerConfigBuilder {
    bind_address: Option<WorthServerBindAddress>,
    middleware: WorthServerMiddlewareConfigBuilder,
    operator_evidence: WorthServerOperatorEvidenceConfigBuilder,
    query_handoff: WorthServerQueryHandoffConfigBuilder,
    response: WorthServerResponseConfigBuilder,
    request_context: WorthServerRequestContextConfigBuilder,
}

impl Default for WorthServerConfigBuilder {
    fn default() -> Self {
        Self {
            bind_address: None,
            middleware: WorthServerMiddlewareConfig::builder(),
            operator_evidence: WorthServerOperatorEvidenceConfig::builder(),
            query_handoff: WorthServerQueryHandoffConfig::builder(),
            response: WorthServerResponseConfig::builder(),
            request_context: WorthServerRequestContextConfig::builder(),
        }
    }
}

impl WorthServerConfigBuilder {
    pub fn with_bind_address(mut self, bind_address: WorthServerBindAddress) -> Self {
        self.bind_address = Some(bind_address);
        self
    }

    pub fn with_middleware_config(mut self, middleware: WorthServerMiddlewareConfig) -> Self {
        self.middleware = WorthServerMiddlewareConfigBuilder::default()
            .with_compat_http_maximum_diagnostics_profile(
                middleware.compat_http_maximum_diagnostics_profile(),
            )
            .with_preview_branch_authorization_enabled(
                middleware.preview_branch_authorization_enabled(),
            )
            .with_query_mutation_enabled(middleware.query_mutation_enabled());
        self
    }

    pub fn with_middleware_config_builder(
        mut self,
        middleware: WorthServerMiddlewareConfigBuilder,
    ) -> Self {
        self.middleware = middleware;
        self
    }

    pub fn with_request_context_config(
        mut self,
        request_context: WorthServerRequestContextConfig,
    ) -> Self {
        self.request_context = WorthServerRequestContextConfigBuilder::default()
            .with_default_diagnostics_profile(request_context.default_diagnostics_profile())
            .with_maximum_diagnostics_profile(request_context.maximum_diagnostics_profile())
            .with_branch_targeting_enabled(request_context.branch_targeting_enabled())
            .with_preview_targeting_enabled(request_context.preview_targeting_enabled());
        self
    }

    pub fn with_query_handoff_config(
        mut self,
        query_handoff: WorthServerQueryHandoffConfig,
    ) -> Self {
        let mut builder = WorthServerQueryHandoffConfigBuilder::default()
            .with_workspace_provider_arc(query_handoff.workspace_provider().clone());
        if let Some(binding) = query_handoff.application_mutation() {
            builder = builder.with_application_mutation(binding.clone());
        }
        self.query_handoff = builder;
        self
    }

    pub fn with_query_handoff_config_builder(
        mut self,
        query_handoff: WorthServerQueryHandoffConfigBuilder,
    ) -> Self {
        self.query_handoff = query_handoff;
        self
    }

    pub fn with_operator_evidence_config(
        mut self,
        operator_evidence: WorthServerOperatorEvidenceConfig,
    ) -> Self {
        self.operator_evidence = WorthServerOperatorEvidenceConfigBuilder::default()
            .with_default_transform(operator_evidence.default_transform())
            .with_minimum_diagnostics_profile(operator_evidence.minimum_diagnostics_profile());
        self
    }

    pub fn with_operator_evidence_config_builder(
        mut self,
        operator_evidence: WorthServerOperatorEvidenceConfigBuilder,
    ) -> Self {
        self.operator_evidence = operator_evidence;
        self
    }

    pub fn with_response_config(mut self, response: WorthServerResponseConfig) -> Self {
        self.response = WorthServerResponseConfigBuilder::default()
            .with_default_success_transform(response.default_success_transform())
            .with_default_denial_transform(response.default_denial_transform())
            .with_success_minimum_diagnostics_profile(
                response.success_minimum_diagnostics_profile(),
            )
            .with_denial_minimum_diagnostics_profile(response.denial_minimum_diagnostics_profile());
        self
    }

    pub fn with_response_config_builder(
        mut self,
        response: WorthServerResponseConfigBuilder,
    ) -> Self {
        self.response = response;
        self
    }

    pub fn with_request_context_config_builder(
        mut self,
        request_context: WorthServerRequestContextConfigBuilder,
    ) -> Self {
        self.request_context = request_context;
        self
    }

    pub fn build(self) -> Result<WorthServerConfig, WorthServerConfigError> {
        let bind_address = self
            .bind_address
            .ok_or(WorthServerConfigError::MissingBindAddress)?;
        let middleware = self
            .middleware
            .build()
            .map_err(WorthServerConfigError::InvalidMiddlewareConfig)?;
        let query_handoff = self
            .query_handoff
            .build()
            .map_err(WorthServerConfigError::InvalidQueryHandoffConfig)?;
        let operator_evidence = self
            .operator_evidence
            .build()
            .map_err(WorthServerConfigError::InvalidOperatorEvidenceConfig)?;
        let response = self
            .response
            .build()
            .map_err(WorthServerConfigError::InvalidResponseConfig)?;
        let request_context = self
            .request_context
            .build()
            .map_err(WorthServerConfigError::InvalidRequestContextConfig)?;
        Ok(WorthServerConfig {
            bind_address,
            middleware,
            operator_evidence,
            query_handoff,
            response,
            request_context,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthServerConfigError {
    MissingBindAddress,
    InvalidMiddlewareConfig(super::WorthServerMiddlewareConfigError),
    InvalidOperatorEvidenceConfig(super::WorthServerOperatorEvidenceConfigError),
    InvalidQueryHandoffConfig(super::WorthServerQueryHandoffConfigError),
    InvalidResponseConfig(super::WorthServerResponseConfigError),
    InvalidRequestContextConfig(super::WorthServerRequestContextConfigError),
}
