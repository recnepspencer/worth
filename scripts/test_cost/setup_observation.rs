//! Optional construction telemetry compiled privately into closed court testkits.
//! A recipe fingerprint never authorizes sharing its resulting live fixture.

use std::io::{self, Write};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

static OBSERVATION_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Default)]
pub(crate) struct ConstructionInputs {
    fields: Vec<(String, u8, Vec<u8>)>,
}

impl ConstructionInputs {
    pub(crate) fn text(&mut self, name: &str, value: &str) {
        self.fields
            .push((name.to_owned(), 0, value.as_bytes().to_vec()));
    }

    pub(crate) fn unsigned(&mut self, name: &str, value: u64) {
        self.fields
            .push((name.to_owned(), 1, value.to_le_bytes().to_vec()));
    }

    pub(crate) fn signed(&mut self, name: &str, value: i64) {
        self.fields
            .push((name.to_owned(), 2, value.to_le_bytes().to_vec()));
    }

    pub(crate) fn boolean(&mut self, name: &str, value: bool) {
        self.fields
            .push((name.to_owned(), 3, vec![u8::from(value)]));
    }

    fn fingerprint(mut self) -> io::Result<String> {
        self.fields.sort_by(|left, right| left.0.cmp(&right.0));
        if self.fields.iter().any(|field| field.0.trim().is_empty())
            || self.fields.windows(2).any(|pair| pair[0].0 == pair[1].0)
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "recipe field names must be nonempty and unique",
            ));
        }
        let mut encoded = vec![1];
        for (name, kind, value) in self.fields {
            encoded.push(kind);
            encoded.extend_from_slice(&(name.len() as u64).to_le_bytes());
            encoded.extend_from_slice(name.as_bytes());
            encoded.extend_from_slice(&(value.len() as u64).to_le_bytes());
            encoded.extend_from_slice(&value);
        }
        let mut fingerprint = String::with_capacity(encoded.len() * 2);
        const HEX: &[u8; 16] = b"0123456789abcdef";
        for byte in encoded {
            fingerprint.push(char::from(HEX[usize::from(byte >> 4)]));
            fingerprint.push(char::from(HEX[usize::from(byte & 15)]));
        }
        Ok(fingerprint)
    }
}

pub(crate) fn record(
    builder: &str,
    construction: &dyn Fn(&mut ConstructionInputs),
) -> io::Result<()> {
    let directory = std::env::var_os("WORTH_TEST_COST_SETUP_DIRECTORY");
    let Some(directory) = directory else {
        return Ok(());
    };
    let binary = std::env::var("NEXTEST_BINARY_ID")
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
    record_at(Some(Path::new(&directory)), &binary, builder, construction)
}

fn record_at(
    directory: Option<&Path>,
    binary: &str,
    builder: &str,
    construction: &dyn Fn(&mut ConstructionInputs),
) -> io::Result<()> {
    let Some(directory) = directory else {
        return Ok(());
    };
    if builder.trim().is_empty() || binary.trim().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "builder and binary identities are required",
        ));
    }
    let mut inputs = ConstructionInputs::default();
    construction(&mut inputs);
    let fingerprint = inputs.fingerprint()?;
    let line = format!(
        "{{\"schema\":1,\"type\":\"test-cost-setup\",\"event\":\"construction-started\",\"builder\":{},\"binary\":{},\"input_encoding\":\"typed-length-prefixed-hex-v1\",\"fingerprint\":{}}}\n",
        json_string(builder), json_string(binary), json_string(&fingerprint));
    std::fs::create_dir_all(directory)?;
    let sequence = OBSERVATION_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let owner = module_path!().replace("::", "-");
    let path = directory.join(format!("{}-{owner}-{sequence}.jsonl", std::process::id()));
    let mut output = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)?;
    output.write_all(line.as_bytes())
}

fn json_string(value: &str) -> String {
    let mut result = String::from("\"");
    for character in value.chars() {
        match character {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            character if character < ' ' => {
                result.push_str(&format!("\\u{:04x}", u32::from(character)))
            }
            character => result.push(character),
        }
    }
    result.push('"');
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    fn directory() -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "worth-setup-observation-{}-{}",
            std::process::id(),
            OBSERVATION_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ))
    }

    #[test]
    fn disabled_observation_does_not_construct_inputs() {
        let called = Cell::new(false);
        record_at(None, "", "", &|_| called.set(true)).unwrap();
        assert!(!called.get());
    }

    #[test]
    fn canonical_named_fields_ignore_declaration_order_but_preserve_values_and_types() {
        let mut first = ConstructionInputs::default();
        first.text("recipe", "a");
        first.unsigned("count", 7);
        let mut reordered = ConstructionInputs::default();
        reordered.unsigned("count", 7);
        reordered.text("recipe", "a");
        let mut changed = ConstructionInputs::default();
        changed.text("recipe", "a");
        changed.unsigned("count", 8);
        let mut changed_type = ConstructionInputs::default();
        changed_type.text("recipe", "a");
        changed_type.signed("count", 7);
        let fingerprint = first.fingerprint().unwrap();
        assert_eq!(fingerprint, reordered.fingerprint().unwrap());
        assert_ne!(fingerprint, changed.fingerprint().unwrap());
        assert_ne!(fingerprint, changed_type.fingerprint().unwrap());
    }

    #[test]
    fn length_prefixes_disambiguate_field_boundaries() {
        let mut first = ConstructionInputs::default();
        first.text("a", "bc");
        let mut second = ConstructionInputs::default();
        second.text("ab", "c");
        assert_ne!(first.fingerprint().unwrap(), second.fingerprint().unwrap());
    }

    #[test]
    fn duplicate_names_refuse_without_writing_a_construction() {
        let path = directory();
        let error = record_at(Some(&path), "binary", "builder", &|inputs| {
            inputs.unsigned("seed", 1);
            inputs.unsigned("seed", 2);
        })
        .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        assert!(!path.exists());
    }

    #[test]
    fn output_failure_is_an_io_result_and_missing_identity_does_not_write() {
        let path = directory();
        std::fs::write(&path, "occupied").unwrap();
        assert!(record_at(Some(&path), "binary", "builder", &|_| {}).is_err());
        assert_eq!(
            record_at(Some(&path), "", "builder", &|_| {})
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidInput
        );
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "occupied");
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn complete_observations_keep_binary_builder_and_every_construction() {
        let path = directory();
        for _ in 0..2 {
            record_at(Some(&path), "binary", "builder", &|inputs| {
                inputs.unsigned("recipe_revision", 1);
                inputs.boolean("enabled", true);
            })
            .unwrap();
        }
        let files = std::fs::read_dir(&path)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect::<Vec<_>>();
        assert_eq!(files.len(), 2);
        let lines = files
            .iter()
            .map(|file| std::fs::read_to_string(file).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(lines[0], lines[1]);
        assert!(lines[0].contains("\"binary\":\"binary\""));
        assert!(lines[0].contains("\"builder\":\"builder\""));
        assert!(lines[0].contains("\"event\":\"construction-started\""));
        assert_eq!(json_string("a\n\"\\"), "\"a\\u000a\\\"\\\\\"");
        std::fs::remove_dir_all(path).unwrap();
    }
}
