use heck::{CamelCase, SnakeCase};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
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

pub fn generate_deserialize_impl(opts: &[CommandOption]) -> TokenStream {
    if opts.is_empty() {
        return quote! {
            impl<'de> serde::Deserialize<'de> for Options {
                fn deserialize<D>(deserializer: D) -> Result<Options, D::Error>
                    where
                        D: Deserializer<'de>,
                    {
                        struct PropertyParser;
                        impl<'de> Visitor<'de> for PropertyParser {
                            type Value = Options;

                            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                                // TODO actually write this lol
                                formatter.write_str("aaa")
                            }

                            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                                Ok(Options {})
                            }
                        }
                        deserializer.deserialize_seq(PropertyParser {})
                    }
                }
        };
    }
    let enum_fields = opts.iter().map(|opt| {
        let ident_snake_case = mk_ident(&opt.name.to_snake_case());
        let type_ident = mk_ident(opt.print_kind());
        quote! {
            #ident_snake_case(#type_ident)
        }
    });

    let match_fields = opts.iter().map(|opt| {
        let ident_snake_case = mk_ident(&opt.name.to_snake_case());
        quote! {
            Property::#ident_snake_case(v) => prop.#ident_snake_case = v
        }
    });

    quote! {
        impl<'de> serde::Deserialize<'de> for Options {
            fn deserialize<D>(deserializer: D) -> Result<Options, D::Error>
                where
                    D: Deserializer<'de>,
                {
                    #[allow(non_camel_case_types)]
                    #[derive(serde::Deserialize, Debug)]
                    #[serde(tag = "name", content = "value")]
                    enum Property {
                        #(#enum_fields,)*
                    }

                    struct PropertyParser;
                    impl<'de> Visitor<'de> for PropertyParser {
                        type Value = Options;

                        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                            // TODO actually write this lol
                            formatter.write_str("aaa")
                        }

                        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                            if let Ok(Some(tmp)) = seq.next_element::<Options>() {
                                Ok(tmp)
                            } else {
                                let mut prop = Options::default();
                                while let Some(tmp) = seq.next_element::<Property>()? {
                                    match tmp {
                                        #(#match_fields,)*
                                    }
                                }
                                Ok(prop)
                            }
                        }
                    }
                    deserializer.deserialize_seq(PropertyParser {})
                }
            }
    }
}

pub fn structify_data(input: &CommandOption) -> Option<TokenStream> {
    let opts = input.options.as_ref()?;
    let name = mk_ident(&input.name.to_camel_case());
    let fields = opts.iter().map(|x| {
        let kind = mk_ident(x.print_kind());
        let name = mk_ident(&x.name);
        quote! {
           pub #name: #kind
        }
    });
    let deser_impl = generate_deserialize_impl(&opts);

    let mod_ident = mk_ident(&input.name.to_snake_case());
    Some(quote! {
        pub mod #mod_ident {
            use serde::de::{SeqAccess, Visitor};
            use serde::Deserializer;
            use std::fmt;
            use std::fmt::Write;
            #[derive(serde::Serialize, serde::Deserialize, Debug)]
            pub struct #name {
                pub name: String,
                pub options: Options,
            }
            #[derive(serde::Serialize, Debug, Default)]
            pub struct Options {
                #(#fields,)*
            }
            #deser_impl
        }
    })
}
pub fn extract_modules(
    schema: &CommandOption,
) -> (Vec<&CommandOption>, HashMap<&str, Vec<&CommandOption>>) {
    fn recurse<'schema>(
        next: &'schema CommandOption,
        path: &mut Vec<&'schema str>,
        root: &mut Vec<&'schema CommandOption>,
        modules: &mut HashMap<&'schema str, Vec<&'schema CommandOption>>,
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

pub fn structify(input: &str) -> TokenStream {
    let schema: CommandOption = serde_json::from_str(input).unwrap();

    let (root, modules) = extract_modules(&schema);

    let root_name_camelcase = mk_ident(&schema.name.to_camel_case());
    let root_name = mk_ident(&schema.name);
    let subcommand_struct_tokens = modules.iter().map(|(k, v)| {
        let mod_ident = mk_ident(k);
        let enum_ident = mk_ident(&k.to_camel_case());
        let fields = v.iter().flat_map(|x| structify_data(x));
        // is k necessarily snake_case?
        let enum_tokens = v.iter().map(|x| {
            let type_ident = mk_ident(&x.name);
            let type_ident_camelcase = mk_ident(&x.name.to_camel_case());
            quote! {
                #type_ident_camelcase(crate::#root_name::#mod_ident::#type_ident::Options)
            }
        });
        quote! {
            pub mod #mod_ident {
                use serde::de::{SeqAccess, Visitor};
                use serde::Deserializer;
                use std::fmt;
                use std::fmt::Write;
                #(#fields)*
                pub mod cmd {
                    #[derive(serde::Serialize, serde::Deserialize, Debug)]
                    #[serde(tag = "name", content = "options")]
                    #[serde(rename_all = "snake_case")]
                    pub enum #enum_ident {
                        #(#enum_tokens,)*
                    }
                }
            }
        }
    });
    let root_enum_tokens = root.iter().map(|x| {
        let x_ident = mk_ident(&x.name);
        let enum_ident = mk_ident(&x.name.to_camel_case());
        quote! {
            #enum_ident(crate::#root_name::#x_ident::Options)
        }
    });
    let root_module_tokens = modules.keys().map(|x| {
        let snake_case_ident = mk_ident(&x.to_snake_case());
        let camel_case_ident = mk_ident(&x.to_camel_case());
        quote! {
            #camel_case_ident(Vec<crate::#root_name::#snake_case_ident::cmd::#camel_case_ident>)
        }
    });

    let root_struct_tokens = root.iter().flat_map(|x| structify_data(x));
    quote! {
        pub mod #root_name {
            #(#root_struct_tokens)*
            pub mod cmd {
                #[derive(serde::Serialize, serde::Deserialize, Debug)]
                pub struct #root_name_camelcase {
                    id: String,
                    name: String,
                    options: Vec<Options>
                }

                #[derive(serde::Serialize, serde::Deserialize, Debug)]
                #[serde(tag = "name", content = "options")]
                #[serde(rename_all = "snake_case")]
                pub enum Options {
                    #(#root_enum_tokens,)*
                    #(#root_module_tokens,)*
                }
            }
            #(#subcommand_struct_tokens)*
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{generate_deserialize_impl, structify, structify_data, CommandOption};
    use quote::quote;
    use serde_json::json;
    use std::fmt;
    use std::io::Write;
    use std::process::{Command, Stdio};

    #[derive(PartialEq)]
    struct DisplayString(String);

    impl fmt::Debug for DisplayString {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(&self.0)
        }
    }

    fn fmt(input: &str) -> Option<String> {
        let mut proc = Command::new("rustfmt")
            .arg("--emit=stdout")
            .arg("--edition=2018")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .ok()?;
        let stdin = proc.stdin.as_mut()?;
        stdin.write_all(input.as_bytes()).ok()?;
        let output = proc.wait_with_output().ok()?;

        if output.status.success() {
            String::from_utf8(output.stdout).ok()
        } else {
            None
        }
    }

    // this is a macro to preserve line information on failure; also,
    // this should only be used on strings that contain Rust code
    macro_rules! assert_eq {
        ($a:expr, $b:expr) => {
            if let (Some(a), Some(b)) = (fmt(&$a), fmt(&$b)) {
                let a = DisplayString(a);
                let b = DisplayString(b);
                pretty_assertions::assert_eq!(a, b);
            } else {
                pretty_assertions::assert_eq!($a, $b);
            }
        };
    }

}
