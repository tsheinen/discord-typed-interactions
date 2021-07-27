use heck::{CamelCase, SnakeCase};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CommandOption {
    // TODO: ensure zero is caught as an illegal type for subcommands at runtime,
    // or make a top-level struct without a `type` field
    #[serde(default)]
    r#type: u8,
    name: String,
    options: Option<Vec<CommandOption>>,
}

impl CommandOption {
    fn print_kind(&self) -> &'static str {
        match self.r#type {
            4 => "u64",
            5 => "bool",
            3 | 6 | 7 | 8 | 9 => "String",
            invalid => panic!("invalid CommandOption kind {}", invalid),
        }
    }
}

pub fn structify_data(input: Option<&CommandOption>) -> Option<TokenStream> {
    let val = input?;
    let name = mk_ident(&val.name.to_camel_case());
    let fields = val.options.as_ref()?.iter().map(|x| {
        let kind = mk_ident(x.print_kind());
        let name = mk_ident(&x.name);
        quote! {
           pub #name: #kind
        }
    });
    let options_struct_ident = format_ident!("{}Options", name);
    let mod_ident = mk_ident(&val.name.to_snake_case());
    Some(quote! {
        pub mod #mod_ident {
            pub struct #name {
                pub id: u64,
                pub name: String,
                pub options: #options_struct_ident,
            }
            pub struct #options_struct_ident {
                #(#fields),*,
            }
        }
    })
}

pub fn extract_modules(schema: &CommandOption) -> Vec<(Vec<&str>, &CommandOption)> {
    fn recurse<'schema>(
        next: &'schema CommandOption,
        path: &mut Vec<&'schema str>,
        output: &mut Vec<(Vec<&'schema str>, &'schema CommandOption)>,
    ) -> Option<()> {
        let arr = next.options.as_ref()?;
        if arr.iter().all(|x| x.options.is_none()) {
            output.push((path.clone(), next));
        }
        path.push(&next.name);
        for i in arr.iter() {
            recurse(i, path, output);
        }
        path.pop();
        Some(())
    }
    let mut output = Vec::new();
    recurse(schema, &mut Vec::new(), &mut output);
    output
}

fn mk_enum_field(input: &str, root_name: &Ident) -> TokenStream {
    let snake_case_ident = mk_ident(&input.to_snake_case());
    let camel_case_ident = mk_ident(&input.to_camel_case());
    quote! {
        #camel_case_ident(crate::#root_name::#snake_case_ident::#camel_case_ident)
    }
}

fn mk_ident(input: &str) -> Ident {
    Ident::new(input, Span::call_site())
}

pub fn structify(input: &str) -> TokenStream {
    let schema: CommandOption = serde_json::from_str(input).unwrap();

    let output = extract_modules(&schema);

    let mut root = Vec::new();
    let mut modules: HashMap<&str, Vec<&CommandOption>> = HashMap::new();
    for (key, val) in output {
        if key.len() == 1 {
            root.push(val);
        } else {
            modules
                .entry(key[1])
                .and_modify(|k| k.push(val))
                .or_insert_with(|| vec![val]);
        }
    }
    let root_name_camelcase = mk_ident(&schema.name.to_camel_case());
    let root_name = mk_ident(&schema.name);
    let subcommand_struct_tokens = modules.iter().map(|(k, v)| {
        let mod_ident = mk_ident(k);
        let enum_ident = mk_ident(&k.to_camel_case());
        let fields = v.iter().flat_map(|x| structify_data(Some(x)));
        let enum_tokens = v.iter().map(|x| mk_enum_field(&x.name, &root_name));
        quote! {
            pub mod #mod_ident {
                #(#fields)*
                pub mod cmd {
                    pub enum #enum_ident {
                        #(#enum_tokens),*,
                    }
                }
            }
        }
    });

    let root_enum_tokens = root.iter().map(|x| mk_enum_field(&x.name, &root_name));
    let root_module_tokens = modules.keys().map(|x| mk_enum_field(x, &root_name));
    let root_struct_tokens = root.iter().flat_map(|x| structify_data(Some(x)));
    let token = quote! {
        pub mod #root_name {
            #(#root_struct_tokens)*
            pub mod cmd {
                pub enum #root_name_camelcase {
                    #(#root_enum_tokens),*,
                    #(#root_module_tokens),*,
                }
            }
            #(#subcommand_struct_tokens)*
        }
    };
    token
}

#[cfg(test)]
mod tests {
    use crate::{structify, structify_data};
    use quote::quote;
    use serde_json::json;

    #[test]
    fn command_data_no_options() {
        let experimental = structify_data(Some(
            &serde_json::from_value(json!({
                "name": "test",
                "description": "test",
                "options": []
            }))
            .unwrap(),
        ))
        .unwrap()
        .to_string();
        let correct = quote! {
            pub mod test {
                pub struct Test {
                    pub id: u64,
                    pub name: String,
                    pub options: TestOptions,
                }
                pub struct TestOptions {,}
            }
        }
        .to_string();
        assert_eq!(experimental, correct);
    }

    #[test]
    fn command_data_no_subcommand() {
        let experimental = structify_data(Some(
            &serde_json::from_value(json!({
                "name": "test",
                "description": "test",
                "options": [
                    {
                        "name": "opt",
                        "description": "opt1",
                        "type": 3,
                        "required": true
                    }
                ]
            }))
            .unwrap(),
        ))
        .unwrap()
        .to_string();
        let correct = quote! {
            pub mod test {
                pub struct Test {
                    pub id: u64,
                    pub name: String,
                    pub options: TestOptions,
                }
                pub struct TestOptions {
                    pub opt: String,
                }
            }
        }
        .to_string();
        assert_eq!(experimental, correct);
    }

    #[test]
    fn real_life() {
        let experimental = structify(
            &json!({
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
            )
            .to_string(),
        )
        .to_string();
        let correct = quote! {
            pub mod ctf {
                pub mod play {
                    pub struct Play {
                        pub id: u64,
                        pub name: String,
                        pub options: PlayOptions,
                    }
                    pub struct PlayOptions {
                        pub name: String,
                    }
                }
                pub mod archive {
                    pub struct Archive {
                        pub id: u64,
                        pub name: String,
                        pub options: ArchiveOptions,
                    }
                    pub struct ArchiveOptions {
                        pub channel: String,
                    }
                }
                pub mod chall {
                    pub struct Chall {
                        pub id: u64,
                        pub name: String,
                        pub options: ChallOptions,
                    }
                    pub struct ChallOptions {
                        pub name: String,
                    }
                }
                pub mod solve {
                    pub struct Solve {
                        pub id: u64,
                        pub name: String,
                        pub options: SolveOptions,
                    }
                    pub struct SolveOptions {
                        pub flag: String,
                        pub channel: String,
                        pub points: u64,
                    }
                }
                pub mod cmd {
                    pub enum Ctf {
                        Play(crate::ctf::play::Play),
                        Archive(crate::ctf::archive::Archive),
                        Chall(crate::ctf::chall::Chall),
                        Solve(crate::ctf::solve::Solve),
                        Players(crate::ctf::players::Players),
                    }
                }
                pub mod players {
                    pub mod add {
                        pub struct Add {
                            pub id: u64,
                            pub name: String,
                            pub options: AddOptions,
                        }
                        pub struct AddOptions {
                            pub name: String,
                        }
                    }
                    pub mod remove {
                        pub struct Remove {
                            pub id: u64,
                            pub name: String,
                            pub options: RemoveOptions,
                        }
                        pub struct RemoveOptions {
                            pub name: String,
                        }
                    }
                    pub mod cmd {
                        pub enum Players {
                            Add(crate::ctf::add::Add),
                            Remove(crate::ctf::remove::Remove),
                        }
                    }
                }
            }

        }
        .to_string();
        assert_eq!(experimental, correct);
    }
}
