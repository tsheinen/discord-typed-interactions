use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use serde::{
    de::{Error, Unexpected, Visitor},
    Deserialize, Deserializer,
};
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};

mod casing;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
struct CommandOption {
    #[serde(default, deserialize_with = "parse_type")]
    r#type: Option<Type>,
    #[serde(deserialize_with = "parse_name")]
    name: Name,
    options: Option<Vec<CommandOption>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Type {
    String,
    Bool,
    U64,
}

#[derive(Clone, Debug, Eq)]
pub(crate) struct Name {
    snake: Ident,
    camel: Ident,
}

// NOTE: camel-case might be shorter by a few characters
impl PartialEq for Name {
    fn eq(&self, other: &Self) -> bool {
        self.camel == other.camel
    }
}

impl Hash for Name {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.camel.hash(state);
    }
}

impl CommandOption {
    fn print_kind(&self) -> Ident {
        match self.r#type.as_ref().unwrap() {
            Type::String => mk_ident("String"),
            Type::Bool => mk_ident("bool"),
            Type::U64 => mk_ident("u64"),
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
                1 | 2 => Ok(None),
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
            f.write_str("Command names must match the regex `^[\\w-]{1,32}$`")
        }
        fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
            casing::mk_name(v).ok_or_else(|| E::invalid_value(Unexpected::Str(v), &self))
        }
    }
    deserializer.deserialize_str(NameVisitor)
}

fn structify_data(input: &CommandOption) -> Option<TokenStream> {
    let opts = input.options.as_ref()?;
    let kinds = opts.iter().map(|x| x.print_kind());
    let names = opts.iter().map(|x| &x.name.snake);
    let mod_ident = &input.name.snake;

    let visit_seq_body = if opts.is_empty() {
        quote! {
            Ok(Options {})
        }
    } else {
        let kinds = opts.iter().map(|opt| opt.print_kind());
        let idents = opts.iter().map(|opt| &opt.name.snake);
        let idents2 = opts.iter().map(|opt| &opt.name.snake);
        quote! {
            #[allow(non_camel_case_types)]
            #[derive(serde::Deserialize, Debug)]
            #[serde(tag = "name", content = "value")]
            enum Property {
                #(#idents(#kinds),)*
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
    };

    Some(quote! {
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
                            #visit_seq_body
                        }
                    }
                    deserializer.deserialize_seq(PropertyParser)
                }
            }
        }
    })
}

fn extract_modules(
    schema: &CommandOption,
) -> (Vec<&CommandOption>, HashMap<&Name, Vec<&CommandOption>>) {
    fn recurse<'schema>(
        next: &'schema CommandOption,
        path: &mut Vec<&'schema Name>,
        root: &mut Vec<&'schema CommandOption>,
        modules: &mut HashMap<&'schema Name, Vec<&'schema CommandOption>>,
    ) {
        if let Some(arr) = next.options.as_ref() {
            if arr.iter().all(|x| x.options.is_none()) {
                if let Some(x) = path.get(1) {
                    modules
                        .entry(x)
                        .and_modify(|v| v.push(next))
                        .or_insert_with(|| vec![next]);
                } else {
                    root.push(next);
                }
            }
            path.push(&next.name);
            for i in arr {
                recurse(i, path, root, modules);
            }
            path.pop();
        }
    }
    let mut root = Vec::new();
    let mut modules = HashMap::new();
    recurse(schema, &mut Vec::new(), &mut root, &mut modules);
    (root, modules)
}

fn mk_ident(input: &str) -> Ident {
    Ident::new(input, Span::call_site())
}

pub fn typify_driver(input: &str) -> TokenStream {
    let schema: CommandOption = serde_json::from_str(input).unwrap();

    let (root, modules) = extract_modules(&schema);

    let root_name_camelcase = &schema.name.camel;
    let root_name = &schema.name.snake;
    let subcommand_struct_tokens = modules.iter().map(|(k, v)| {
        let mod_ident = &k.snake;
        let enum_ident = &k.camel;
        let fields = v.iter().flat_map(|x| structify_data(x));
        let type_idents = v.iter().map(|x| &x.name.snake);
        let type_idents_camelcase = v.iter().map(|x| &x.name.camel);
        quote! {
            pub mod #mod_ident {
                #(#fields)*
                pub mod cmd {
                    #[derive(serde::Serialize, serde::Deserialize, Debug)]
                    #[serde(tag = "name", content = "options")]
                    #[serde(rename_all = "snake_case")]
                    pub enum #enum_ident {
                        #(#type_idents_camelcase(crate::#root_name::#mod_ident::#type_idents::Options),)*
                    }
                }
            }
        }
    });
    let root_enum_snake = root.iter().map(|x| &x.name.snake);
    let root_enum_camel = root.iter().map(|x| &x.name.camel);
    let root_module_snake = modules.keys().map(|x| &x.snake);
    let root_module_camel = modules.keys().map(|x| &x.camel);
    let (options_type_tokens, options_enum_tokens) = if root.iter().any(|x| x.r#type.is_none()) {
        let x = root.first().expect("root to be nonempty");
        let x_ident = &x.name.snake;
        (quote! { crate::#root_name::#x_ident::Options }, quote! {})
    } else {
        (
            quote! {Vec<Options>},
            quote! {
                #[derive(serde::Serialize, serde::Deserialize, Debug)]
                #[serde(tag = "name", content = "options", rename_all = "snake_case")]
                pub enum Options {
                    #(#root_enum_camel(crate::#root_name::#root_enum_snake::Options),)*
                    #(#root_module_camel(Vec<crate::#root_name::#root_module_snake::cmd::#root_module_camel>),)*
                }
            },
        )
    };
    let root_struct_tokens = root.iter().flat_map(|x| structify_data(x));
    quote! {
        pub mod #root_name {
            #(#root_struct_tokens)*
            pub mod cmd {
                #[derive(serde::Serialize, serde::Deserialize, Debug)]
                pub struct #root_name_camelcase {
                    id: String,
                    name: String,
                    options: #options_type_tokens
                }

                #options_enum_tokens

            }
            #(#subcommand_struct_tokens)*
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{mk_ident, CommandOption, Name, Type};
    use heck::{CamelCase, SnakeCase};
    use serde_json::json;

    #[test]
    fn deserializes_command_option() {
        let x: CommandOption = serde_json::from_value(json!({
            "type": 4,
            "name": "abc"
        }))
        .unwrap();
        assert_eq!(x.r#type, Some(Type::U64));
        assert_eq!(
            x.name,
            Name {
                snake: mk_ident(&"abc".to_snake_case()),
                camel: mk_ident(&"abc".to_camel_case()),
            }
        );
    }
}
