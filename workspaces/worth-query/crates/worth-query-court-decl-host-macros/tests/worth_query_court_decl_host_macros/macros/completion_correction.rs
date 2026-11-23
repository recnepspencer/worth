//! An authored inverse of a recorded board completion.
use super::{actions::*, reads::*, schema::*};
use worth_foundational::facade::WorthValue;
use worth_query_decl::facade::{
    application_aftermath::*, application_identity::ApplicationIdentityNamespace,
    application_operation::*, application_query::*, application_schema::*,
    worth_query_application_query, worth_query_identity, worth_query_identity_namespace,
    worth_query_query_binding, worth_query_structured_value_binding, WorthQueryAction,
};

const CORRECTION: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("macros.tasks.correct-completion.v1");
const DETAILS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("macros.tasks.correction-details.v1");
const RESTORE: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("macros.tasks.restore-done.v1");

#[derive(Clone, WorthValue, serde::Deserialize, WorthQueryAction)]
#[worth_query(schema = Schema, context = BoardContext, namespace = RESTORE,
    input = RestoreDoneInput, operation = RestoreDone, binding = RestoreDoneBinding,
    delivery = RestoredDone, body, program, reads(Task))]
pub struct RestoreCompletion {
    #[worth_query(scope = TaskKey)]
    task: u64,
    expected: bool,
    done: bool,
}

pub fn restore_done() -> ApplicationOperationDefinition<Schema, RestoreDone, RestoreCompletion> {
    let body = RestoreDone::reference()
        .body()
        .expect_input(TaskFactsValue::select_done(), "expected")
        .unwrap()
        .assign_input(TaskFactsValue::select_done(), "done")
        .unwrap()
        .finish()
        .unwrap();
    RestoreDone::reference()
        .definition()
        .no_external_effect()
        .no_aftermath()
        .finish()
        .with_body(body)
        .unwrap()
}

pub struct CorrectCompletion;

impl ApplicationCorrectionBinding<Schema> for CorrectCompletion {
    const IDENTITY: &'static str = worth_query_identity!(CORRECTION, binding);
    type Source = CompleteBinding;
    type InverseIntent = RestoreCompletion;
    type Disclosure = DetailsBinding;

    fn recover_source_input(
        bytes: &[u8],
        expected: CanonicalDigestId,
        bounds: ValueDecodeBounds,
    ) -> Result<ApplicationEncodedInput<CompleteInput>, ApplicationRetainedInputDenial> {
        ApplicationEncodedInput::decode_retained(bytes, expected, bounds)
    }

    fn recover_inverse_input(
        bytes: &[u8],
        expected: CanonicalDigestId,
        bounds: ValueDecodeBounds,
    ) -> Result<ApplicationEncodedInput<RestoreDoneInput>, ApplicationRetainedInputDenial> {
        ApplicationEncodedInput::decode_retained(bytes, expected, bounds)
    }

    fn disclosure_parameters(
        _: &Complete,
    ) -> Result<ApplicationQueryParameterSet<DetailsQuery>, ApplicationCorrectionDerivationDenial>
    {
        Ok(ApplicationQueryParameterSet::new())
    }

    fn source_definition(
    ) -> ApplicationOperationDefinitionBuilder<Schema, CompleteTask, Complete, true, false> {
        CompleteTask::reference().definition().no_external_effect()
    }

    fn source_body() -> Option<ApplicationOperationBody<Schema, CompleteTask, Complete>> {
        Some(complete_body())
    }

    fn mechanism() -> DeclaredCorrectionMechanism<Schema> {
        let bytes = BoolApplicationValueBinding::encode(&true)
            .unwrap()
            .semantic_byte_width();
        DeclaredCorrectionMechanism::recorded_inverse_binding::<Self>(DeclaredPreImageDemand::new(
            [DeclaredPreImageLocus::from_field(
                TaskFactsValue::select_done(),
            )],
            bytes,
        ))
    }

    fn derive_correction(
        original: &Complete,
        material: ApplicationCorrectionSourceMaterial<'_, Schema>,
    ) -> Result<RestoreCompletion, ApplicationCorrectionDerivationDenial> {
        use ApplicationCorrectionDerivationDenial as Denial;
        let ApplicationCorrectionSourceMaterial::RecordedInverse(preimage) = material else {
            return Err(Denial::UnsupportedSourceMeaning);
        };
        let done = preimage
            .field(&DeclaredPreImageLocus::from_field(
                TaskFactsValue::select_done(),
            ))
            .ok_or(Denial::MissingPreImage)?;
        Ok(RestoreCompletion {
            task: original.task,
            expected: original.done,
            done: BoolApplicationValueBinding::decode(done).map_err(|_| Denial::InvalidPreImage)?,
        })
    }

    fn inverse_key(key: &str) -> Result<u64, ApplicationValueEncodeDenial> {
        key.parse().map_err(|_| ApplicationValueEncodeDenial::CodecRejected {
            binding_identity: worth_query_decl::facade::portable_identity::WorthQueryPortableTypeIdentity::declared(Self::IDENTITY),
            reason: "expected an unsigned command key",
        })
    }

    fn visit<V: ApplicationCorrectionVisitor<Schema>>(
        dispatch: ApplicationCorrectionDispatch<Schema, Self, V>,
    ) -> V::Output {
        dispatch.conventional()
    }
}

worth_query_application_query!(pub DetailsQuery for Schema,
    identity worth_query_identity!(DETAILS, query), result RowBinding,
    scope Task => "Task", name "correction_task");

pub fn task_details() -> ApplicationQueryDefinitionBuilder<Schema, DetailsQuery, (), TaskRow, Task>
{
    ApplicationQueryDefinitionBuilder::declare(DetailsQuery::reference())
        .root(Task::reference())
        .scope(Task::reference())
        .result_shape(TaskRow::result_shape::<DetailsQuery>())
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
}

#[derive(Clone, serde::Serialize)]
pub struct ReadDetails(pub u64);
worth_query_structured_value_binding!(pub ReadDetailsInput for ReadDetails {
    identity: worth_query_identity!(DETAILS, request)
});
fn task(input: ReadDetails) -> u64 {
    input.0
}
worth_query_query_binding!(pub DetailsBinding for ReadDetails, schema Schema, context BoardContext,
    identity worth_query_identity!(DETAILS, binding), input ReadDetailsInput, query DetailsQuery,
    parameters => |_| ApplicationQueryParameterSet::new(),
    scope TaskKey, field TaskKey::at("TaskKey"), value task, limits results 1);
