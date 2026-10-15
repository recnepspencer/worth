#[allow(unused_imports)]
use crate::shared_support_direct_context_runtime as direct_context_runtime;
#[path = "support/direct_delivery_runtime.rs"]
mod direct_delivery_runtime;
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
mod declaration_denials;
#[path = "worth_native/declaration_intake.rs"]
mod declaration_intake;
#[path = "worth_native/direct_composition.rs"]
mod direct_composition;
#[path = "worth_native/direct_context.rs"]
mod direct_context;
#[path = "worth_native/direct_delivery.rs"]
mod direct_delivery;
#[path = "worth_native/direct_mutation.rs"]
mod direct_mutation;
#[path = "worth_native/direct_projection.rs"]
mod direct_projection;
#[path = "worth_native/direct_runtime.rs"]
mod direct_runtime;
#[path = "worth_native/entry_boundary.rs"]
mod entry_boundary;
