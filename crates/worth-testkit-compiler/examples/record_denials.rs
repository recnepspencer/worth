//! The compiler owner records only explicitly selected, reviewed source paths.
use worth_testkit_compiler::{record_denial_snapshot, review_denial_snapshot};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut arguments = std::env::args().skip(1).peekable();
    let write = arguments.peek().is_some_and(|value| value == "--write");
    if write {
        arguments.next();
    }
    let mut sources = Vec::new();
    while let Some(argument) = arguments.next() {
        if argument == "--sources-from" {
            let path = arguments
                .next()
                .ok_or("--sources-from requires a JSON path list")?;
            sources.extend(serde_json::from_str::<Vec<String>>(
                &std::fs::read_to_string(path)?,
            )?);
        } else {
            sources.push(argument);
        }
    }
    for source in sources {
        let review = review_denial_snapshot(&source)?;
        if review.expected != review.observed {
            println!(
                "{}",
                serde_json::json!({"source": review.source, "expected": review.expected, "observed": review.observed})
            );
            if write {
                record_denial_snapshot(&source)?;
            }
        }
    }
    Ok(())
}
