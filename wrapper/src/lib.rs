#[cfg(feature = "macro")]
pub use discord_typed_interactions_proc_macro::typify;

#[cfg(not(feature = "macro"))]
pub mod export {
    use std::path::Path;
    use std::process::{Command, Stdio};
    use std::io::Write;
    use discord_typed_interactions_lib::typify_driver;

    fn fmt(input: &str) -> Option<String> {
        let mut proc = Command::new("rustfmt")
            .arg("--emit=stdout")
            .arg("--edition=2018")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .ok()?;
        let stdin = proc.stdin.as_mut()?;
        stdin.write_all(input.as_bytes()).ok()?;
        let output = proc.wait_with_output().ok()?;

        if output.status.success() {
            String::from_utf8(output.stdout).ok()
        } else {
            None
        }
    }

    // TODO: make a config struct, bikeshed name, etc.
    pub fn todo(src: impl AsRef<Path>, dst: impl AsRef<Path>) {
        let schema_contents = std::fs::read_to_string(src).unwrap();
        let rust_source = typify_driver(&schema_contents).to_string();
        let formatted_source = fmt(&rust_source).unwrap_or(rust_source);
        std::fs::write(dst, formatted_source).unwrap();
    }
}
#[cfg(not(feature = "macro"))]
pub use export::*;
