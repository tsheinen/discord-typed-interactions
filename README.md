# discord-typed-interactions

I was writing a discord bot and all the dynamic command data checking was really painful so I was inspired to not do that ever again. Thus, this. 

A few points to note:
* Input paths are relative to Cargo.toml; include_str! is a compiler built-in and we don't have any easy way to replicate that behavior.
* We do not re-export serde, so you will need to depend on serde for the generated code to compile.  

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
We haven't finalized the builder yet so enjoy this meme which I made while avoiding writing this readme :)
![](https://imgflip.com/i/5iw3di)

## generated code

```json
{
  "name": "ctf",
  "description": "placeholder",
  "options": [
    {
      "type": 1,
      "name": "play",
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
      "type": 1,
      "name": "chall",
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
      "name": "solve",
      "description": "placeholder",
      "options": [
        {
          "type": 3,
          "name": "flag",
          "description": "placeholder",
          "required": true
        },
        {
          "type": 7,
          "name": "channel",
          "description": "placeholder"
        },
        {
          "type": 4,
          "name": "points",
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
```

That schema generates the following code.  

```rust
pub mod ctf {
    pub mod play {
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
    pub mod chall {
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
    pub mod solve {
        use serde::{
            de::{SeqAccess, Visitor},
            Deserializer,
        };
        use std::fmt;
        #[derive(serde :: Serialize, Debug, Default)]
        pub struct Options {
            pub flag: String,
            pub channel: String,
            pub points: u64,
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
                            flag(String),
                            channel(String),
                            points(u64),
                        }
                        let mut prop = Options::default();
                        while let Some(tmp) = seq.next_element::<Property>()? {
                            match tmp {
                                Property::flag(v) => prop.flag = v,
                                Property::channel(v) => prop.channel = v,
                                Property::points(v) => prop.points = v,
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
    pub struct Ctf {
        pub id: String,
        pub name: String,
        #[serde(deserialize_with = "parse_single")]
        pub options: Options,
        pub resolved: Option<super::Resolved>,
    }
    #[derive(serde :: Serialize, serde :: Deserialize, Debug)]
    #[serde(tag = "name", content = "options", rename_all = "snake_case")]
    pub enum Options {
        Play(play::Options),
        Archive(archive::Options),
        Chall(chall::Options),
        Solve(solve::Options),
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
use std::collections::HashMap;
#[derive(serde :: Serialize, serde :: Deserialize, Debug)]
pub struct Resolved {
    #[serde(default)]
    users: HashMap<String, User>,
    #[serde(default)]
    members: HashMap<String, PartialMember>,
    #[serde(default)]
    roles: HashMap<String, Role>,
    #[serde(default)]
    channels: HashMap<String, PartialChannel>,
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
#[derive(serde :: Serialize, serde :: Deserialize, Debug)]
struct Role {
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
```