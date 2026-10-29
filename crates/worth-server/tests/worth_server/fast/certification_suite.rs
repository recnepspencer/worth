#[allow(unused_imports)]
pub(crate) use crate::shared_support_direct_context_runtime as direct_context_runtime;
#[allow(unused_imports)]
pub(crate) use crate::shared_support_query_handoff_fixture as query_handoff_fixture;
#[allow(unused_imports)]
pub(crate) use crate::shared_support_query_handoff_runtime as query_handoff_runtime;
#[allow(unused_imports)]
pub(crate) use crate::shared_support_response_fixture as response_fixture;
#[allow(unused_imports)]
pub(crate) use crate::shared_support_test_bootstrap_policy as test_bootstrap_policy;

#[allow(unused_imports)]
pub(crate) use crate::shared_support_worth_native_assertions as worth_native_assertions;
#[allow(unused_imports)]
pub(crate) use crate::shared_support_worth_native_runtime as worth_native_runtime;
#[path = "../fixtures/support/certification/bundle.rs"]
pub(crate) mod certification_bundle;

#[path = "../fixtures/support/certification/counter_assertions.rs"]
pub(crate) mod certification_counter_assertions;

#[path = "../fixtures/support/certification/digest_assertions.rs"]
pub(crate) mod certification_digest_assertions;

#[path = "../fixtures/support/certification/fixture.rs"]
pub(crate) mod certification_fixture;

#[path = "../fixtures/support/certification/worth_native_fixture.rs"]
pub(crate) mod certification_worth_native_fixture;

#[allow(unused_imports)]
pub(crate) use crate::exhaustive::certification_suite::mixed_hostility_matrix;

#[path = "certification/response_denial_localization.rs"]
pub(crate) mod response_denial_localization;

#[path = "certification/response_envelope_parity.rs"]
pub(crate) mod response_envelope_parity;

#[allow(unused_imports)]
pub(crate) use crate::compile_fail::certification_suite::structural_sabotage_suite;

#[allow(unused_imports)]
pub(crate) use crate::exhaustive::certification_suite::worth_native_hostility_matrix;

#[path = "certification/worth_native_no_glue_equivalence.rs"]
pub(crate) mod worth_native_no_glue_equivalence;

#[path = "certification/worth_native_retained_fact_parity.rs"]
pub(crate) mod worth_native_retained_fact_parity;

#[allow(unused_imports)]
pub(crate) use crate::compile_fail::certification_suite::worth_native_sabotage_suite;
