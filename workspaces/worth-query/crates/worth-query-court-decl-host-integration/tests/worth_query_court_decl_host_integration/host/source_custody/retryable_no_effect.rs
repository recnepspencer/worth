//! Proves `close_unexecuted` strictly rejects post-handler `RetryableNoEffect` retained custody
//! (both typed and erased) while preserving the exact owner for downcast and explicit close.
use super::super::application_home::correction_material::numeric_home::{
    app_limits, contribution::configure_with_edit_handler, model, read_rows, seed_numeric,
    DirectoryOutputProgram,
};
use super::*;
use model::{
    authentication::{authenticate_operator, request_scope},
    edit::{Edit, EditBinding, Edited, MissingValue, Value},
    program::{Action, Program},
    schema::{Key, Number, NumericContribution, NumericSchema, Primary},
};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Barrier,
};
use worth_query_decl::facade::{application_program::*, application_schema::*};
use worth_query_host::facade::{
    application_contribution::{
        with_bootstrap_advancement_from_request, WorthQueryApplicationContribution,
        WorthQueryApplicationContributionContracts, WorthQueryApplicationContributionSetup,
    },
    application_entry::{
        WorthQueryApplicationRequestExt, WorthQueryOperationCloseDenial,
        WorthQuerySourceOnlyPosture,
    },
    application_installation::{ApplicationBuildIdentity, WorthQueryApplicationProgramRoster},
    primary_graph::{
        CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
        WorthQueryInvariantMutationTarget, WorthQueryPrimaryGraphInstallationDenial,
    },
    WorthQueryApplicationHome, WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
};

struct RacingEditHandler {
    contacts: Arc<AtomicUsize>,
    entered_decide: Arc<Barrier>,
    release_decide: Arc<Barrier>,
    pause_value: u64,
}

impl OperationHandler<NumericSchema, EditBinding> for RacingEditHandler {
    fn decide(
        &self,
        input: &Edit,
        reader: &mut DecisionReader<'_, '_, '_, NumericSchema, EditBinding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<NumericSchema, Number>, MissingValue> {
        self.contacts.fetch_add(1, Ordering::SeqCst);
        let result: Result<_, HandlerExecutionDenial> = (|| {
            let entity = reader.resolve_entity(Key::at("Key"), input.key.clone())?;
            let _ = reader.field(&entity, Primary::at("Primary"))?;
            reader.mutation_target(&entity)
        })();
        if matches!(input.value, Value::Primary(v) if v == self.pause_value) {
            self.entered_decide.wait();
            self.release_decide.wait();
        }
        result.into()
    }

    fn build_candidate(
        &self,
        input: &Edit,
        target: WorthQueryInvariantMutationTarget<NumericSchema, Number>,
        writer: &mut CandidateWriter<'_, NumericSchema, EditBinding>,
    ) -> HandlerResult<Edited, MissingValue> {
        let Value::Primary(next) = input.value else {
            return Ok::<_, HandlerExecutionDenial>(Edited).into();
        };
        let result: Result<Edited, HandlerExecutionDenial> = (|| {
            let entity = writer.projected_entity(&target)?;
            writer.write_field(
                &entity,
                Primary::at("Primary"),
                model::schema::Counter(next),
            )?;
            Ok(Edited)
        })();
        result.into()
    }
}

struct RacingContribution;
impl ApplicationSchemaContribution<NumericSchema> for RacingContribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        <NumericContribution as ApplicationSchemaContribution<NumericSchema>>::IDENTITY;
    type Members = <NumericContribution as ApplicationSchemaContribution<NumericSchema>>::Members;
}
impl WorthQueryApplicationContribution<NumericSchema> for RacingContribution {
    type Configuration = RacingEditHandler;
    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<NumericSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        <super::super::application_home::correction_material::numeric_home::contribution::DirectoryNumericContribution as WorthQueryApplicationContribution<NumericSchema>>::contracts(contracts)
    }
    fn configure(
        handler: Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, NumericSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        configure_with_edit_handler(setup, handler)
    }
}

struct RacingProgram;
impl ApplicationProgramDefinition<NumericSchema> for RacingProgram {
    type Contributions = (RacingContribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("numeric-correction.racing-retryable-program.v1");
    fn adoption_authorization<V: ApplicationProgramAdoptionAuthorizationVisitor<NumericSchema>>(
        visitor: V,
    ) -> V::Output {
        visitor.capability::<model::adoption::AdoptBinding>()
    }
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        <Program as ApplicationProgramDefinition<NumericSchema>>::feature_specs()
    }
}

#[test]
fn close_unexecuted_refuses_retryable_no_effect_after_handler_contact() {
    let contacts = Arc::new(AtomicUsize::new(0));
    let entered_decide = Arc::new(Barrier::new(2));
    let release_decide = Arc::new(Barrier::new(2));
    let handler = RacingEditHandler {
        contacts: contacts.clone(),
        entered_decide: entered_decide.clone(),
        release_decide: release_decide.clone(),
        pause_value: 19,
    };

    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let declaration = NumericSchema::declaration().unwrap();
    let program = ApplicationProgramAuthoring::<NumericSchema, RacingProgram>::begin()
        .validated_program()
        .unwrap();
    let roster = WorthQueryApplicationProgramRoster::new().support(
        ApplicationProgramAuthoring::<NumericSchema, DirectoryOutputProgram>::begin()
            .validated_program()
            .unwrap(),
    );
    let request = WorthQueryApplicationHomeRequest::new(
        WorthQueryApplicationHomeLocation::Directory(fixture::location()),
        program,
        roster,
        declaration,
        (handler,),
        ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
        app_limits(),
        |_, graph, installed| seed_numeric(graph, installed),
    );
    let home: WorthQueryApplicationHome<NumericSchema, RacingProgram> =
        with_bootstrap_advancement_from_request(execution, |phase| {
            request.open_in_advancement(&phase)
        })
        .unwrap()
        .unwrap_or_else(|refused| panic!("racing Directory Home opens: {refused}"));
    let branch = home.primary_graph().current_world();

    // Thread A starts an edit (value = 19) and pauses inside `decide` after handler contact.
    // Thread B commits a competing edit (value = 21), forcing Thread A's commit into
    // `ProductBasisStale` -> `WorthQuerySourceOnlyPosture::RetryableNoEffect` (`Pending::Progress`).
    let pending_custody = std::thread::scope(|threads| {
        let loser = threads.spawn(|| {
            let scope = request_scope();
            let principal = authenticate_operator(home.primary_graph().installed_schema(), &scope);
            let req = home.runtime().request(&principal, &scope).on_branch(branch);
            let described = req
                .mutate(Edit {
                    key: "numeric".to_owned(),
                    value: Value::Primary(19),
                })
                .without_source()
                .idempotency_owned("racing-loser-19".to_owned())
                .unwrap_or_else(|(_, _, d)| panic!("capture loser: {d:?}"));
            WorthQuerySourceCustody::<NumericSchema, Edit, RacingProgram, Action>::Described(
                described,
            )
            .resume(home.runtime(), &req)
        });

        entered_decide.wait();
        struct ReleaseGuard<'a>(&'a Barrier);
        impl Drop for ReleaseGuard<'_> {
            fn drop(&mut self) {
                self.0.wait();
            }
        }
        {
            let _release = ReleaseGuard(&release_decide);
            let scope = request_scope();
            let principal = authenticate_operator(home.primary_graph().installed_schema(), &scope);
            let req = home.runtime().request(&principal, &scope).on_branch(branch);
            let winner = req
                .mutate(Edit {
                    key: "numeric".to_owned(),
                    value: Value::Primary(21),
                })
                .without_source()
                .idempotency_owned("racing-winner-21".to_owned())
                .unwrap_or_else(|(_, _, d)| panic!("capture winner: {d:?}"));
            match WorthQuerySourceCustody::<NumericSchema, Edit, RacingProgram, Action>::Described(
                winner,
            )
            .resume(home.runtime(), &req)
            {
                WorthQuerySourceCustodyStep::Committed { .. } => {}
                _ => panic!("competing edit must commit first"),
            }
        }

        match loser.join().unwrap() {
            WorthQuerySourceCustodyStep::Pending { custody, cause } => {
                assert!(matches!(cause, WorthQuerySourceCustodyPending::Progress));
                custody
            }
            _ => {
                panic!("stale unpinned loser must return Pending::Progress with RetryableNoEffect")
            }
        }
    });

    assert_eq!(
        contacts.load(Ordering::SeqCst),
        2,
        "both loser and winner handlers ran"
    );
    assert!(pending_custody.has_durable_original());

    let scope = request_scope();
    let principal = authenticate_operator(home.primary_graph().installed_schema(), &scope);
    let fresh = home.runtime().request(&principal, &scope).on_branch(branch);

    // 1. Typed `close_unexecuted` MUST refuse post-handler `RetryableNoEffect` intact.
    let (typed, denial) = pending_custody
        .close_unexecuted(&fresh)
        .expect_err("typed close_unexecuted must refuse post-handler RetryableNoEffect");
    assert!(matches!(
        denial,
        WorthQueryOperationCloseDenial::NativeObligationRetained
    ));

    // 2. Erased `close_unexecuted` MUST also refuse and preserve the exact owner for downcast.
    let (erased, denial) = typed
        .erase()
        .close_unexecuted(&fresh)
        .expect_err("erased close_unexecuted must refuse post-handler RetryableNoEffect");
    assert!(matches!(
        denial,
        WorthQueryOperationCloseDenial::NativeObligationRetained
    ));
    assert!(erased.has_durable_original());

    let WorthQuerySourceCustody::Retained(operation) = erased
        .try_into_typed::<Edit, Action>()
        .unwrap_or_else(|_| panic!("downcast must recover exact post-handler Retained owner"))
    else {
        panic!("downcast must preserve Retained variant");
    };
    assert_eq!(
        operation.posture(),
        WorthQuerySourceOnlyPosture::RetryableNoEffect
    );
    assert!(operation.native_retry_outcome().is_some());

    // 3. Publication's separate explicit `close` for proven no-effect attempts still succeeds.
    operation
        .close(&fresh)
        .unwrap_or_else(|(_, d)| panic!("explicit proven no-effect close: {d:?}"));
    assert_eq!(read_rows(home.runtime(), branch)[0].primary, 21);
    assert_eq!(contacts.load(Ordering::SeqCst), 2);

    drop(principal);
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("racing Home closes cleanly: {refused:?}"))
        .reclaim();
}
