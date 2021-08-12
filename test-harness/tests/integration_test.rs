#[test]
fn ctf() -> Result<(), std::io::Error>{
    assert_eq!(std::process::Command::new("cargo").arg("run").arg("--bin").arg("ctf").spawn()?.wait()?.code(), Some(0));
    Ok(())
}

#[test]
fn no_subcommands() -> Result<(), std::io::Error>{
    assert_eq!(std::process::Command::new("cargo").arg("run").arg("--bin").arg("no_subcommands").spawn()?.wait()?.code(), Some(0));
    Ok(())
}

#[test]
fn multiple_schema() -> Result<(), std::io::Error>{
    assert_eq!(std::process::Command::new("cargo").arg("run").arg("--bin").arg("multiple_schema").spawn()?.wait()?.code(), Some(0));
    Ok(())
}

#[test]
fn builder() -> Result<(), std::io::Error>{
    assert_eq!(std::process::Command::new("cargo").arg("run").arg("--bin").arg("builder").spawn()?.wait()?.code(), Some(0));
    Ok(())
}