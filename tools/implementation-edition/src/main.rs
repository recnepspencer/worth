use std::path::Path;

fn main() {
    if let Err(error) = run() {
        eprintln!("implementation-edition: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    if arguments.first().is_some_and(|value| value == "build") {
        if arguments.len() < 3 {
            return Err(
                "usage: implementation-edition build MANIFEST TARGET [cargo-build-args]".into(),
            );
        }
        implementation_edition::build(
            Path::new(&arguments[1]),
            Path::new(&arguments[2]),
            &arguments[3..],
        )
        .map(|_| ())
    } else {
        let (compiler, arguments) = arguments.split_first().ok_or("missing rustc invocation")?;
        implementation_edition::observe(compiler, arguments)
    }
}
