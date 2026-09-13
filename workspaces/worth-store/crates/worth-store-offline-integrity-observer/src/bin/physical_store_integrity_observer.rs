#![forbid(unsafe_code)]

#[path = "physical_store_integrity_observer/arguments.rs"]
mod arguments;
#[path = "physical_store_integrity_observer/comparison.rs"]
mod comparison;
#[path = "physical_store_integrity_observer/observation.rs"]
mod observation;
#[path = "physical_store_integrity_observer/report_output.rs"]
mod report_output;

fn main() {
    let arguments = match arguments::parse(std::env::args_os().skip(1)) {
        Ok(arguments) => arguments,
        Err(arguments::ArgumentOutcome::Help(help)) => {
            print!("{help}");
            return;
        }
        Err(arguments::ArgumentOutcome::Denied(message)) => {
            eprintln!("physical_store_integrity_observer: {message}");
            std::process::exit(2);
        }
    };
    let result = match arguments {
        arguments::OperationArguments::Observe(arguments) => observation::observe(arguments),
        arguments::OperationArguments::Compare(arguments) => comparison::compare(arguments),
    };
    if let Err(message) = result {
        eprintln!("physical_store_integrity_observer: {message}");
        std::process::exit(1);
    }
}
