use discord_typed_interactions::Configuration;

fn main() {
    Configuration::new("schema/ctf.json")
        .dest(std::env::var("OUT_DIR").unwrap() + "/ctf_gen.rs")
        .watch_schema()
        .generate();
}
