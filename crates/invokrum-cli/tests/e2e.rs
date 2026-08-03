use std::process::Command;

#[test]
fn binary_reports_its_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_invokrum"))
        .output()
        .expect("invokrum binary should execute");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert_eq!(stdout, format!("invokrum {}\n", env!("CARGO_PKG_VERSION")));
}
