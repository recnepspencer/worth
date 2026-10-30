use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

use serde_json::Value;

use super::SetupConstructionInputs;

static PROCESS_OUTPUT: Mutex<()> = Mutex::new(());

#[derive(Debug, PartialEq, Eq)]
pub enum SetupObservationDisposition {
    Disabled,
    Recorded,
}

/// An owned destination for optional setup observations in a test binary.
pub struct SetupObservation {
    directory: Option<PathBuf>,
}

impl SetupObservation {
    pub fn from_environment() -> Self {
        Self {
            directory: std::env::var_os("WORTH_TEST_COST_SETUP_DIRECTORY").map(PathBuf::from),
        }
    }

    pub fn disabled() -> Self {
        Self { directory: None }
    }

    pub fn record(
        &self,
        builder: &str,
        construction: &dyn Fn(&mut SetupConstructionInputs),
    ) -> std::io::Result<SetupObservationDisposition> {
        record_at(self.directory.clone(), builder, construction)
    }
}

/// Record optional telemetry without constructing the recipe when disabled.
/// Builders may ignore an I/O failure: observation never changes their verdict.
pub fn record(
    builder: &str,
    construction: &dyn Fn(&mut SetupConstructionInputs),
) -> std::io::Result<SetupObservationDisposition> {
    SetupObservation::from_environment().record(builder, construction)
}

fn record_at(
    directory: Option<PathBuf>,
    builder: &str,
    construction: &dyn Fn(&mut SetupConstructionInputs),
) -> std::io::Result<SetupObservationDisposition> {
    let Some(directory) = directory else {
        return Ok(SetupObservationDisposition::Disabled);
    };
    let binary = std::env::var("NEXTEST_BINARY_ID")
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidInput, error))?;
    if builder.trim().is_empty() || binary.trim().is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "setup observations require builder and binary identities",
        ));
    }
    let mut inputs = SetupConstructionInputs::default();
    construction(&mut inputs);
    let mut event = serde_json::Map::new();
    event.insert("schema".to_owned(), Value::from(1));
    event.insert("type".to_owned(), Value::from("test-cost-setup"));
    event.insert("event".to_owned(), Value::from("construction-started"));
    event.insert("builder".to_owned(), Value::from(builder));
    event.insert("binary".to_owned(), Value::from(binary));
    event.insert(
        "fingerprint".to_owned(),
        Value::from(inputs.fingerprint(builder)),
    );
    let mut line = serde_json::to_vec(&event).map_err(std::io::Error::other)?;
    line.push(b'\n');
    std::fs::create_dir_all(&directory)?;
    let mut filename = std::process::id().to_string();
    filename.push_str(".jsonl");
    let _output = PROCESS_OUTPUT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(directory.join(filename))?;
    let complete_prefix = file.metadata()?.len();
    if let Err(error) = file.write_all(&line) {
        let _ = file.set_len(complete_prefix);
        return Err(error);
    }
    Ok(SetupObservationDisposition::Recorded)
}
