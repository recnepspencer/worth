#[allow(unused_imports)]
use crate::shared_support_direct_context_runtime as direct_context_runtime;
#[path = "../fixtures/support/direct_delivery_runtime.rs"]
pub(crate) mod direct_delivery_runtime;

#[allow(unused_imports)]
use crate::shared_support_query_handoff_fixture as query_handoff_fixture;
#[allow(unused_imports)]
use crate::shared_support_query_handoff_runtime as query_handoff_runtime;
#[allow(unused_imports)]
use crate::shared_support_test_bootstrap_policy as test_bootstrap_policy;

#[allow(unused_imports)]
use crate::shared_support_worth_native_assertions as worth_native_assertions;
#[allow(unused_imports)]
use crate::shared_support_worth_native_runtime as worth_native_runtime;
#[path = "worth_native/declaration_denials.rs"]
pub(crate) mod declaration_denials;

#[path = "worth_native/declaration_intake.rs"]
pub(crate) mod declaration_intake;

#[path = "worth_native/direct_composition.rs"]
pub(crate) mod direct_composition;

#[path = "worth_native/direct_context.rs"]
pub(crate) mod direct_context;

#[path = "worth_native/direct_delivery.rs"]
pub(crate) mod direct_delivery;

#[path = "worth_native/direct_mutation.rs"]
pub(crate) mod direct_mutation;

#[path = "worth_native/direct_projection.rs"]
pub(crate) mod direct_projection;

#[path = "worth_native/direct_runtime.rs"]
pub(crate) mod direct_runtime;

#[path = "worth_native/entry_boundary.rs"]
pub(crate) mod entry_boundary;
