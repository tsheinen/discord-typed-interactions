#[cfg(feature = "macro")]
pub use discord_typed_interactions_proc_macro::typify;

#[cfg(feature = "builder")]
pub mod export {
    use discord_typed_interactions_lib::typify_driver;
    use std::io::Write;
    use std::path::PathBuf;
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
        src: Vec<PathBuf>,
        dst: PathBuf,
        resolved_struct: Option<String>,
    }

    impl Configuration {
        pub fn new(src: impl Into<PathBuf>) -> Self {
            Configuration {
                src: vec![src.into()],
                dst: PathBuf::from(std::env::var("OUT_DIR").unwrap() + "/interactions.rs"),
                resolved_struct: None,
            }
        }

        pub fn src(&mut self, src: impl Into<PathBuf>) -> &mut Self {
            self.src.push(src.into());
            self
        }

        pub fn dest(&mut self, dst: impl Into<PathBuf>) -> &mut Self {
            self.dst = dst.into();
            self
        }

        pub fn resolved_struct(&mut self, resolved: impl Into<String>) -> &mut Self {
            self.resolved_struct = Some(resolved.into());
            self
        }

        pub fn watch_schema(&mut self) -> &mut Self {
            for i in self.src.as_slice() {
                println!("cargo:rerun-if-changed={}", i.display());

            }
            self
        }

        pub fn generate(&self) {
            let schema_contents = self.src.iter().map(|x| std::fs::read_to_string(x).unwrap());
            let rust_source =
                typify_driver(schema_contents, self.resolved_struct.as_deref())
                    .to_string();
            let formatted_source = fmt(&rust_source).unwrap_or(rust_source);
            std::fs::write(&self.dst, formatted_source).unwrap();
        }
    }
}
#[cfg(feature = "builder")]
pub use export::*;
