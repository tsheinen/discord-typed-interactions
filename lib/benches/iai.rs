use iai::black_box;
use discord_typed_interactions_lib::typify_driver;

fn no_subcommands() {
    typify_driver(black_box(include_str!("../../test-harness/schema/no_subcommands.json")));
}

fn ctf() {
    typify_driver(black_box(include_str!("../../test-harness/schema/ctf.json")));
}

iai::main!(no_subcommands, ctf);
