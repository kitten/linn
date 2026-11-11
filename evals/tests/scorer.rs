use serde_json::json;
use std::{fs, process::Command};

#[test]
fn scores_generated_gold_section_and_silence_without_external_data() {
    let dir = tempfile::tempdir().unwrap();
    let records = [
        json!({"id":"r1", "state":"accepted", "kind":"doc", "body":"# Fixture\nfixture_identifier_xyz details.", "label":"Fixture"}),
        json!({"id":"r2", "state":"accepted", "kind":"note", "body":"cross_fixture_abc", "scope":"other/repo"}),
    ];
    let cases = [
        json!({"id":"gold", "query":"fixture_identifier_xyz", "scope":null, "category":["doc-passage"], "gold":[{"record":"r1", "quote":"fixture_identifier_xyz"}]}),
        json!({"id":"equivalent", "query":"fixture_identifier_xyz", "gold":[{"record":"missing", "equivalent":["r1"]}]}),
        json!({"id":"cross", "query":"cross_fixture_abc", "scope":"my/repo", "category":["cross-scope"], "gold":[], "must_not":["r2"]}),
        json!({"id":"empty", "query":"", "scope":null, "gold":[], "must_not":["r1"]}),
    ];
    for (name, rows) in [
        ("records.jsonl", records.as_slice()),
        ("cases.jsonl", cases.as_slice()),
    ] {
        fs::write(
            dir.path().join(name),
            rows.iter().map(|v| format!("{v}\n")).collect::<String>(),
        )
        .unwrap();
    }
    let output = Command::new(env!("CARGO_BIN_EXE_linn-eval"))
        .args(["--mode", "all", "--corpus"])
        .arg(dir.path())
        .arg("--pages")
        .arg(dir.path().join("actual.jsonl"))
        .arg("--timings")
        .arg(dir.path().join("timings.jsonl"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "any 2/2 first 2/2 xs 1/1 1/1\nempty 2/2 section 1/1\n"
    );
    let timings: Vec<serde_json::Value> = fs::read_to_string(dir.path().join("timings.jsonl"))
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(timings.len(), 1 + cases.len());
    assert_eq!(timings[0]["event"], "store-ready");
    assert_eq!(timings[0]["records"], records.len());
    for (event, case) in timings[1..].iter().zip(&cases) {
        assert_eq!(event["case"], case["id"]);
        assert_eq!(event["wording"], 0);
        assert!(event["query_ms"].as_f64().unwrap() >= 0.);
    }
    let neutral = Command::new(env!("CARGO_BIN_EXE_linn-eval"))
        .args(["--neutral", "--mode", "all", "--corpus"])
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(
        neutral.status.success(),
        "{}",
        String::from_utf8_lossy(&neutral.stderr)
    );
    let table = String::from_utf8(neutral.stdout).unwrap();
    for gate in [
        "G1", "G2", "G3", "G4", "G5", "Q1", "Q2", "Q3", "Q4", "Q5", "Q6", "C1", "C2", "R1", "R2",
    ] {
        assert!(
            table.contains(&format!("| {gate}")),
            "missing gate {gate}: {table}"
        );
    }
    assert!(table.contains("INSUFFICIENT-N"));

    let complete = Command::new(env!("CARGO_BIN_EXE_linn-eval"))
        .args(["--mode", "all", "--corpus"])
        .arg(dir.path())
        .arg("--compare")
        .arg(dir.path().join("actual.jsonl"))
        .output()
        .unwrap();
    assert!(
        complete.status.success(),
        "{}",
        String::from_utf8_lossy(&complete.stderr)
    );

    // A partial export must not silently validate all of the selected cases.
    let comparison = dir.path().join("expected.jsonl");
    fs::write(
        &comparison,
        format!("{}\n", json!({"case":"gold","wording":0,"records":["r1"]})),
    )
    .unwrap();
    let incomplete = Command::new(env!("CARGO_BIN_EXE_linn-eval"))
        .args(["--mode", "all", "--corpus"])
        .arg(dir.path())
        .arg("--compare")
        .arg(&comparison)
        .output()
        .unwrap();
    assert!(!incomplete.status.success());
    assert!(String::from_utf8_lossy(&incomplete.stderr).contains("missing expected page"));

    let duplicate = format!("{}\n", json!({"case":"gold","wording":0,"records":["r1"]}));
    fs::write(&comparison, duplicate.repeat(2)).unwrap();
    let duplicates = Command::new(env!("CARGO_BIN_EXE_linn-eval"))
        .args(["--mode", "all", "--corpus"])
        .arg(dir.path())
        .arg("--compare")
        .arg(&comparison)
        .output()
        .unwrap();
    assert!(!duplicates.status.success());
    assert!(String::from_utf8_lossy(&duplicates.stderr).contains("duplicate page key"));

    let benchmark = Command::new(env!("CARGO_BIN_EXE_linn-eval"))
        .args([
            "--document-benchmark",
            "--mode",
            "all",
            "--case",
            "gold",
            "--corpus",
        ])
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(
        benchmark.status.success(),
        "{}",
        String::from_utf8_lossy(&benchmark.stderr)
    );
    let table = String::from_utf8(benchmark.stdout).unwrap();
    assert!(table.contains("| all | 1 | 1/1 | 1/1 | 1/1 | 1.0000 |"));
}
