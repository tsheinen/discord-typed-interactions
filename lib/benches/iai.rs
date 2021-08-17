use iai::black_box;
use discord_typed_interactions_lib::typify_driver;

fn no_subcommands() {
    typify_driver(black_box(Some(include_str!("../../test-harness/schema/no_subcommands.json"))), None);
}

fn ctf() {
    typify_driver(black_box(Some(include_str!("../../test-harness/schema/ctf.json"))), None);
}

iai::main!(no_subcommands, ctf);
