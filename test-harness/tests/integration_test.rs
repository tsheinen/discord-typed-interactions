fn run_test_harness(bin: impl AsRef<str>) -> Result<Option<i32>, std::io::Error> {
    Ok(std::process::Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg(bin.as_ref())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?
        .wait()?
        .code())
}

#[test]
fn ctf() -> Result<(), std::io::Error> {
    assert_eq!(run_test_harness("ctf")?, Some(0));
    Ok(())
}

#[test]
fn no_subcommands() -> Result<(), std::io::Error> {
    assert_eq!(run_test_harness("no_subcommands")?, Some(0));
    Ok(())
}

#[test]
fn multiple_schema() -> Result<(), std::io::Error> {
    assert_eq!(run_test_harness("multiple_schema")?, Some(0));
    Ok(())
}

#[test]
fn builder() -> Result<(), std::io::Error> {
    assert_eq!(run_test_harness("builder")?, Some(0));
    Ok(())
}
