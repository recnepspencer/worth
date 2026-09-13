mod arguments;
mod execution;
mod phase_eight_process_suite;
mod plan;
mod product;

use std::path::{Path, PathBuf};

use arguments::Arguments;

pub fn run_from_environment() -> Result<(), String> {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    if arguments::help_requested(&arguments) {
        println!("{}", arguments::usage());
        return Ok(());
    }
    let arguments = Arguments::parse(arguments)?;
    run(arguments, &workspace_root())
}

fn run(arguments: Arguments, workspace_root: &Path) -> Result<(), String> {
    run_planned_product(arguments, workspace_root)
}

#[doc(hidden)]
pub fn run_process_scenario_from_environment() -> Result<(), String> {
    phase_eight_process_suite::run(&workspace_root(), None)
}

fn run_planned_product(arguments: Arguments, workspace_root: &Path) -> Result<(), String> {
    let plan = plan::TestPlan::build(&arguments.product, workspace_root)?;

    if arguments.list {
        for unit in plan.units() {
            println!("{}\t{}", unit.identity(), unit.display_command());
        }
        return Ok(());
    }

    println!("{}: {} unit(s)", plan.product_name(), plan.units().len());
    let started = std::time::Instant::now();
    execution::execute(plan.units(), arguments.target_root.as_deref())?;
    println!(
        "{}: passed in {:.2}s",
        plan.product_name(),
        started.elapsed().as_secs_f64()
    );
    Ok(())
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("runner must live under tools/<crate>")
        .to_path_buf()
}
