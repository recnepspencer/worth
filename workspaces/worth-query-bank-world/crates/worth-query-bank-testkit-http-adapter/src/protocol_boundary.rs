pub use bank_http_adapter::{
    AuthentikAuthorizationCallback, AuthentikOidcConfiguration, AuthentikOidcConfigurationError,
    AuthentikOidcFlowError,
};

pub fn valid_configuration_builder() -> bank_http_adapter::AuthentikOidcConfigurationBuilder {
    AuthentikOidcConfiguration::builder()
        .issuer("https://issuer.example/application/o/bank/")
        .client_id("bank-client")
        .client_secret("sensitive-client-secret")
        .redirect_url("https://bank.example/callback")
        .introspection_url("https://issuer.example/application/o/introspect/")
        .revocation_url("https://issuer.example/application/o/revoke/")
}

pub fn assert_callback_denial(
    code: impl Into<String>,
    state: impl Into<String>,
    expected: AuthentikOidcFlowError,
) {
    let error =
        AuthentikAuthorizationCallback::new(code, state).expect_err("malformed callback must fail");
    assert_eq!(error, expected);
}

pub fn rust_files(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut pending = vec![root.to_path_buf()];
    let mut files = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(directory).expect("protocol directory") {
            let path = entry.expect("protocol entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                files.push(path);
            }
        }
    }
    files
}
