//! One Directory journey in two genuine main-thread processes.
mod assessment;
mod correction;
mod deferred;
mod external;
mod installation;
mod protocol;
mod workflow;

use bank_domain::schema::BankSchema;

fn main() {
    assert_eq!(std::thread::current().name(), Some("main"));
    worth_execution::run_with_execution_stack(|| {
        let args = protocol::Arguments::read();
        match args.stage {
            protocol::Stage::Preclose => preclose(&args),
            protocol::Stage::Reopen => reopen(&args),
        }
        println!(
            "H10_GRACEFUL_STAGE {} PID {}",
            args.stage.name(),
            std::process::id()
        );
    });
}

fn preclose(args: &protocol::Arguments) {
    let inputs = installation::Inputs::from_recipe(&args.recipe);
    let home = inputs.open(&args.directory);
    let handoff = {
        let principal = inputs.authenticate(&home, 1);
        let transport = external::connect(&home, args.rail, args.control);
        let (original_workflow, required, instance) =
            workflow::start_wait(&home, &principal, inputs.payment);
        let assessment = assessment::retain(&home, &principal, inputs.payment, instance, &required);
        let (rejection, components) = deferred::retain(
            &home,
            &principal,
            inputs.deferred_payment,
            "h10-deferred-before-close",
        );
        use worth_query_host::facade::runtime::CompositeComponentChangePosture::Published;
        assert_eq!(
            components,
            [Published, Published],
            "one World publication changes Relational and Signal"
        );
        let pending = external::dispatch_ambiguously(&home, &principal, &transport);
        let reversals = correction::reverse_on_both(&home, &principal);
        let workflow = original_workflow.to_recorded_bytes().unwrap();
        use worth_query_host::facade::application_entry::{
            RequiredWorkflowAssessment, WorthQueryWorkflowInstanceDiscoveryIntent,
        };
        assert_eq!(
            WorthQueryWorkflowInstanceDiscoveryIntent::from_recorded_bytes(
                &workflow,
                *home.runtime().installed_program().revision()
            )
            .unwrap(),
            original_workflow
        );
        assert!(
            WorthQueryWorkflowInstanceDiscoveryIntent::<BankSchema>::from_recorded_bytes(
                &workflow[..workflow.len() - 1],
                *home.runtime().installed_program().revision()
            )
            .is_err()
        );
        let required_bytes = required.to_recorded_bytes().unwrap();
        assert_eq!(
            RequiredWorkflowAssessment::from_recorded_bytes(&required_bytes).unwrap(),
            required
        );
        let assessment_bytes = assessment.to_recorded_bytes().unwrap();
        let decoded = assessment::Original::from_recorded_bytes(&assessment_bytes).unwrap();
        assert_eq!(decoded.to_recorded_bytes().unwrap(), assessment_bytes);
        assert!(assessment::Original::from_recorded_bytes(
            &assessment_bytes[..assessment_bytes.len() - 1]
        )
        .is_err());
        protocol::Handoff {
            preclose_pid: std::process::id(),
            workflow,
            required: required_bytes,
            assessment: assessment_bytes,
            rejection_home: rejection.identity().home(),
            rejection_position: rejection.identity().acquire_position(),
            pending,
            reversals,
        }
    };
    // All process-local owners are gone. DATA is written only after actual
    // graceful close and reclaim, and the parent also requires process success.
    installation::close(home);
    handoff.write(&args.handoff);
}

fn reopen(args: &protocol::Arguments) {
    let handoff = protocol::Handoff::read(&args.handoff);
    assert_ne!(
        handoff.preclose_pid,
        std::process::id(),
        "reopen must be a fresh process"
    );
    let inputs = installation::Inputs::from_recipe(&args.recipe);
    let home = inputs.open(&args.directory);
    {
        let principal = inputs.authenticate(&home, 1);
        let transport = external::connect(&home, args.rail, args.control);
        use worth_query_host::facade::application_entry::{
            RequiredWorkflowAssessment, WorthQueryWorkflowInstanceDiscoveryIntent,
        };
        let original_workflow = WorthQueryWorkflowInstanceDiscoveryIntent::from_recorded_bytes(
            &handoff.workflow,
            *home.runtime().installed_program().revision(),
        )
        .expect("the current installed validated revision admits the locator DATA");
        let required = RequiredWorkflowAssessment::from_recorded_bytes(&handoff.required).unwrap();
        let assessment = assessment::Original::from_recorded_bytes(&handoff.assessment).unwrap();
        let instance = workflow::resume_wait(
            &home,
            &principal,
            inputs.payment,
            &original_workflow,
            &required,
        );
        deferred::resume(
            &home,
            &principal,
            worth_foundational::facade::ProgramSupportObligationIdentity::from_recorded(
                handoff.rejection_home,
                handoff.rejection_position,
            ),
            inputs.deferred_payment,
        );
        external::observe_and_replay(&home, &principal, &transport, handoff.pending);
        correction::reapply_recovered_child(&home, &principal, handoff.reversals);
        assessment::recover_and_advance(
            &home,
            &principal,
            inputs.payment,
            inputs.deferred_payment,
            instance,
            &required,
            &assessment,
        );
    }
    installation::close(home);
}
