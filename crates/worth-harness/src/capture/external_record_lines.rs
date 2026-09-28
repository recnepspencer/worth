use std::fs;
use std::path::{Path, PathBuf};

use serde::{de::DeserializeOwned, Serialize};

pub fn emit_external_record_line(value: impl Serialize) {
    println!(
        "{}",
        serde_json::to_string(&value).expect("external record-line serialization")
    );
}

pub fn read_external_record_lines<T>(path: &Path, record_description: &str) -> Vec<T>
where
    T: DeserializeOwned,
{
    fs::read_to_string(path)
        .unwrap_or_else(|error| {
            panic!(
                "failed to read external {record_description} records {}: {error}",
                path.display()
            )
        })
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            serde_json::from_str::<T>(line).unwrap_or_else(|error| {
                panic!(
                    "failed to deserialize external {record_description} record from {}: {error}",
                    path.display()
                )
            })
        })
        .collect()
}

pub fn engineering_external_record_lines_path(file_stem: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("plans")
        .join("engineering")
        .join(file_stem);
    path.set_extension("jsonl");
    path
}

#[cfg(test)]
mod tests {
    use std::fs;

    use serde::{Deserialize, Serialize};

    use super::read_external_record_lines;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct ExternalLineRecord {
        name: String,
        count: u64,
    }

    #[test]
    fn external_record_lines_materialize_typed_records_at_harness_boundary() {
        let path = std::env::temp_dir().join("worth_harness_external_record_lines_test.jsonl");
        fs::write(
            &path,
            r#"{"name":"first","count":1}
{"name":"second","count":2}
"#,
        )
        .expect("external record-line fixture");

        let records: Vec<ExternalLineRecord> = read_external_record_lines(&path, "test boundary");

        fs::remove_file(&path).ok();
        assert_eq!(
            records,
            vec![
                ExternalLineRecord {
                    name: "first".to_string(),
                    count: 1,
                },
                ExternalLineRecord {
                    name: "second".to_string(),
                    count: 2,
                },
            ]
        );
    }
}
