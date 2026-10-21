//! Canonical producer input preparation on the demand's cumulative admission.

use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
#[cfg(test)]
use worth_query_declaration::facade::application_operation::ApplicationMutationIdentities;
use worth_query_installation::facade::ApplicationSchema;

use super::identity_unavailable;
use super::WorthQueryOutputDemandDenial;

#[cfg(test)]
fn encode_input<'input, Schema, Binding>(
    producer: &str,
    key: &'input Binding::IdempotencyKey,
    input: &'input Binding::Input,
) -> Result<ApplicationMutationIdentities<'input, Schema, Binding>, WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    ApplicationMutationIdentities::encode(key, input)
        .map_err(|error| identity_unavailable(producer, error))
}

pub(super) fn encode_owned_input<Schema, Binding>(
    producer: &str,
    key: Binding::IdempotencyKey,
    input: Binding::Input,
) -> Result<
    worth_query_declaration::facade::application_operation::ApplicationOwnedMutationIdentities<
        Schema,
        Binding,
    >,
    WorthQueryOutputDemandDenial,
>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    worth_query_declaration::facade::application_operation::ApplicationOwnedMutationIdentities::encode(key, input).map_err(|error| identity_unavailable(producer, error))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
    use crate::domain_computation::primary_graph::tests::fixture::ProgramRequiredInput;
    use crate::domain_computation::primary_graph::tests::fixture::ProgramRequiredMutationBinding;

    #[test]
    fn owned_request_preserves_the_same_issued_key_and_input_identity() {
        let key = "suspended-owner-request".to_owned();
        let input = ProgramRequiredInput::new("open");
        let plain = ApplicationMutationIdentities::<
            IdentityExecutionSchema,
            ProgramRequiredMutationBinding,
        >::encode(&key, &input)
        .unwrap();
        let expected_key = *plain.key_identity();
        let expected_input = *plain.input_identity();
        let borrowed = encode_input::<IdentityExecutionSchema, ProgramRequiredMutationBinding>(
            "producer", &key, &input,
        )
        .unwrap();
        assert_eq!(borrowed.key_identity(), &expected_key);
        assert_eq!(borrowed.input_identity(), &expected_input);
        let owned = encode_owned_input::<IdentityExecutionSchema, ProgramRequiredMutationBinding>(
            "producer", key, input,
        )
        .unwrap();
        for _ in 0..2 {
            let borrowed = owned.borrow();
            assert_eq!(borrowed.key_identity(), &expected_key);
            assert_eq!(borrowed.input_identity(), &expected_input);
        }
    }
}
