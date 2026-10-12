use bank_domain::estate::{EmergencyAccessId, EstateCaseId};
use bank_server::{
    queries, BankApplicationQueryDenial, BankApprovedEstateElevation, BankAuthenticatedPrincipal,
    BankEstateEmergencyAccessActivityContinuation, BankIdentityRuntime, BankPreviewSession,
    BankReadControls,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationLiveControls, WorthQueryApplicationQueryResumeControls,
};

pub struct EmergencyEstateQueryInputs<'a> {
    pub estate: EstateCaseId,
    pub access: EmergencyAccessId,
    pub approved: &'a BankApprovedEstateElevation,
    pub preview: &'a BankPreviewSession,
    pub continuation: BankEstateEmergencyAccessActivityContinuation,
    pub readmission_continuation: BankEstateEmergencyAccessActivityContinuation,
    pub resume: WorthQueryApplicationQueryResumeControls<'a>,
    pub readmission_resume: WorthQueryApplicationQueryResumeControls<'a>,
    pub live: WorthQueryApplicationLiveControls,
}

pub fn exercise_ordinary_estate_queries(
    runtime: &BankIdentityRuntime,
    principal: &BankAuthenticatedPrincipal,
    controls: &BankReadControls,
    estate: EstateCaseId,
) -> Vec<Result<(), BankApplicationQueryDenial>> {
    vec![
        runtime
            .query(queries::estate_case(estate))
            .as_principal(principal)
            .controls(controls.clone())
            .execute()
            .map(|_| ()),
        runtime
            .query(queries::estate_customer_identity(estate))
            .as_principal(principal)
            .controls(controls.clone())
            .execute()
            .map(|_| ()),
        runtime
            .query(queries::estate_governance_context(estate))
            .as_principal(principal)
            .controls(controls.clone())
            .execute()
            .map(|_| ()),
        runtime
            .query(queries::estate_legal_compliance(estate))
            .as_principal(principal)
            .controls(controls.clone())
            .execute()
            .map(|_| ()),
        runtime
            .query(queries::estate_mandatory_reviews(estate))
            .as_principal(principal)
            .controls(controls.clone())
            .execute()
            .map(|_| ()),
    ]
}

pub fn exercise_emergency_estate_queries(
    runtime: &BankIdentityRuntime,
    principal: &BankAuthenticatedPrincipal,
    controls: &BankReadControls,
    inputs: EmergencyEstateQueryInputs<'_>,
) -> Vec<Result<(), BankApplicationQueryDenial>> {
    let mut outcomes = Vec::new();
    outcomes.push(emergency_details(runtime, principal, controls, &inputs).map(|_| ()));
    outcomes.extend(emergency_activity(runtime, principal, controls, inputs));
    outcomes
}

fn emergency_details(
    runtime: &BankIdentityRuntime,
    principal: &BankAuthenticatedPrincipal,
    controls: &BankReadControls,
    inputs: &EmergencyEstateQueryInputs<'_>,
) -> Result<(), BankApplicationQueryDenial> {
    runtime
        .query(queries::estate_emergency_account_details(
            inputs.estate,
            inputs.access,
        ))
        .as_principal(principal)
        .controls(controls.clone())
        .execute_with_approved_elevation(inputs.approved)?;
    runtime
        .query(queries::estate_emergency_account_details(
            inputs.estate,
            inputs.access,
        ))
        .as_principal(principal)
        .controls(controls.clone())
        .admit_historical_with_approved_elevation(inputs.approved, |phase, admitted| {
            admitted.execute(phase)
        })?;
    runtime
        .query(queries::estate_emergency_account_details(
            inputs.estate,
            inputs.access,
        ))
        .as_principal(principal)
        .controls(controls.clone())
        .admit_preview_with_approved_elevation(
            inputs.approved,
            inputs.preview,
            |phase, admitted| admitted.execute(phase),
        )?;
    Ok(())
}

fn emergency_activity(
    runtime: &BankIdentityRuntime,
    principal: &BankAuthenticatedPrincipal,
    controls: &BankReadControls,
    inputs: EmergencyEstateQueryInputs<'_>,
) -> Vec<Result<(), BankApplicationQueryDenial>> {
    let mut outcomes = bounded_emergency_activity(runtime, principal, controls, &inputs);
    outcomes.extend(resumable_emergency_activity(
        runtime, principal, controls, inputs,
    ));
    outcomes
}

fn bounded_emergency_activity(
    runtime: &BankIdentityRuntime,
    principal: &BankAuthenticatedPrincipal,
    controls: &BankReadControls,
    inputs: &EmergencyEstateQueryInputs<'_>,
) -> Vec<Result<(), BankApplicationQueryDenial>> {
    let request = || queries::estate_emergency_access_activity(inputs.estate, inputs.access);
    vec![
        runtime
            .query(request())
            .as_principal(principal)
            .controls(controls.clone())
            .execute_with_approved_elevation(inputs.approved)
            .map(|_| ()),
        runtime
            .query(request())
            .as_principal(principal)
            .controls(controls.clone())
            .admit_historical_with_approved_elevation(inputs.approved, |phase, admitted| {
                admitted.execute(phase)
            })
            .map(|_| ()),
        runtime
            .query(request())
            .as_principal(principal)
            .controls(controls.clone())
            .admit_preview_with_approved_elevation(
                inputs.approved,
                inputs.preview,
                |phase, admitted| admitted.execute(phase),
            )
            .map(|_| ()),
        runtime
            .query(request())
            .as_principal(principal)
            .controls(controls.clone())
            .page_with_approved_elevation(inputs.approved)
            .map(|_| ()),
    ]
}

fn resumable_emergency_activity(
    runtime: &BankIdentityRuntime,
    principal: &BankAuthenticatedPrincipal,
    controls: &BankReadControls,
    inputs: EmergencyEstateQueryInputs<'_>,
) -> Vec<Result<(), BankApplicationQueryDenial>> {
    let EmergencyEstateQueryInputs {
        estate,
        access,
        approved,
        continuation,
        readmission_continuation,
        resume,
        readmission_resume,
        live,
        preview: _,
    } = inputs;
    let request = || queries::estate_emergency_access_activity(estate, access);
    vec![
        runtime
            .query(request())
            .as_principal(principal)
            .controls(controls.clone())
            .resume_with_approved_elevation(approved, continuation, resume)
            .map(|_| ()),
        runtime
            .query(request())
            .as_principal(principal)
            .controls(controls.clone())
            .readmit_resume_with_approved_elevation(
                approved,
                readmission_continuation,
                readmission_resume,
                |phase, admitted| admitted.execute(phase),
            )
            .map(|_| ()),
        runtime
            .query(request())
            .as_principal(principal)
            .controls(controls.clone())
            .subscribe_with_approved_elevation(approved, live)
            .map(|_| ()),
    ]
}
