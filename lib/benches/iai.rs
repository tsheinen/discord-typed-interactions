use iai::black_box;
use discord_typed_interactions_lib::structify;


fn no_subcommands() {
    structify(black_box(include_str!("../../test-harness/schema/no_subcommands.json")));
}

fn ctf() {
    structify(black_box(include_str!("../../test-harness/schema/ctf.json")));
}

iai::main!(no_subcommands, ctf);