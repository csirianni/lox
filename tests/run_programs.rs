use std::fs;
use std::process::Command;

fn expected_output(contents: &str) -> String {
    contents
        .lines()
        .filter_map(|line| line.strip_prefix("//! "))
        .collect::<Vec<&str>>()
        .join("\n")
}

#[test]
fn run_programs() {
    let programs = fs::read_dir("tests/programs").unwrap();
    for program in programs {
        let path = program.unwrap().path();
        let expected = expected_output(&fs::read_to_string(&path).unwrap());
        let output = Command::new(env!("CARGO_BIN_EXE_lox"))
            .arg(&path)
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(
            stdout.trim(),
            expected.trim(),
            "Output mismatch for {}",
            path.display()
        );
    }
}
