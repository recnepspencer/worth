use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WorthServerProductSessionOperationProtocol {
    operation_name: String,
    method: String,
    route: String,
    request_schema_identity: String,
    response_schema_identity: String,
    requires_product_session: bool,
    #[serde(skip_serializing_if = "is_false")]
    requires_server_instance: bool,
}

impl WorthServerProductSessionOperationProtocol {
    pub(crate) fn new(
        operation_name: String,
        method: String,
        route: String,
        request_schema_identity: String,
        response_schema_identity: String,
        requires_product_session: bool,
        requires_server_instance: bool,
    ) -> Self {
        Self {
            operation_name,
            method,
            route,
            request_schema_identity,
            response_schema_identity,
            requires_product_session,
            requires_server_instance,
        }
    }

    pub fn operation_name(&self) -> &str {
        &self.operation_name
    }

    pub fn method(&self) -> &str {
        &self.method
    }

    pub fn route(&self) -> &str {
        &self.route
    }

    pub fn request_schema_identity(&self) -> &str {
        &self.request_schema_identity
    }

    pub fn response_schema_identity(&self) -> &str {
        &self.response_schema_identity
    }

    pub fn requires_product_session(&self) -> bool {
        self.requires_product_session
    }

    pub fn requires_server_instance(&self) -> bool {
        self.requires_server_instance
    }
}

fn is_false(value: &bool) -> bool {
    !value
}
