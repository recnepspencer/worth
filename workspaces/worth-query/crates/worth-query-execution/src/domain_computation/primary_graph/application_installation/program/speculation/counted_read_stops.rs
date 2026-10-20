mod custody;

use super::*;
use crate::basis::WorthQueryProductBranchAdmissionDenial as Branch;
use crate::domain_computation::{
    WorthQueryProviderSessionControlStopKind as Control,
    WorthQueryProviderSessionDenialKind as Kind,
};
use crate::facade::application_installation::WorthQueryProgramOwner;
use crate::facade::application_installation::{
    in_memory_program, WorthQueryInMemoryApplicationLimits,
};
use worth_query_declaration::facade::application_program::*;
use worth_query_declaration::facade::application_schema::U64ApplicationValueBinding;
use worth_query_declaration::facade::authentication::{
    WorthQueryExternalPrincipalIdentity, WorthQueryExternalPrincipalIdentityBinding,
    WorthQueryPrincipalMappingStatus, WorthQueryPrincipalMappingStatusBinding,
};

worth_query_declaration::worth_query_application! {
    PreviewSchema { owner: "worth.query.test.preview", version: (1, 0), contributions: [PreviewMembers], }
}
worth_query_declaration::worth_query_entity!(PreviewEntity for PreviewSchema);
worth_query_declaration::worth_query_entity!(PreviewMapping for PreviewSchema);
worth_query_declaration::worth_query_aspect!(MappingAspect for PreviewSchema, PreviewMapping; identity = AspectIdentity(0x720001), revision = AspectContractRevision(1),);
worth_query_declaration::worth_query_aspect!(PrincipalAspect for PreviewSchema, PreviewEntity; identity = AspectIdentity(0x720002), revision = AspectContractRevision(1),);
worth_query_declaration::worth_query_field!(MappingIdentity for PreviewSchema, PreviewMapping, MappingAspect: WorthQueryExternalPrincipalIdentity => WorthQueryExternalPrincipalIdentityBinding, read_only, equality);
worth_query_declaration::worth_query_field!(MappingStatus for PreviewSchema, PreviewMapping, MappingAspect: WorthQueryPrincipalMappingStatus => WorthQueryPrincipalMappingStatusBinding, read_write, equality);
worth_query_declaration::worth_query_field!(PrincipalIdentity for PreviewSchema, PreviewEntity, PrincipalAspect: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_declaration::worth_query_relation!(MappingTarget in PreviewSchema, PreviewMapping => PreviewEntity; integrity = same_context_unbounded_retain_dangling);
worth_query_declaration::worth_query_principal_binding!(PreviewBinding in PreviewSchema, mapping PreviewMapping {
    identity: MappingIdentity, status: MappingStatus, target: MappingTarget => PreviewEntity, principal_identity: PrincipalIdentity
});
worth_query_declaration::worth_query_application_contribution! {
    contribution PreviewMembers in PreviewSchema {
        identity: "worth.query.test.preview-members", members: |builder| {
            builder.entity(PreviewEntity::reference()).entity(PreviewMapping::reference())
                .aspect(PreviewMapping::reference(), MappingAspect::reference())
                .aspect(PreviewEntity::reference(), PrincipalAspect::reference())
                .field(PreviewMapping::reference(), MappingIdentity::reference())
                .field(PreviewMapping::reference(), MappingStatus::reference())
                .field(PreviewEntity::reference(), PrincipalIdentity::reference())
                .relation(MappingTarget::reference(), PreviewMapping::reference(), PreviewEntity::reference())
                .principal_binding(PreviewBinding::reference())
        }
    }
}
impl crate::domain_computation::primary_graph::WorthQueryApplicationContribution<PreviewSchema>
    for PreviewMembers
{
    type Configuration = ();
    fn configure(
        _: (),
        _: &mut crate::domain_computation::primary_graph::WorthQueryApplicationContributionSetup<
            '_,
            PreviewSchema,
        >,
    ) -> Result<
        (),
        crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenial,
    > {
        Ok(())
    }
}
struct PreviewFeature;
impl ApplicationFeature<PreviewSchema> for PreviewFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.test.preview-feature";
}
struct PreviewProgram;
impl ApplicationProgramDefinition<PreviewSchema> for PreviewProgram {
    type Contributions = (PreviewMembers,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.test.preview-program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![ApplicationFeatureSpec::root::<PreviewSchema, PreviewFeature>().finish()]
    }
}

#[test]
fn begin_preview_preserves_admission_stops_at_every_consult() {
    let runtime = installed_preview_runtime();
    let observation = runtime
        .with_host_advancement(|phase| {
            runtime
                .on_branch(runtime.current_world())
                .select(&phase)
                .unwrap()
                .retain_application_read()
        })
        .unwrap();
    let identity = std::cell::Cell::new(0);
    crate::test_read_stops::every_advancement_consult(
        |request, ceiling| runtime.with_application_advancement(request, |phase| {
            let ordinal = identity.get();
            identity.set(ordinal + 1);
            phase.execution_request_for(&runtime.product_runtime).unwrap().run(worth_execution::ExecutionWorkCeiling::new(Some(ceiling)), |_| {
                runtime.begin_preview(&phase, &observation, WorthQueryApplicationPreviewRequest::new(format!("counted-preview-{ordinal}")))
                .map(|session| session.discard().unwrap())
                .inspect_err(|denial| {
                    if let WorthQueryApplicationPreviewReadmissionDenial::SourceSelection(cause) = denial {
                        assert!(!cause.is_transient(), "a request stop cannot defer admission");
                    }
                })
            }).unwrap().0
        }).unwrap(),
        WorthQueryApplicationPreviewReadmissionDenial::SourceSelection(Branch::Stopped(Err(Control::Cancelled))),
        WorthQueryApplicationPreviewReadmissionDenial::SourceSelection(Branch::Stopped(Ok(Kind::ExecutionResource {
            denial: crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial::WorkExhausted,
            partition_identity: None, policy_ancestor: None,
        }))),
    );
}

fn installed_preview_runtime() -> WorthQueryProgramApplicationRuntime<PreviewSchema, PreviewProgram>
{
    let program = ApplicationProgramAuthoring::<PreviewSchema, PreviewProgram>::begin()
        .validated_program()
        .unwrap();
    in_memory_program(
        program, PreviewSchema::declaration().unwrap(), ((),),
        WorthQueryInMemoryApplicationLimits::new(
            crate::domain_computation::execution_runtime::product_world::test_product_world_resources(),
            Default::default(),
            worth_signal::facade::runtime::SignalConditionalEvaluationBudget::development(),
        ),
        |_, graph, installed| {
            let binding = installed.principal_binding(PreviewBinding::reference()).unwrap();
            graph.bind_principal(&binding, crate::domain_computation::primary_graph::WorthQueryApplicationPrincipalKey::new("preview-owner").unwrap(), 1_u64,
                WorthQueryExternalPrincipalIdentity::new("preview", "owner").unwrap(), WorthQueryPrincipalMappingStatus::Enabled)?;
            Ok(())
        },
    ).unwrap()
}

#[test]
fn readmit_preview_keeps_source_selection_stops_at_every_consult() {
    let runtime = installed_preview_runtime();
    let observation = runtime
        .with_host_advancement(|phase| {
            runtime
                .on_branch(runtime.current_world())
                .select(&phase)
                .unwrap()
                .retain_application_read()
        })
        .unwrap();
    let identity = std::cell::Cell::new(0);
    crate::test_read_stops::every_advancement_consult_with_door(
        |counted| {
            let ordinal = identity.get(); identity.set(ordinal + 1);
            let preview = runtime.with_host_advancement(|phase| runtime.begin_preview(&phase, &observation, WorthQueryApplicationPreviewRequest::new(format!("readmit-preview-{ordinal}")))).unwrap().unwrap();
            let preview = std::cell::RefCell::new(Some(preview));
            counted(&|request, ceiling| runtime.with_application_advancement(request, |phase| {
                phase.execution_request_for(&runtime.product_runtime).unwrap().run(worth_execution::ExecutionWorkCeiling::new(Some(ceiling)), |_| preview.borrow_mut().take().unwrap().readmit(&phase, &runtime).map(|readmitted| readmitted.discard().unwrap()).map_err(|(preview, denial)| { preview.discard().unwrap(); denial })).unwrap().0
            }).unwrap())
        },
        WorthQueryApplicationPreviewReadmissionDenial::SourceSelection(Branch::Stopped(Err(Control::Cancelled))),
        WorthQueryApplicationPreviewReadmissionDenial::SourceSelection(Branch::Stopped(Ok(Kind::ExecutionResource { denial: crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial::WorkExhausted, partition_identity: None, policy_ancestor: None }))),
    );
}

#[test]
fn support_retirement_keeps_read_stops_inventory_and_active_roster_at_every_consult() {
    use crate::facade::application_installation::WorthQueryProgramSupportRetirementFailure as Failure;
    use worth_query_installation::facade::WorthQueryProgramSupportRetirementDenial as Policy;
    let runtime = installed_preview_runtime();
    let revision = *runtime.owned_revision();
    crate::test_read_stops::every_consult(
        |execution| match runtime.retire_program_support(execution, &revision) {
            Err(Failure::ReadExecution { stop, inventory }) => {
                assert_eq!(inventory.revision(), &revision);

                assert!(inventory.retained_program_bytes() > 0);
                assert_eq!(inventory.retained_interpretations(), 0);
                assert_eq!(inventory.mandatory_custody(), 0);
                assert!(runtime.presented_program().is_some(), "stopped retirement resets the roster to active");
                Err(stop)
            }
            Err(Failure::Policy(Policy::CurrentBranches(inventory))) => {
                assert_eq!(inventory.current_branches(), 1);
                Ok(())
            }
            other @ (Err(Failure::Policy(..)) | Ok(..)) => panic!("unexpected retirement answer: {other:?}"),
        },
        Err(Control::Cancelled),
        Ok(Kind::ExecutionResource { denial: crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial::WorkExhausted, partition_identity: None, policy_ancestor: None }),
    );
}

#[test]
fn branch_creation_keeps_the_source_program_resolver_read_stop() {
    use worth_query_admission::facade::authenticated_principal::{
        WorthQueryCancellationSource, WorthQueryRequestScope,
    };
    let runtime = installed_preview_runtime();
    let before = runtime.current_world();
    let source = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(None, source.token());
    let _named = runtime
        .product_runtime()
        .observe_named_read_consults_for_test();
    let observed_runtime = runtime.product_runtime().clone();
    let reached = std::rc::Rc::new(std::cell::Cell::new(false));
    let fired = std::rc::Rc::clone(&reached);
    let result = runtime
        .with_application_advancement(&request, |phase| {
            let _guard = crate::test_read_stops::observe_consults_for_test(move |_| {
                if observed_runtime.source_program_resolution_is_active_for_test()
                    && !fired.replace(true)
                {
                    source.cancel();
                }
            });
            runtime
                .branches()
                .fork(before)
                .components(|components| components.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
        })
        .unwrap();
    assert!(
        reached.get(),
        "the source program resolver consult was reached"
    );
    assert!(matches!(
        result,
        Err(
            crate::basis::WorthQueryProductBranchCreateError::SourceAdmission(Branch::Stopped(
                Err(Control::Cancelled)
            ))
        )
    ));
    assert_eq!(runtime.current_world(), before);
    let child = runtime
        .with_host_advancement(|phase| {
            runtime
                .branches()
                .fork(before)
                .components(|components| components.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
        })
        .unwrap()
        .expect("live retry forks the same rostered source");
    runtime
        .with_host_advancement(|phase| {
            let selected = runtime.on_branch(child).select(&phase).unwrap();
            assert_eq!(
                selected.inspect_selected_program().unwrap().revision(),
                runtime.owned_revision()
            );
        })
        .unwrap();
}

#[test]
fn failed_preview_discard_retains_original_owner_after_every_readmission_stop() {
    let runtime = installed_preview_runtime();
    let observation = runtime
        .with_host_advancement(|phase| {
            runtime
                .on_branch(runtime.current_world())
                .select(&phase)
                .unwrap()
                .retain_application_read()
        })
        .unwrap();
    let identity = std::cell::Cell::new(0);
    crate::test_read_stops::every_advancement_consult_with_door(
        |counted| {
            let ordinal = identity.get(); identity.set(ordinal + 1);
            let mut preview = runtime.with_host_advancement(|phase| runtime.begin_preview(
                &phase, &observation, WorthQueryApplicationPreviewRequest::new(format!("discard-refusal-{ordinal}")),
            )).unwrap().unwrap();
            let observer = preview.handle.liveness_observer();
            let original = preview.handle.execution_record().digest().to_owned();
            let fault = preview.reject_discard_for_test();
            let preview = std::cell::RefCell::new(Some(preview));
            let retained = std::cell::RefCell::new(None);
            let answer = counted(&|request, ceiling| runtime.with_application_advancement(request, |phase| {
                phase.execution_request_for(&runtime.product_runtime).unwrap().run(worth_execution::ExecutionWorkCeiling::new(Some(ceiling)), |_| {
                    let (preview, cause) = match preview.borrow_mut().take().unwrap().readmit(&phase, &runtime) {
                        Ok(readmitted) => (readmitted.session, None),
                        Err((preview, denial)) => (preview, Some(denial)),
                    };
                    let (preview, denial) = preview.discard().unwrap_err();
                    assert_eq!(denial.kind(), worth_runtime_bridge::facade::BridgeSpeculationErrorKind::PreviewResidueClassificationMismatch);
                    assert_eq!(preview.handle.execution_record().digest(), original);
                    assert!(observer.admit_active_session().is_some());
                    *retained.borrow_mut() = Some(preview);
                    cause.map_or(Ok(()), Err)
                }).unwrap().0
            }).unwrap());
            drop(fault);
            let preview = retained.into_inner().unwrap();
            let readmitted = runtime.with_host_advancement(|phase| preview.readmit(&phase, &runtime)).unwrap().unwrap();
            assert_eq!(readmitted.session.handle.execution_record().digest(), original);
            readmitted.discard().unwrap();
            assert!(observer.admit_active_session().is_none());
            answer
        },
        WorthQueryApplicationPreviewReadmissionDenial::SourceSelection(Branch::Stopped(Err(Control::Cancelled))),
        WorthQueryApplicationPreviewReadmissionDenial::SourceSelection(Branch::Stopped(Ok(Kind::ExecutionResource {
            denial: crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial::WorkExhausted,
            partition_identity: None, policy_ancestor: None,
        }))),
    );
}
