use discord_typed_interactions::generate_structs;
use serde_json::json;

generate_structs!("./test-harness/schema.json");

fn main() {
    let data = json!({
    "id":"868983602015252520",
    "name":"ctf",
    "options":[
       {
          "name":"play",
          "options":[
             {
                "name":"name",
                "value":"howdy"
             }
          ]
       }
    ]
    });
    let ctf: ctf::cmd::Ctf = serde_json::from_value(data).unwrap();
    println!("{:#?}", ctf);
}
