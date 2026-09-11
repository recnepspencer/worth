use std::marker::PhantomData;

use worth_query_declaration::facade::application_schema::{
    ApplicationEffectMarkerIdentity, ApplicationRetainedEffectBinding,
};

#[derive(Clone)]
pub(super) struct TestPayload;

pub(super) struct TestEffect<Schema>(PhantomData<Schema>);

worth_query_declaration::worth_query_portable_type!(
    TestPayload => "worth.query.installation-test.effect-payload"
);
worth_query_declaration::worth_query_structured_value_binding!(
    pub(super) TestPayloadBinding for TestPayload { identity: "worth.query.installation-test.effect-payload" }
);

impl ApplicationRetainedEffectBinding for TestPayloadBinding {
    fn retained_bytes(_value: &Self::Value) -> u64 {
        0
    }
}

impl<Schema> ApplicationEffectMarkerIdentity<Schema> for TestEffect<Schema> {
    type PayloadBinding = TestPayloadBinding;
    const IDENTIFIER: &'static str = "TestEffect";
}
