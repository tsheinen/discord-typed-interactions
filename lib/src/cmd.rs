use crate::defer::Defer;
use crate::name::Name;

use serde::{
    de::{Error, IgnoredAny, MapAccess, Unexpected, Visitor},
    Deserialize, Deserializer,
};
use std::fmt;

#[derive(Debug, PartialEq)]
pub(crate) struct CommandOption {
    pub r#type: Option<Type>,
    pub name: Name,
    pub options: Vec<CommandOption>,
}

impl CommandOption {
    pub fn as_type(&self) -> Defer<&str> {
        match self.r#type.as_ref().unwrap() {
            Type::String => Defer("String"),
            Type::Bool => Defer("bool"),
            Type::U64 => Defer("u64"),
            Type::Subcommand => unreachable!("tried to print type of subcommand"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Type {
    String,
    Bool,
    U64,
    Subcommand,
}

impl<'de> Deserialize<'de> for CommandOption {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        enum Field {
            Type,
            Name,
            Options,
            Unknown,
        }
        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                struct FieldVisitor;
                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;
                    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        f.write_str("`type`, `name`, or `options`")
                    }
                    fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
                        Ok(match v {
                            "type" => Field::Type,
                            "name" => Field::Name,
                            "options" => Field::Options,
                            _ => Field::Unknown,
                        })
                    }
                }
                deserializer.deserialize_identifier(FieldVisitor)
            }
        }
        struct CommandOptionVisitor;
        impl<'de> Visitor<'de> for CommandOptionVisitor {
            type Value = CommandOption;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("struct CommandOption")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut r#type = None;
                let mut options = Vec::new();
                let mut name = None;
                // allow duplicates because JSON (last entry has precedence)
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Type => r#type = Some(map.next_value()?),
                        Field::Options => options = map.next_value()?,
                        Field::Name => name = map.next_value()?,
                        Field::Unknown => {
                            map.next_value::<IgnoredAny>()?;
                        }
                    }
                }
                Ok(CommandOption {
                    r#type,
                    options,
                    name: name.ok_or_else(|| Error::missing_field("name"))?,
                })
            }
        }
        deserializer.deserialize_struct(
            "CommandOption",
            &["type", "name", "options"],
            CommandOptionVisitor,
        )
    }
}

impl<'de> Deserialize<'de> for Type {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Type, D::Error> {
        struct TypeVisitor;
        impl<'de> Visitor<'de> for TypeVisitor {
            type Value = Type;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("1..=9")
            }
            // https://discord.com/developers/docs/interactions/slash-commands#data-models-and-types
            fn visit_u64<E: Error>(self, v: u64) -> Result<Self::Value, E> {
                match v {
                    4 => Ok(Type::U64),
                    5 => Ok(Type::Bool),
                    3 | 6..=9 => Ok(Type::String),
                    1 | 2 => Ok(Type::Subcommand),
                    _ => Err(E::invalid_value(Unexpected::Unsigned(v), &self)),
                }
            }
        }
        deserializer.deserialize_u64(TypeVisitor)
    }
}

impl<'de> Deserialize<'de> for Name {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Name, D::Error> {
        struct NameVisitor;
        impl<'de> Visitor<'de> for NameVisitor {
            type Value = Name;

            // https://discord.com/developers/docs/interactions/slash-commands#registering-a-command
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a string matching the regex `^[\\w-]{1,32}$`")
            }
            fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
                Name::new(v).ok_or_else(|| E::invalid_value(Unexpected::Str(v), &self))
            }
        }
        deserializer.deserialize_str(NameVisitor)
    }
}
