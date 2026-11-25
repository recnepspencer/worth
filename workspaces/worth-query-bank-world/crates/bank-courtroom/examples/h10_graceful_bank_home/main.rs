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
            protocol::Stage::OpenCloseProbe => open_close_probe(&args),
            protocol::Stage::Preclose => preclose(&args),
            protocol::Stage::PrekillBeforeCheckpoint => prekill(&args, false),
            protocol::Stage::PrekillAfterCheckpoint => prekill(&args, true),
            protocol::Stage::Reopen => reopen(&args),
        }
        println!(
            "H10_GRACEFUL_STAGE {} PID {}",
            args.stage.name(),
            std::process::id()
        );
    });
}

fn open_close_probe(args: &protocol::Arguments) {
    let inputs = installation::Inputs::from_recipe(&args.recipe);
    for cycle in ["fresh", "reopened"] {
        let home = inputs.open(&args.directory);
        eprintln!("BANK_HOLDER_PROBE {cycle} opened; no requests have executed");
        // The real consuming close checks physical Arc exclusivity. A refusal
        // reports its exact remaining holders instead of bypassing retirement.
        installation::close(home);
        eprintln!("BANK_HOLDER_PROBE {cycle} closed; physical exclusivity established");
    }
}

fn run_workload(
    args: &protocol::Arguments,
    inputs: &installation::Inputs,
    home: &installation::Home,
    killed_without_close: bool,
) -> Option<protocol::Handoff> {
    let principal = inputs.authenticate(home, 1);
    if args.step_completed(1, "authenticate") {
        return None;
    }
    let transport = external::connect(home, args.rail, args.control);
    if args.step_completed(2, "install-external-transport") {
        return None;
    }
    let (original_workflow, required, instance) =
        workflow::start_wait(home, &principal, inputs.payment);
    if args.step_completed(3, "workflow-start-wait") {
        return None;
    }
    let assessment = assessment::retain(home, &principal, inputs.payment, instance, &required);
    if args.step_completed(4, "retain-assessment") {
        return None;
    }
    let (rejection, components) = deferred::retain(
        home,
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
    if args.step_completed(5, "deferred-composite") {
        return None;
    }
    let pending = external::dispatch_ambiguously(home, &principal, &transport);
    if args.step_completed(6, "ambiguous-external-dispatch") {
        return None;
    }
    let reversals = correction::reverse_on_both(home, &principal);
    if args.step_completed(7, "reverse-both") {
        return None;
    }
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
    if args.step_completed(8, "descriptor-codecs") {
        return None;
    }
    Some(protocol::Handoff {
        preclose_pid: std::process::id(),
        killed_without_close,
        prekill_checkpoint_coverage: Vec::new(),
        workflow,
        required: required_bytes,
        assessment: assessment_bytes,
        rejection_home: rejection.identity().home(),
        rejection_position: rejection.identity().acquire_position(),
        pending,
        reversals,
    })
}

fn preclose(args: &protocol::Arguments) {
    let inputs = installation::Inputs::from_recipe(&args.recipe);
    let home = inputs.open(&args.directory);
    let handoff = run_workload(args, &inputs, &home, false);
    // All process-local owners are gone. DATA is written only after actual
    // graceful close and reclaim, and the parent also requires process success.
    installation::close(home);
    if let Some(handoff) = handoff {
        handoff.write(&args.handoff);
    }
}

fn prekill(args: &protocol::Arguments, take_checkpoint: bool) {
    use std::io::Write as _;
    let inputs = installation::Inputs::from_recipe(&args.recipe);
    let home = inputs.open(&args.directory);
    let mut handoff =
        run_workload(args, &inputs, &home, true).expect("prekill runs complete pre-kill workload");
    if take_checkpoint {
        let generation = installation::checkpoint(&home);
        assert!(
            generation >= 1,
            "post-checkpoint kill must follow an actual selected home checkpoint"
        );
    }
    let coverage = installation::checkpoint_covered_journals(&home);
    if take_checkpoint {
        assert!(
            coverage.iter().any(|(_, covered)| *covered > 0),
            "post-checkpoint kill must report positive journal coverage: {coverage:?}"
        );
    } else {
        assert!(
            coverage.iter().all(|(_, covered)| *covered == 0),
            "pre-checkpoint kill must occur before the first home checkpoint: {coverage:?}"
        );
    }
    handoff.prekill_checkpoint_coverage = coverage;
    handoff.write(&args.handoff);
    println!(
        "H10_PREKILL_READY {} PID {}",
        args.stage.name(),
        std::process::id()
    );
    let _ = std::io::stdout().flush();
    // Hold the live Home unclosed until the parent sends a real OS kill.
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        assert!(home.is_directory());
    }
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
    if handoff.killed_without_close {
        assert_eq!(
            installation::checkpoint_covered_journals(&home),
            handoff.prekill_checkpoint_coverage,
            "reopen must observe the exact pre-kill checkpoint posture"
        );
    }
    (|| {
        let principal = inputs.authenticate(&home, 1);
        if args.step_completed(1, "reopen-authenticate") {
            return;
        }
        let transport = external::connect(&home, args.rail, args.control);
        if args.step_completed(2, "reopen-install-transport") {
            return;
        }
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
        if args.step_completed(3, "reopen-descriptor-readmission") {
            return;
        }
        let instance = workflow::resume_wait(
            &home,
            &principal,
            inputs.payment,
            &original_workflow,
            &required,
        );
        if args.step_completed(4, "reopen-workflow-resume") {
            return;
        }
        deferred::resume(
            &home,
            &principal,
            worth_foundational::facade::ProgramSupportObligationIdentity::from_recorded(
                handoff.rejection_home,
                handoff.rejection_position,
            ),
            inputs.deferred_payment,
        );
        if args.step_completed(5, "reopen-deferred-resume") {
            return;
        }
        external::observe_and_replay(&home, &principal, &transport, handoff.pending);
        if args.step_completed(6, "reopen-external-replay") {
            return;
        }
        correction::reapply_recovered_child(&home, &principal, handoff.reversals);
        if args.step_completed(7, "reopen-correction") {
            return;
        }
        assessment::recover_and_advance(
            &home,
            &principal,
            inputs.payment,
            inputs.deferred_payment,
            instance,
            &required,
            &assessment,
        );
        if args.step_completed(8, "reopen-assessment") {}
    })();
    installation::close(home);
}
