use discord_typed_interactions::typify;
use serde_json::json;

typify!("./test-harness/schema/no_subcommands.json");

fn main() {
    let test = json!({
    "id":"868983602015252520",
    "name":"test",
    "options":[
        {
            "name":"a",
            "value":"a"
        },
        {
            "name":"b",
            "value":"b"
        },
        {
            "name":"c",
            "value":"c"
        }
    ]
    });
    serde_json::from_value::<test::Test>(test).unwrap();
}
