use discord_typed_interactions::typify;
use serde_json::json;

typify!("./schema/ctf.json");

fn main() {
    let play = json!({
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
    serde_json::from_value::<ctf::Ctf>(play).unwrap();

    let players_add = json!({
       "id":"868983602015252520",
       "name":"ctf",
       "options":[
          {
             "name":"players",
             "options":[
                {
                   "name":"add",
                   "options":[
                      {
                         "name":"name",
                         "value":"174667467509989376"
                      }
                   ]
                }
             ]
          }
       ],
       "resolved":{
          "members":{
             "174667467509989376":{
                "joined_at":"2018-01-10T22:44:05.797000+00:00",
                "roles":[
                   "868920975901736991"
                ]
             }
          },
          "users":{
             "174667467509989376":{
                "avatar":"a_662952cca2d45e446f0ccd6fe58f7453",
                "bot":false,
                "discriminator":"0004",
                "id":"174667467509989376",
                "username":"sky",
                "public_flags":0
             }
          }
       }
    });
    let parsed = serde_json::from_value::<ctf::Ctf>(players_add.clone()).unwrap();
    match parsed.options {
        ctf::Options::Players(_) => {}
        _ => panic!("deserialized into an options variant that is not Players"),
    }
    parsed.resolved.unwrap();
}
