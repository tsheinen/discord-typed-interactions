#[cfg(feature = "macro")]
pub use discord_typed_interactions_proc_macro::typify;

// #[cfg(not(feature = "macro"))]
pub mod export {
    use discord_typed_interactions_lib::typify_driver;
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use std::process::{Command, Stdio};

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

    pub struct Configuration {
        src: PathBuf,
        dst: PathBuf,
        resolved_struct: Option<String>,
    }

    impl Configuration {
        pub fn new(src: impl Into<PathBuf>) -> Self {
            Configuration {
                src: src.into(),
                dst: PathBuf::from(std::env::var("OUT_DIR").unwrap() + "/interactions.rs"),
                resolved_struct: None,
            }
        }

        pub fn dest(&mut self, dst: impl Into<PathBuf>) -> &mut Self {
            self.dst = dst.into();
            self
        }

        pub fn resolved_struct(&mut self, resolved: impl Into<String>) -> &mut Self {
            self.resolved_struct = Some(resolved.into());
            self
        }

        pub fn generate(&self) {
            let schema_contents = std::fs::read_to_string(&self.src).unwrap();
            let rust_source =
                typify_driver(&schema_contents, self.resolved_struct.as_deref())
                    .to_string();
            let formatted_source = fmt(&rust_source).unwrap_or(rust_source);
            std::fs::write(&self.dst, formatted_source).unwrap();
        }
    }
}
#[cfg(not(feature = "macro"))]
pub use export::*;
