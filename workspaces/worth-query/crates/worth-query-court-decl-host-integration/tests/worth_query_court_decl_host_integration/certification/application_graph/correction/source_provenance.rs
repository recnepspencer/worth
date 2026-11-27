//! A session explains what an authored operation of its own branch wrote,
//! field by field, under its own admission; it refuses an ungranted request
//! and a receipt committed on another branch.
use super::model::{
    authentication::{authenticate_operator, request_scope},
    causal_inspection::{capability, SourceDisclosure},
    edit::Value,
    host,
    schema::NumericSchema,
};
use worth_query_host::facade::{
    application_entry::WorthQueryApplicationRequestExt,
    audit::{
        WorthQueryGovernedReadAccess, WorthQuerySourceProvenanceDenial,
        WorthQuerySourceProvenanceRequest,
    },
    causal_inspection::{
        WorthQueryApplicationMaterializationRequest, WorthQueryMaterializedFamilyOutcome,
    },
    runtime::ExecutionAllocationPolicy,
    WorthQueryRefusalCategory, WorthQueryUserFacingRefusal,
};

type Access = WorthQueryGovernedReadAccess<
    NumericSchema,
    capability::Inspect,
    capability::InspectCausalEvidence,
    capability::Input,
>;

fn access(key: &str) -> Access {
    WorthQueryGovernedReadAccess::new(
        capability::Inspect::reference(),
        capability::InspectCausalEvidence::reference(),
        capability::Input::at(key),
    )
}

fn inspect(
    key: &str,
) -> WorthQuerySourceProvenanceRequest<
    NumericSchema,
    capability::Inspect,
    capability::InspectCausalEvidence,
    capability::Input,
    capability::Inspect,
    capability::InspectCausalEvidence,
    capability::Input,
> {
    WorthQuerySourceProvenanceRequest {
        metadata: access(key),
        diagnostic: access(key),
    }
}

#[test]
fn a_session_explains_its_own_branch_operation_and_refuses_others() {
    let guard = host::open_directory(host::unique_directory("source-provenance"));
    let home = guard.home();
    let scope = request_scope();
    let principal = authenticate_operator(home.runtime().installed_schema(), &scope);
    let session = home.session(&principal);
    let receipt = host::edit(home.runtime(), Value::Primary(11), "provenance-original");

    let provenance = session
        .source_provenance::<SourceDisclosure>(&receipt, inspect("numeric"))
        .unwrap();
    assert_eq!(provenance.source_commit(), receipt.commit_reference());
    assert_eq!(
        provenance.source_effects_outcome(),
        WorthQueryMaterializedFamilyOutcome::Provided
    );
    // The same receipt through the plain inspection terminal is the oracle.
    let graph = home.primary_graph();
    let request = graph.request(&principal, &scope);
    let (metadata, diagnostic) = graph
        .with_application_advancement(&scope, |phase| {
            (
                request
                    .admit_read_access_in_advancement::<
                        <SourceDisclosure as worth_query_host::facade::declaration::application_operation::ApplicationSourceDisclosureBinding<NumericSchema>>::Disclosure,
                        _, _, _,
                    >(&phase, &access("numeric"))
                    .unwrap(),
                request
                    .admit_read_access_in_advancement::<
                        <SourceDisclosure as worth_query_host::facade::declaration::application_operation::ApplicationSourceDisclosureBinding<NumericSchema>>::Disclosure,
                        _, _, _,
                    >(&phase, &access("numeric"))
                    .unwrap(),
            )
        })
        .unwrap();
    let direct = request
        .inspect_source::<SourceDisclosure>(&receipt)
        .why_performed()
        .materialize(WorthQueryApplicationMaterializationRequest::source_effects())
        .with_allocation_policy(ExecutionAllocationPolicy::SystemAllocation)
        .execute_governed(metadata, diagnostic)
        .unwrap();
    let observed: Vec<_> = provenance
        .fields()
        .map(|field| {
            (
                field.result_path().to_owned(),
                field.effect(),
                field.value().cloned(),
            )
        })
        .collect();
    let expected: Vec<_> = direct
        .materialization()
        .source_effects()
        .unwrap()
        .fields()
        .map(|field| {
            (
                field.result_path().to_owned(),
                field.effect(),
                field.value().cloned(),
            )
        })
        .collect();
    assert!(!observed.is_empty());
    assert_eq!(observed, expected);
    assert!(
        observed.iter().any(|(_, _, value)| value.is_some()),
        "{observed:?}"
    );
    let first = &observed[0].0;
    assert_eq!(
        provenance.field(first).map(|field| field.result_path()),
        Some(first.as_str())
    );

    // No grant covers another key: admission refuses before inspection.
    let ungranted = session.source_provenance::<SourceDisclosure>(&receipt, inspect("absent"));
    assert!(
        matches!(
            ungranted,
            Err(WorthQuerySourceProvenanceDenial::Admission(_))
        ),
        "{ungranted:?}"
    );

    // A preview's write is explained only by a session of the preview branch.
    let preview = home.fork_preview("provenance what-if").unwrap();
    let elsewhere = host::edit_on(
        home.runtime(),
        preview.branch(),
        Value::Primary(12),
        "provenance-preview",
    );
    let foreign = session.source_provenance::<SourceDisclosure>(&elsewhere, inspect("numeric"));
    assert!(
        matches!(
            foreign,
            Err(WorthQuerySourceProvenanceDenial::ForeignBranch)
        ),
        "{foreign:?}"
    );
    let on_preview = home.session(&principal).on_branch(preview.branch());
    assert!(on_preview
        .source_provenance::<SourceDisclosure>(&elsewhere, inspect("numeric"))
        .is_ok());
    drop((on_preview, elsewhere, direct, provenance));
    preview.discard(home).unwrap();
}

#[test]
fn every_work_budget_stop_admission_or_inspection_refuses_as_capacity() {
    let guard = host::open_directory(host::unique_directory("source-provenance-budget"));
    let home = guard.home();
    let scope = request_scope();
    let principal = authenticate_operator(home.runtime().installed_schema(), &scope);
    let receipt = host::edit(home.runtime(), Value::Primary(13), "provenance-budget");
    // Grow the work budget until the read completes: small budgets stop at
    // capability admission, larger ones inside the admitted inspection.
    let mut work = 1_u64;
    let mut refused = Vec::new();
    let provenance = loop {
        let budget = worth_foundational::facade::ExecutionBudget::new(
            std::num::NonZeroUsize::MIN,
            Some(128 * 1024 * 1024),
            Some(work),
        );
        match home
            .session(&principal)
            .with_budget(budget)
            .source_provenance::<SourceDisclosure>(&receipt, inspect("numeric"))
        {
            Ok(provenance) => break provenance,
            Err(denial) => refused.push(denial),
        }
        assert!(work < 1 << 40, "no finite budget completed the read");
        work += work / 4 + 1;
    };
    let stopped_in = |inspection: bool| {
        refused.iter().any(|denial| {
            matches!(denial, WorthQuerySourceProvenanceDenial::Inspection(_)) == inspection
        })
    };
    assert!(stopped_in(false), "no budget stopped at admission");
    assert!(stopped_in(true), "no budget stopped inside the inspection");
    for denial in &refused {
        assert!(!denial.is_unsupported_field_shape(), "{denial:?}");
        // A budget stop is a resource stop, never a permission refusal or
        // an internal defect, wherever it lands.
        assert!(
            matches!(
                denial.refusal_category(),
                WorthQueryRefusalCategory::Capacity | WorthQueryRefusalCategory::Unavailable
            ),
            "{denial:?} renders {:?}",
            denial.refusal_category()
        );
        assert!(denial.retryable());
        assert!(!denial.user_message().contains("WorthQuery"));
    }
    assert_eq!(
        provenance.source_effects_outcome(),
        WorthQueryMaterializedFamilyOutcome::Provided
    );
}
