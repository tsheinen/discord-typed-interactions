use proc_macro2::TokenStream;
use quote::quote;
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

fn structify_data(input: &CommandOption) -> Defer<impl Fn() -> TokenStream + '_> {
        Defer(move || {
            let kinds = input.options.iter().map(|x| x.as_type());
            let names = input.options.iter().map(|x| x.name.snake());
            let mod_ident = input.name.snake();

            let kinds2 = input.options.iter().map(|opt| opt.as_type());
            let idents = input.options.iter().map(|opt| opt.name.snake());
            let idents2 = input.options.iter().map(|opt| opt.name.snake());

            quote! {
                pub mod #mod_ident {
                    use serde::{de::{SeqAccess, Visitor}, Deserializer, Serialize, Deserialize};
                    use std::fmt::{self, Write};

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
                                    if let Ok(Some(tmp)) = seq.next_element::<Options>() {
                                        Ok(tmp)
                                    } else {
                                        let mut prop = Options::default();
                                        while let Some(tmp) = seq.next_element::<Property>()? {
                                            match tmp {
                                                #(Property::#idents2(v) => prop.#idents2 = v,)*
                                            }
                                        }
                                        Ok(prop)
                                    }
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

#[inline]
fn generate_resolved_structs(resolved_struct: Option<String>) -> (TokenStream, TokenStream) {
    resolved_struct
        .map(|name| (name.parse().unwrap(), quote! {}))
        .unwrap_or_else(|| {
            (
               "crate::Resolved".parse().unwrap(),
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

                },
            )
        })
}
pub fn typify_driver(input: &str) -> TokenStream {
    let schema: CommandOption = serde_json::from_str(input).unwrap();

    let (root, modules) = extract_modules(&schema);

    let root_name_camelcase = schema.name.camel();
    let root_name = schema.name.snake();
    let subcommand_struct_tokens = modules.iter().map(|(k, v)| {
        Defer(move || {
            let mod_ident = k.snake();
            let enum_ident = k.camel();
            let fields = v.iter().map(|x| (!x.options.is_empty()).then(|| structify_data(x)));
            let type_idents = v.iter().map(|x| x.name.snake());
            let type_idents_camelcase = v.iter().map(|x| x.name.camel());
            quote! {
                pub mod #mod_ident {
                    #(#fields)*

                    #[derive(serde::Serialize, serde::Deserialize, Debug)]
                    #[serde(tag = "name", content = "options")]
                    #[serde(rename_all = "snake_case")]
                    pub enum #enum_ident {
                        #(#type_idents_camelcase(crate::#root_name::#mod_ident::#type_idents::Options),)*
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
            quote! { pub options: crate::#root_name::#x_ident::Options }
        } else {
            quote! {
                #[serde(deserialize_with = "parse_options")]
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
                #(#root_enum_camel(crate::#root_name::#root_enum_snake::Options),)*
                #(#root_module_camel(Vec<crate::#root_name::#root_module_snake::#root_module_camel>),)*
            }

            use serde::{de::{SeqAccess, Visitor, Error}, Deserializer, Serialize, Deserialize};
            use std::fmt::{self, Write};

            fn parse_options<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Options, D::Error> {
                struct PropertyParser;
                impl<'de> Visitor<'de> for PropertyParser {
                    type Value = Options;
                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("a map matching the root Options enum")
                    }

                    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                        seq.next_element::<Options>()?.ok_or(A::Error::custom("empty array"))

                    }
                }
                deserializer.deserialize_seq(PropertyParser)
            }
        }
    }));

    let (resolved_type, resolved_code) = generate_resolved_structs(None);

    let root_struct_tokens = root.iter().map(|x| (!x.options.is_empty()).then(|| structify_data(x)));
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

        #resolved_code
    }
}

#[cfg(test)]
mod tests {
    use crate::{CommandOption, Name, Type, extract_modules};
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
        let cmd_option = serde_json::from_str(include_str!("../../test-harness/schema/multiple_subgroups.json")).unwrap();
        let (_root, _submodules) = extract_modules(&cmd_option);
    }
}
