use worth_testkit_compiler::{
    ProductionDiagnosticExpectation as Diagnostic, ProductionSemanticDenialContract as Contract,
    ProductionTypeMismatchPair as Pair,
};

pub(super) fn contracts() -> Vec<Contract<'static>> {
    vec![
        Contract {
            fixture: "route_request_requires_truth_commit_identity.rs",
            errors: &[
                Diagnostic::TypeMismatch {
                    pairs: &[
                        Pair {
                            expected: "worth_runtime_bridge::identity::BridgeIdentity<worth_runtime_bridge::identity::TruthCommitTag>",
                            found: "&str",
                        },
                    ],
                    symbols: &["BridgeIdentity", "TruthCommitTag", "str"],
                },
                Diagnostic::Code {
                    code: "E0277",
                    symbols: &["BridgeRouteRequest", "From", "From<&str>", "str"],
                },
            ],
        },
    ]
}
