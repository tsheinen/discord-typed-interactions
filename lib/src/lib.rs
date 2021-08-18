use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use serde::{
    de::{Error, Unexpected, Visitor},
    Deserialize, Deserializer,
};
use std::fmt;

mod defer;
mod name;

use defer::Defer;
use name::Name;

#[derive(Debug, Deserialize, PartialEq)]
struct CommandOption {
    #[serde(default, deserialize_with = "parse_type")]
    r#type: Option<Type>,
    #[serde(deserialize_with = "parse_name")]
    name: Name,
    #[serde(default)]
    options: Vec<CommandOption>,
}

#[derive(Debug, PartialEq, Eq)]
enum Type {
    String,
    Bool,
    U64,
    Subcommand,
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

fn parse_type<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<Type>, D::Error> {
    struct TypeVisitor;
    impl<'de> Visitor<'de> for TypeVisitor {
        type Value = Option<Type>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("1..=9")
        }
        // https://discord.com/developers/docs/interactions/slash-commands#data-models-and-types
        fn visit_u64<E: Error>(self, v: u64) -> Result<Self::Value, E> {
            match v {
                4 => Ok(Some(Type::U64)),
                5 => Ok(Some(Type::Bool)),
                3 | 6..=9 => Ok(Some(Type::String)),
                1 | 2 => Ok(Some(Type::Subcommand)),
                _ => Err(E::invalid_value(Unexpected::Unsigned(v), &self)),
            }
        }
    }
    deserializer.deserialize_u64(TypeVisitor)
}

fn parse_name<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Name, D::Error> {
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

fn structify_data(input: &CommandOption) -> impl ToTokens + '_ {
    Defer(move || {
        let kinds = input.options.iter().map(|x| x.as_type());
        let names = input.options.iter().map(|x| x.name.snake());
        let mod_ident = input.name.snake();

        let kinds2 = input.options.iter().map(|opt| opt.as_type());
        let idents = input.options.iter().map(|opt| opt.name.snake());
        let idents2 = input.options.iter().map(|opt| opt.name.snake());

        quote! {
            pub mod #mod_ident {
                use serde::{de::{SeqAccess, Visitor}, Deserializer};
                use std::fmt;

                #[derive(serde::Serialize, Debug, Default)]
                pub struct Options {
                    #(pub #names: #kinds,)*
                }
                impl<'de> serde::Deserialize<'de> for Options {
                    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Options, D::Error> {
                        struct PropertyParser;
                        impl<'de> Visitor<'de> for PropertyParser {
                            type Value = Options;
                            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                                // TODO actually write this lol
                                formatter.write_str("aaa")
                            }

                            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                                #[allow(non_camel_case_types)]
                                #[derive(serde::Deserialize, Debug)]
                                #[serde(tag = "name", content = "value")]
                                enum Property {
                                    #(#idents(#kinds2),)*
                                }

                                let mut prop = Options::default();
                                while let Some(tmp) = seq.next_element::<Property>()? {
                                    match tmp {
                                        #(Property::#idents2(v) => prop.#idents2 = v,)*
                                    }
                                }
                                Ok(prop)
                            }
                        }
                        deserializer.deserialize_seq(PropertyParser)
                    }
                }
            }
        }
    })
}

fn extract_modules(
    schema: &CommandOption,
) -> (Vec<&CommandOption>, Vec<(&Name, Vec<&CommandOption>)>) {
    fn recurse<'schema>(
        next: &'schema CommandOption,
        path: &mut Vec<&'schema Name>,
        root: &mut Vec<&'schema CommandOption>,
        modules: &mut Vec<(&'schema Name, Vec<&'schema CommandOption>)>,
    ) {
        if !next.options.is_empty() {
            if next.options.iter().all(|x| x.options.is_empty()) {
                if let Some(x) = path.get(1) {
                    // should be correct as long as the traversal groups names together
                    if !modules.is_empty() && &modules.last().unwrap().0 == x {
                        modules.last_mut().unwrap().1.push(next);
                    } else {
                        modules.push((x, vec![next]));
                    }
                } else {
                    root.push(next);
                }
            }
            path.push(&next.name);
            for i in &next.options {
                recurse(i, path, root, modules);
            }
            path.pop();
        }
    }
    let mut root = Vec::new();
    let mut modules = Vec::new();
    recurse(schema, &mut Vec::new(), &mut root, &mut modules);
    (root, modules)
}

fn generate_interaction_struct<'a>(
    commands: impl IntoIterator<Item = &'a Name>,
) -> impl ToTokens + 'a {
    let commands = commands.into_iter().collect::<Vec<_>>();
    Defer(move || {
        let command_enum_tokens = commands.iter().map(|x| {
            let name_camel = x.camel();
            let name_snake = x.snake();
            quote! {
                #name_camel(#name_snake::#name_camel)
            }
        });
        quote! {
            #[derive(serde::Serialize, Debug)]
            #[serde(tag = "type")]
            #[non_exhaustive]
            pub enum Interaction {
                Ping(Ping),
                ApplicationCommand(ApplicationCommand),
            }
            use serde::de::Error;
            // the issue which would let me do this via derive is 4 years old https://github.com/serde-rs/serde/issues/745 </3
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
                                Ping::deserialize(value).map_err(|x| {
                                    D::Error::custom(x.to_string())
                                })?,
                            ),
                            2 =>
                            Interaction::ApplicationCommand(
                                ApplicationCommand::deserialize(value).map_err(|x| {
                                    D::Error::custom(x.to_string())
                                })?,
                            ),
                            _ => panic!("type isn't valid"),
                        },
                    )
                }
            }

            #[derive(serde::Serialize, serde::Deserialize, Debug)]
            pub struct Ping {
                application_id: String,
                id: String,
                r#type: u64,
                token: String
            }

            #[derive(serde::Serialize, serde::Deserialize, Debug)]
            pub struct ApplicationCommand {
                application_id: String,
                channel_id: String,
                data: Command,
                guild_id: String,
                id: String,
                member: Option<PartialMember>,
                user: Option<User>,
                token: String,
                r#type: u64,
                version: u64,
            }

            #[derive(serde::Serialize, serde::Deserialize, Debug)]
            #[serde(untagged)]
            pub enum Command {
                #(#command_enum_tokens),*
            }

            #[derive(serde::Serialize, serde::Deserialize, Debug)]
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

            #[derive(serde::Serialize, serde::Deserialize, Debug)]
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

        }
    })
}

fn generate_resolved_structs(resolved_struct: Option<&str>) -> impl ToTokens {
    Defer((resolved_struct.is_none(), || {
        quote! {
            use std::collections::HashMap;

            #[derive(serde::Serialize, serde::Deserialize, Debug)]
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

            #[derive(serde::Serialize, serde::Deserialize, Debug)]
            struct Role {
                pub id: String,
                pub name: String,
                pub color: u64,
                pub hoist: bool,
                pub position: u64,
                pub permissions: String,
                pub managed: bool,
                pub mentionable: bool,
                pub tags: Option<RoleTags>
            }

            #[derive(serde::Serialize, serde::Deserialize, Debug)]
            pub struct RoleTags {
                pub bot_id: Option<String>,
                pub integration_id: Option<String>,
                pub premium_subscriber: Option<String>,
            }

            #[derive(serde::Serialize, serde::Deserialize, Debug)]
            pub struct PartialChannel {
                pub id: String,
                pub r#type: u64,
                pub name: String,
                pub permissions: String
            }

        }
    }))
}

pub fn typify_driver(
    input: impl IntoIterator<Item = impl AsRef<str>>,
    resolved_struct: Option<&str>,
) -> TokenStream {
    let (names, tokens): (Vec<_>, Vec<_>) = input
        .into_iter()
        .map(|x| generate_command_data(x, resolved_struct))
        .unzip();
    let resolved_code = generate_resolved_structs(resolved_struct);
    let interaction_struct = generate_interaction_struct(&names);

    quote! {
        #(#tokens)*

        #interaction_struct

        #resolved_code
    }
}

fn generate_command_data<'a>(
    input: impl AsRef<str> + 'a,
    resolved_struct: Option<&'a str>,
) -> (Name, impl ToTokens + 'a) {
    let schema: CommandOption = serde_json::from_str(input.as_ref()).unwrap();

    (
        schema.name.clone(),
        Defer(move || {
            let (root, modules) = extract_modules(&schema);

            let root_name_camelcase = schema.name.camel();
            let root_name = schema.name.snake();
            let subcommand_struct_tokens = modules.iter().map(|(k, v)| {
                Defer(move || {
                    let mod_ident = k.snake();
                    let enum_ident = k.camel();
                    let fields = v
                        .iter()
                        .map(|x| (!x.options.is_empty()).then(|| structify_data(x)));
                    let type_idents = v.iter().map(|x| x.name.snake());
                    let type_idents_camelcase = v.iter().map(|x| x.name.camel());
                    quote! {
                        pub mod #mod_ident {
                            #(#fields)*

                            #[derive(serde::Serialize, serde::Deserialize, Debug)]
                            #[serde(tag = "name", content = "options")]
                            #[serde(rename_all = "snake_case")]
                            pub enum #enum_ident {
                                #(#type_idents_camelcase(#type_idents::Options),)*
                            }

                        }
                    }
                })
            });
            let has_options = root.iter().any(|x| x.r#type.is_none());
            let options_type_tokens = Defer(|| {
                if has_options {
                    let x = root.first().expect("root to be nonempty");
                    let x_ident = x.name.snake();
                    quote! { pub options: #x_ident::Options }
                } else {
                    quote! {
                        #[serde(deserialize_with = "parse_single")]
                        pub options: Options
                    }
                }
            });
            let options_enum_tokens = Defer((!has_options, || {
                let root_enum_snake = root.iter().map(|x| x.name.snake());
                let root_enum_camel = root.iter().map(|x| x.name.camel());
                let root_module_snake = modules.iter().map(|(x, _)| x.snake());
                let root_module_camel = modules.iter().map(|(x, _)| x.camel());
                // this deserializer relies on the assumption that there can only be a single subcommand active at a time
                quote! {
                    #[derive(serde::Serialize, serde::Deserialize, Debug)]
                    #[serde(tag = "name", content = "options", rename_all = "snake_case")]
                    pub enum Options {
                        #(#root_enum_camel(#root_enum_snake::Options),)*
                        #[serde(deserialize_with = "parse_single")]
                        #(#root_module_camel(#root_module_snake::#root_module_camel),)*
                    }

                    use serde::{de::{SeqAccess, Visitor, Error}, Deserializer};
                    use std::fmt;
                    use std::marker::PhantomData;

                    fn parse_single<'de, D: Deserializer<'de>, T: serde::Deserialize<'de>>(deserializer: D) -> Result<T, D::Error> {
                        struct PropertyParser<T>(PhantomData<T>);
                        impl<'de, T: serde::Deserialize<'de>> Visitor<'de> for PropertyParser<T> {
                            type Value = T;
                            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                                write!(formatter, "a nonempty list of {}", std::any::type_name::<T>())
                            }

                            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<T, A::Error> {
                                seq.next_element::<T>()?.ok_or_else(|| A::Error::custom("empty array"))

                            }
                        }
                        deserializer.deserialize_seq(PropertyParser(PhantomData))
                    }
                }
            }));
            let resolved_type = Defer(move || {
                if let Some(name) = resolved_struct {
                    let ident = Defer(name);
                    quote! { #ident }
                } else {
                    quote! { super::Resolved }
                }
            });
            let root_struct_tokens = root
                .iter()
                .map(|x| (!x.options.is_empty()).then(|| structify_data(x)));
            quote! {
                pub mod #root_name {
                    #(#root_struct_tokens)*

                    #[derive(serde::Serialize, serde::Deserialize, Debug)]
                    pub struct #root_name_camelcase {
                        pub id: String,
                        pub name: String,
                        #options_type_tokens,
                        pub resolved: Option<#resolved_type>,
                    }


                    #options_enum_tokens

                    #(#subcommand_struct_tokens)*
                }

            }
        }),
    )
}

#[cfg(test)]
mod tests {
    use crate::{extract_modules, CommandOption, Name, Type};
    use serde_json::json;

    #[test]
    fn deserializes_command_option() {
        let x: CommandOption = serde_json::from_value(json!({
            "type": 4,
            "name": "abc"
        }))
        .unwrap();
        assert_eq!(
            x,
            CommandOption {
                name: Name::new("abc").unwrap(),
                r#type: Some(Type::U64),
                options: vec![],
            }
        );
    }

    #[test]
    fn extracts_modules() {
        let cmd_option = serde_json::from_str(include_str!(
            "../../test-harness/schema/multiple_subgroups.json"
        ))
        .unwrap();
        let (_root, _submodules) = extract_modules(&cmd_option);
    }
}
