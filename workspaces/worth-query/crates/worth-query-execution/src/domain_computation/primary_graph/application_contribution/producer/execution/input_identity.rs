//! Canonical producer input preparation on the demand's cumulative admission.

#[cfg(test)]
use worth_query_declaration::facade::application_operation::ApplicationMutationIdentities;
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIdentityAdmittedDenial,
    ApplicationMutationIdentityDenial, CanonicalEncodingCharge,
};
use worth_query_installation::facade::ApplicationSchema;

use super::{
    denial, identity_unavailable, InvalidationEditAdmission, WorthQueryOutputDemandDenial,
    WorthQueryOutputDemandDenialKind,
};

#[cfg(test)]
fn encode_input<'input, Schema, Binding>(
    producer: &str,
    key: &'input Binding::IdempotencyKey,
    input: &'input Binding::Input,
    admission: &mut InvalidationEditAdmission,
) -> Result<ApplicationMutationIdentities<'input, Schema, Binding>, WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    ApplicationMutationIdentities::encode_admitted(key, input, &mut |charge| {
        admit_encoding_charge(producer, admission, charge)
    })
    .map_err(|error| encoding_denial(producer, error))
}

pub(super) fn encode_owned_input<Schema, Binding>(
    producer: &str,
    key: Binding::IdempotencyKey,
    input: Binding::Input,
    admission: &mut InvalidationEditAdmission,
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
    worth_query_declaration::facade::application_operation::ApplicationOwnedMutationIdentities::encode_admitted(
        key, input, &mut |charge| admit_encoding_charge(producer, admission, charge),
    ).map_err(|error| encoding_denial(producer, error))
}

fn admit_encoding_charge(
    producer: &str,
    admission: &mut InvalidationEditAdmission,
    charge: CanonicalEncodingCharge,
) -> Result<(), WorthQueryOutputDemandDenial> {
    match charge {
        CanonicalEncodingCharge::Work(units) => {
            admission.charge_external_work(units).map_err(|stop| {
                denial(
                    WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                    format!("{producer}: canonical input preparation: {stop:?}"),
                )
            })
        }
        CanonicalEncodingCharge::Scratch(bytes) => {
            admission.admit_read_scratch(bytes).map_err(|stop| {
                denial(
                    WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
                    format!("{producer}: canonical input preparation: {stop:?}"),
                )
            })
        }
    }
}

fn encoding_denial(
    producer: &str,
    error: ApplicationMutationIdentityAdmittedDenial<WorthQueryOutputDemandDenial>,
) -> WorthQueryOutputDemandDenial {
    match error {
        ApplicationMutationIdentityAdmittedDenial::Key(error) => {
            identity_unavailable(producer, ApplicationMutationIdentityDenial::Key(error))
        }
        ApplicationMutationIdentityAdmittedDenial::Input(error) => {
            identity_unavailable(producer, ApplicationMutationIdentityDenial::Input(error))
        }
        ApplicationMutationIdentityAdmittedDenial::Admission(denial) => denial,
        ApplicationMutationIdentityAdmittedDenial::CapacityOverflow
        | ApplicationMutationIdentityAdmittedDenial::Allocation => denial(
            WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
            format!("{producer}: canonical input capacity unavailable"),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain_computation::primary_graph::tests::fixture::{
        IdentityExecutionSchema, ProgramRequiredInput, ProgramRequiredMutationBinding,
    };
    use worth_relational::facade::mvcc::CompanionPreflightBudget;

    fn admission(work: u64) -> InvalidationEditAdmission {
        InvalidationEditAdmission::new(CompanionPreflightBudget {
            maximum_work_visits: Some(work),
            maximum_preparation_bytes: 1024 * 1024,
        })
    }

    #[test]
    fn owned_request_keeps_the_issued_pair_without_another_encoding_charge() {
        let key = "suspended-owner-request".to_owned();
        let input = ProgramRequiredInput::new("open");
        let plain = ApplicationMutationIdentities::<
            IdentityExecutionSchema,
            ProgramRequiredMutationBinding,
        >::encode(&key, &input)
        .unwrap();
        let expected_key = *plain.key_identity();
        let expected_input = *plain.input_identity();
        let expected_work = plain.canonical_work();
        let mut borrowed_admission = admission(1_000_000);
        encode_input::<IdentityExecutionSchema, ProgramRequiredMutationBinding>(
            "producer",
            &key,
            &input,
            &mut borrowed_admission,
        )
        .unwrap();
        let mut owned_admission = admission(1_000_000);
        let owned = encode_owned_input::<IdentityExecutionSchema, ProgramRequiredMutationBinding>(
            "producer",
            key,
            input,
            &mut owned_admission,
        )
        .unwrap();
        let work = owned_admission.charged_work();
        assert_eq!(work, borrowed_admission.charged_work());
        for _ in 0..2 {
            let borrowed = owned.borrow();
            assert_eq!(*borrowed.key_identity(), expected_key);
            assert_eq!(*borrowed.input_identity(), expected_input);
            assert_eq!(borrowed.canonical_work(), expected_work);
        }
        assert_eq!(owned_admission.charged_work(), work);
    }

    #[test]
    fn input_encoding_preserves_spent_work_and_retries_with_the_exact_remaining_allowance() {
        let key = "one-admitted-input".to_owned();
        let input = ProgramRequiredInput::new("open");
        let mut measured = admission(1_000_000);
        let encoded = encode_input::<IdentityExecutionSchema, ProgramRequiredMutationBinding>(
            "producer",
            &key,
            &input,
            &mut measured,
        )
        .unwrap();
        let plain = ApplicationMutationIdentities::<
            IdentityExecutionSchema,
            ProgramRequiredMutationBinding,
        >::encode(&key, &input)
        .unwrap();
        assert_eq!(encoded, plain);
        assert_eq!(encoded.canonical_work(), plain.canonical_work());
        let required = measured.charged_work();
        assert!(required > 1);

        let mut short = admission(required + 5 - 1);
        short.charge_external_work(5).unwrap();
        let refused = encode_input::<IdentityExecutionSchema, ProgramRequiredMutationBinding>(
            "producer", &key, &input, &mut short,
        )
        .unwrap_err();
        assert_eq!(
            refused.kind(),
            WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
        );
        assert!(
            short.charged_work() > 5,
            "successful hash work remains spent on refusal"
        );

        let mut exact = admission(required + 5);
        exact.charge_external_work(5).unwrap();
        let retried = encode_input::<IdentityExecutionSchema, ProgramRequiredMutationBinding>(
            "producer", &key, &input, &mut exact,
        )
        .unwrap();
        assert_eq!(retried, plain);
        assert_eq!(exact.charged_work(), required + 5);
    }
}
