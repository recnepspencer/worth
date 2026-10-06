//! Canonical producer input preparation on the demand's cumulative admission.

use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIdentities,
    ApplicationMutationIdentityAdmittedDenial, ApplicationMutationIdentityDenial,
    CanonicalEncodingCharge,
};
use worth_query_installation::facade::ApplicationSchema;

use super::{
    denial, identity_unavailable, InvalidationEditAdmission, WorthQueryOutputDemandDenial,
    WorthQueryOutputDemandDenialKind,
};

pub(super) fn encode_input<'input, Schema, Binding>(
    producer: &str,
    key: &'input Binding::IdempotencyKey,
    input: &'input Binding::Input,
    admission: &mut InvalidationEditAdmission,
) -> Result<ApplicationMutationIdentities<'input, Schema, Binding>, WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    ApplicationMutationIdentities::encode_admitted(key, input, &mut |charge| match charge {
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
    })
    .map_err(|error| match error {
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
    })
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
            maximum_work_visits: work,
            maximum_preparation_bytes: 1024 * 1024,
        })
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
