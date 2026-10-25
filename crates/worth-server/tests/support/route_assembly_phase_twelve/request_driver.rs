use axum::{
    body::{to_bytes, Body},
    http::{Method, Request, StatusCode},
};
use serde_json::Value;
use tower::util::ServiceExt;
use worth_server::{WorthServer, WorthServerProjectedRouter};

pub struct WorthServerRouteHttpTestDriver {
    router: WorthServerProjectedRouter,
}

impl WorthServerRouteHttpTestDriver {
    pub fn new(server: &WorthServer) -> Self {
        Self {
            router: server.projected_router(),
        }
    }

    pub async fn get(
        &self,
        path: &str,
        headers: &[(&str, &str)],
    ) -> WorthServerRouteHttpTestResponse {
        self.send(Method::GET, path, headers, None, Vec::new())
            .await
    }

    pub async fn options(
        &self,
        path: &str,
        headers: &[(&str, &str)],
    ) -> WorthServerRouteHttpTestResponse {
        self.send(Method::OPTIONS, path, headers, None, Vec::new())
            .await
    }

    pub async fn post_json(
        &self,
        path: &str,
        headers: &[(&str, &str)],
        body: &Value,
    ) -> WorthServerRouteHttpTestResponse {
        self.send(
            Method::POST,
            path,
            headers,
            Some("application/json"),
            serde_json::to_vec(body).expect("json body should encode"),
        )
        .await
    }

    pub async fn request_json(
        &self,
        method: Method,
        path: &str,
        headers: &[(&str, &str)],
        body: &Value,
    ) -> WorthServerRouteHttpTestResponse {
        self.send(
            method,
            path,
            headers,
            Some("application/json"),
            serde_json::to_vec(body).expect("json body should encode"),
        )
        .await
    }

    pub async fn post_bytes(
        &self,
        path: &str,
        headers: &[(&str, &str)],
        content_type: Option<&str>,
        body: Vec<u8>,
    ) -> WorthServerRouteHttpTestResponse {
        self.send(Method::POST, path, headers, content_type, body)
            .await
    }

    async fn send(
        &self,
        method: Method,
        path: &str,
        headers: &[(&str, &str)],
        content_type: Option<&str>,
        body: Vec<u8>,
    ) -> WorthServerRouteHttpTestResponse {
        let mut request = Request::builder().method(method).uri(path);
        for (name, value) in headers {
            request = request.header(*name, *value);
        }
        if let Some(content_type) = content_type {
            request = request.header("content-type", content_type);
        }
        let request = request
            .body(Body::from(body))
            .expect("request should build");
        let response = self
            .router
            .clone_axum_router()
            .oneshot(request)
            .await
            .expect("route request should complete");
        let status = response.status();
        let headers = response.headers().clone();
        let body_bytes = to_bytes(response.into_body(), 2 * 1024 * 1024)
            .await
            .expect("response body should read");
        let body_json = serde_json::from_slice::<Value>(&body_bytes).ok();
        WorthServerRouteHttpTestResponse {
            status,
            headers,
            body_bytes: body_bytes.to_vec(),
            body_json,
        }
    }
}

pub struct WorthServerRouteHttpTestResponse {
    status: StatusCode,
    headers: axum::http::HeaderMap,
    body_bytes: Vec<u8>,
    body_json: Option<Value>,
}

impl WorthServerRouteHttpTestResponse {
    pub fn status(&self) -> StatusCode {
        self.status
    }

    pub fn route_kind(&self) -> Option<&str> {
        self.header("x-Worth-route-kind")
    }

    pub fn semantic_runtime_entered(&self) -> Option<bool> {
        self.header("x-Worth-semantic-runtime-entered")
            .map(|value| value == "true")
    }

    pub fn operation_name(&self) -> Option<&str> {
        self.header("x-Worth-operation-name")
    }

    pub fn plan_digest(&self) -> Option<&str> {
        self.header("x-Worth-plan-digest")
    }

    pub fn envelope_digest(&self) -> Option<&str> {
        self.header("x-Worth-envelope-digest").or_else(|| {
            self.json_body()
                .and_then(|body| body.get("canonical_digest"))
                .and_then(|value| value.as_str())
        })
    }

    pub fn scheduler_lane(&self) -> Option<&str> {
        self.header("x-Worth-scheduler-lane")
    }

    pub fn transport_denial_code(&self) -> Option<&str> {
        self.header("x-Worth-transport-denial-code")
    }

    pub fn json_body(&self) -> Option<&Value> {
        self.body_json.as_ref()
    }

    pub fn body_bytes(&self) -> &[u8] {
        &self.body_bytes
    }

    fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name).and_then(|value| value.to_str().ok())
    }
}
