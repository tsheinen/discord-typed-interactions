# discord-typed-interactions

I was writing a discord bot and all the dynamic command data checking was really painful so I was inspired to not do that ever again. Thus, this. 

A few points to note:
* Input paths are relative to Cargo.toml; include_str! is a compiler built-in and we don't have any easy way to replicate that behavior.
* We do not re-export serde, so you will need to depend on serde and serde_json for the generated code to compile.  

## proc macro

```rust
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
    println!("{:#?}", play)
}
```

```text
Ctf {
    id: "868983602015252520",
    name: "ctf",
    options: Play(
        Options {
            name: "howdy",
        },
    ),
    resolved: None,
}
```

## build.rs

```rust
use discord_typed_interactions::Configuration;

fn main() {
    Configuration::new("schema/ctf.json")
        // .src("schema/other.json") // should you have more commands you can use Configuration::src multiple times
        .dest("src/command.rs")
        .generate();
}
```

## generated code

<details>
<summary>schema</summary>
<pre lang="json">
<code>
{
  "name": "ctf",
  "description": "placeholder",
  "options": [
    {
      "type": 1,
      "name": "add",
      "description": "placeholder",
      "options": [
        {
          "type": 3,
          "name": "name",
          "description": "placeholder",
          "required": true
        }
      ]
    },
    {
      "type": 1,
      "name": "archive",
      "description": "placeholder",
      "options": [
        {
          "type": 7,
          "name": "channel",
          "description": "placeholder"
        }
      ]
    },
    {
      "type": 2,
      "name": "players",
      "description": "placeholder",
      "options": [
        {
          "type": 1,
          "name": "add",
          "description": "placeholder",
          "options": [
            {
              "type": 9,
              "name": "name",
              "description": "placeholder",
              "required": true
            }
          ]
        },
        {
          "type": 1,
          "name": "remove",
          "description": "placeholder",
          "options": [
            {
              "type": 9,
              "name": "name",
              "description": "placeholder",
              "required": true
            }
          ]
        }
      ]
    }
  ]
}
</code>
</pre>
```
</details>
That schema generates the following code.  

<details>
<summary>generated code</summary>
<pre lang="rust">
<code>
pub mod ctf {
    pub mod add {
        use serde::{
            de::{SeqAccess, Visitor},
            Deserializer,
        };
        use std::fmt;
        #[derive(serde :: Serialize, Debug, Default)]
        pub struct Options {
            pub name: String,
        }
        impl<'de> serde::Deserialize<'de> for Options {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Options, D::Error> {
                struct PropertyParser;
                impl<'de> Visitor<'de> for PropertyParser {
                    type Value = Options;
                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("aaa")
                    }
                    fn visit_seq<A: SeqAccess<'de>>(
                        self,
                        mut seq: A,
                    ) -> Result<Self::Value, A::Error> {
                        #[allow(non_camel_case_types)]
                        #[derive(serde :: Deserialize, Debug)]
                        #[serde(tag = "name", content = "value")]
                        enum Property {
                            name(String),
                        }
                        let mut prop = Options::default();
                        while let Some(tmp) = seq.next_element::<Property>()? {
                            match tmp {
                                Property::name(v) => prop.name = v,
                            }
                        }
                        Ok(prop)
                    }
                }
                deserializer.deserialize_seq(PropertyParser)
            }
        }
    }
    pub mod archive {
        use serde::{
            de::{SeqAccess, Visitor},
            Deserializer,
        };
        use std::fmt;
        #[derive(serde :: Serialize, Debug, Default)]
        pub struct Options {
            pub channel: String,
        }
        impl<'de> serde::Deserialize<'de> for Options {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Options, D::Error> {
                struct PropertyParser;
                impl<'de> Visitor<'de> for PropertyParser {
                    type Value = Options;
                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("aaa")
                    }
                    fn visit_seq<A: SeqAccess<'de>>(
                        self,
                        mut seq: A,
                    ) -> Result<Self::Value, A::Error> {
                        #[allow(non_camel_case_types)]
                        #[derive(serde :: Deserialize, Debug)]
                        #[serde(tag = "name", content = "value")]
                        enum Property {
                            channel(String),
                        }
                        let mut prop = Options::default();
                        while let Some(tmp) = seq.next_element::<Property>()? {
                            match tmp {
                                Property::channel(v) => prop.channel = v,
                            }
                        }
                        Ok(prop)
                    }
                }
                deserializer.deserialize_seq(PropertyParser)
            }
        }
    }
    #[derive(serde :: Serialize, serde :: Deserialize, Debug)]
    #[serde(tag = "name", rename_all = "snake_case")]
    pub struct Ctf {
        pub id: String,
        #[serde(deserialize_with = "parse_single")]
        pub options: Options,
        pub resolved: Option<super::Resolved>,
    }
    #[derive(serde :: Serialize, serde :: Deserialize, Debug)]
    #[serde(tag = "name", content = "options", rename_all = "snake_case")]
    pub enum Options {
        Add(add::Options),
        Archive(archive::Options),
        #[serde(deserialize_with = "parse_single")]
        Players(players::Players),
    }
    use serde::{
        de::{Error, SeqAccess, Visitor},
        Deserializer,
    };
    use std::fmt;
    use std::marker::PhantomData;
    fn parse_single<'de, D: Deserializer<'de>, T: serde::Deserialize<'de>>(
        deserializer: D,
    ) -> Result<T, D::Error> {
        struct PropertyParser<T>(PhantomData<T>);
        impl<'de, T: serde::Deserialize<'de>> Visitor<'de> for PropertyParser<T> {
            type Value = T;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(
                    formatter,
                    "a nonempty list of {}",
                    std::any::type_name::<T>()
                )
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<T, A::Error> {
                seq.next_element::<T>()?
                    .ok_or_else(|| A::Error::custom("empty array"))
            }
        }
        deserializer.deserialize_seq(PropertyParser(PhantomData))
    }
    pub mod players {
        pub mod add {
            use serde::{
                de::{SeqAccess, Visitor},
                Deserializer,
            };
            use std::fmt;
            #[derive(serde :: Serialize, Debug, Default)]
            pub struct Options {
                pub name: String,
            }
            impl<'de> serde::Deserialize<'de> for Options {
                fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Options, D::Error> {
                    struct PropertyParser;
                    impl<'de> Visitor<'de> for PropertyParser {
                        type Value = Options;
                        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                            formatter.write_str("aaa")
                        }
                        fn visit_seq<A: SeqAccess<'de>>(
                            self,
                            mut seq: A,
                        ) -> Result<Self::Value, A::Error> {
                            #[allow(non_camel_case_types)]
                            #[derive(serde :: Deserialize, Debug)]
                            #[serde(tag = "name", content = "value")]
                            enum Property {
                                name(String),
                            }
                            let mut prop = Options::default();
                            while let Some(tmp) = seq.next_element::<Property>()? {
                                match tmp {
                                    Property::name(v) => prop.name = v,
                                }
                            }
                            Ok(prop)
                        }
                    }
                    deserializer.deserialize_seq(PropertyParser)
                }
            }
        }
        pub mod remove {
            use serde::{
                de::{SeqAccess, Visitor},
                Deserializer,
            };
            use std::fmt;
            #[derive(serde :: Serialize, Debug, Default)]
            pub struct Options {
                pub name: String,
            }
            impl<'de> serde::Deserialize<'de> for Options {
                fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Options, D::Error> {
                    struct PropertyParser;
                    impl<'de> Visitor<'de> for PropertyParser {
                        type Value = Options;
                        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                            formatter.write_str("aaa")
                        }
                        fn visit_seq<A: SeqAccess<'de>>(
                            self,
                            mut seq: A,
                        ) -> Result<Self::Value, A::Error> {
                            #[allow(non_camel_case_types)]
                            #[derive(serde :: Deserialize, Debug)]
                            #[serde(tag = "name", content = "value")]
                            enum Property {
                                name(String),
                            }
                            let mut prop = Options::default();
                            while let Some(tmp) = seq.next_element::<Property>()? {
                                match tmp {
                                    Property::name(v) => prop.name = v,
                                }
                            }
                            Ok(prop)
                        }
                    }
                    deserializer.deserialize_seq(PropertyParser)
                }
            }
        }
        #[derive(serde :: Serialize, serde :: Deserialize, Debug)]
        #[serde(tag = "name", content = "options")]
        #[serde(rename_all = "snake_case")]
        pub enum Players {
            Add(add::Options),
            Remove(remove::Options),
        }
    }
}
#[derive(serde :: Serialize, Debug)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum Interaction {
    Ping(Ping),
    ApplicationCommand(ApplicationCommand),
}
use serde::de::Error;
impl<'de> serde::Deserialize<'de> for Interaction {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Interaction, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        Ok(
            match value
                .get("type")
                .and_then(serde_json::Value::as_u64)
                .ok_or_else(|| D::Error::custom("type field is either missing or not u64"))?
            {
                1 => Interaction::Ping(
                    Ping::deserialize(value).map_err(|x| D::Error::custom(x.to_string()))?,
                ),
                2 => Interaction::ApplicationCommand(
                    ApplicationCommand::deserialize(value)
                        .map_err(|x| D::Error::custom(x.to_string()))?,
                ),
                _ => panic!("type isn't valid"),
            },
        )
    }
}
#[derive(serde :: Serialize, serde :: Deserialize, Debug)]
pub struct Ping {
    pub application_id: String,
    pub id: String,
    pub r#type: u64,
    pub token: String,
}
#[derive(serde :: Serialize, serde :: Deserialize, Debug)]
pub struct ApplicationCommand {
    pub application_id: String,
    pub channel_id: String,
    pub data: Command,
    pub guild_id: Option<String>,
    pub id: String,
    pub member: Option<PartialMember>,
    pub user: Option<User>,
    pub token: String,
    pub r#type: u64,
    pub version: u64,
}
#[derive(serde :: Serialize, serde :: Deserialize, Debug)]
#[serde(untagged)]
pub enum Command {
    Ctf(ctf::Ctf),
    Other { id: String, name: String },
}
#[derive(serde :: Serialize, serde :: Deserialize, Debug)]
pub struct User {
    pub id: String,
    pub username: String,
    pub discriminator: String,
    pub avatar: String,
    pub bot: Option<bool>,
    pub system: Option<bool>,
    pub mfa_enabled: Option<bool>,
    pub locale: Option<String>,
    pub verified: Option<bool>,
    pub email: Option<String>,
    pub flags: Option<u64>,
    pub premium_type: Option<u64>,
    pub public_flags: Option<u64>,
}
#[derive(serde :: Serialize, serde :: Deserialize, Debug)]
pub struct PartialMember {
    pub user: Option<User>,
    pub nick: Option<String>,
    pub roles: Vec<String>,
    pub joined_at: String,
    pub premium_since: Option<String>,
    pub deaf: Option<bool>,
    pub mute: Option<bool>,
    pub pending: Option<bool>,
    pub permissions: Option<String>,
}
use std::collections::HashMap;
#[derive(serde :: Serialize, serde :: Deserialize, Debug)]
pub struct Resolved {
    #[serde(default)]
    pub users: HashMap<String, User>,
    #[serde(default)]
    pub members: HashMap<String, PartialMember>,
    #[serde(default)]
    pub roles: HashMap<String, Role>,
    #[serde(default)]
    pub channels: HashMap<String, PartialChannel>,
}
#[derive(serde :: Serialize, serde :: Deserialize, Debug)]
pub struct Role {
    pub id: String,
    pub name: String,
    pub color: u64,
    pub hoist: bool,
    pub position: u64,
    pub permissions: String,
    pub managed: bool,
    pub mentionable: bool,
    pub tags: Option<RoleTags>,
}
#[derive(serde :: Serialize, serde :: Deserialize, Debug)]
pub struct RoleTags {
    pub bot_id: Option<String>,
    pub integration_id: Option<String>,
    pub premium_subscriber: Option<String>,
}
#[derive(serde :: Serialize, serde :: Deserialize, Debug)]
pub struct PartialChannel {
    pub id: String,
    pub r#type: u64,
    pub name: String,
    pub permissions: String,
}
</code>
</pre>
</details>
