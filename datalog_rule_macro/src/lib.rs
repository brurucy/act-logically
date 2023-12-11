extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use std::collections::HashMap;
use syn::parse::{Parse, ParseStream};
use syn::{bracketed, parenthesized, Ident, Result, Token, Lit, LitStr};

enum TermArg {
    Variable(Ident),
    Constant(Lit),
    SkolemFunction(Ident, Vec<Ident>)
}

struct AtomArgs {
    name: Ident,
    args: Vec<TermArg>,
}

struct RuleMacroInput {
    head: AtomArgs,
    body: Vec<AtomArgs>,
}

impl Parse for RuleMacroInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let head = input.parse::<AtomArgs>()?;
        let mut distinguished_variables: HashMap<String, (&Ident, bool)> = head
            .args
            .iter()
            .filter(|term| matches!(term, TermArg::Variable(_)))
            .map(|variable| match variable {
                TermArg::Variable(ident) => {
                    (ident.to_string(), (ident, false))
                },
                _ => unreachable!(),
            })
            .collect();

        input.parse::<Token![<-]>()?;
        let content2;
        bracketed!(content2 in input);
        let body: syn::punctuated::Punctuated<AtomArgs, Token![,]> =
            content2.parse_terminated(AtomArgs::parse)?;
        let body_vec: Vec<AtomArgs> = body.into_iter().collect();
        body_vec.iter().for_each(|body_atom| {
            body_atom
                .args
                .iter()
                .filter(|term| matches!(term, TermArg::Variable(_)))
                .for_each(|variable| match variable {
                    TermArg::Variable(ident) => {
                        let owned_ident = ident.to_string();

                        if distinguished_variables.contains_key(&owned_ident) {
                            (distinguished_variables.get_mut(&owned_ident).unwrap()).1 = true;
                        }
                    }
                    _ => unreachable!(),
                });
        });

        for (key, value) in distinguished_variables {
            if !value.1 {
                return Err(syn::Error::new(
                    value.0.span(),
                    format!("variable {} not found in the body", key),
                ));
            }
        }

        Ok(RuleMacroInput {
            head,
            body: body_vec,
        })
    }
}

impl Parse for AtomArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse()?;
        let content;
        parenthesized!(content in input);

        let mut args = syn::punctuated::Punctuated::<TermArg, Token![,]>::new();

        while !content.is_empty() {
            if content.peek(Ident) && content.peek2(syn::token::Paren) {
                // Parse a SkolemFunction with its arguments
                let func_name: Ident = content.parse()?;
                let func_args_content;
                parenthesized!(func_args_content in content);
                let func_args: syn::punctuated::Punctuated<Ident, Token![,]> =
                    func_args_content.parse_terminated(|p| {
                        if p.peek(Token![?]) {
                            // Parse variables for the function
                            p.parse::<Token![?]>()?;
                            Ok(p.parse()?)
                        } else {
                            // Handle other cases or throw an error if needed
                            Err(syn::Error::new(p.span(), "Expected a variable"))
                        }
                    })?;

                args.push(TermArg::SkolemFunction(func_name, func_args.into_iter().collect()));
            } else if content.peek(Token![?]) {
                // Existing logic for variables
                content.parse::<Token![?]>()?;
                args.push(TermArg::Variable(content.parse()?));
            } else {
                // Existing logic for constants
                args.push(TermArg::Constant(content.parse()?));
            }

            // Handle the comma between arguments if there is one
            if content.peek(Token![,]) {
                content.parse::<Token![,]>()?;
            }
        }

        Ok(AtomArgs {
            name,
            args: args.into_iter().collect(),
        })
    }
}


#[proc_macro]
pub fn rule(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as RuleMacroInput);

    let head_name = &input.head.name;
    let head_terms: Vec<_> = input
        .head
        .args
        .iter()
        .map(|arg| match arg {
            TermArg::Variable(ident) => quote! { Term::Variable(stringify!(#ident).to_string()) } ,
            TermArg::Constant(lit) => quote! { Term::Constant(TypedValue::from(#lit)) },
            TermArg::SkolemFunction(ident, vars) => {
                let var_strings: Vec<_> = vars.iter().map(|var| quote! { stringify!(#var).to_string() }).collect();

                quote! { Term::Skolemizer(SkolemFunction { func: #ident, deps: vec![#(#var_strings),*]}) }
            }
        })
        .collect();

    let body_atoms: Vec<_> = input
        .body
        .iter()
        .map(|atom| {
            let name = &atom.name;
            let terms: Vec<_> = atom
                .args
                .iter()
                .map(|arg| match arg {
                    TermArg::Variable(ident) => quote! { Term::Variable(stringify!(#ident).to_string()) } ,
                    TermArg::Constant(lit) => quote! { Term::Constant(TypedValue::from(#lit)) },
                    TermArg::SkolemFunction(ident, vars) => {
                        let var_strings: Vec<_> = vars.iter().map(|var| quote! { stringify!(#var).to_string() }).collect();

                        quote! { Term::Skolemizer(SkolemFunction { func: #ident, deps: vec![#(#var_strings),*]}) }
                    }
                })
                .collect();
            quote! { Atom { terms: vec![#(#terms),*], symbol: stringify!(#name).to_string() } }
        })
        .collect();

    let expanded = quote! {
        Rule {
            head: Atom { terms: vec![#(#head_terms),*], symbol: stringify!(#head_name).to_string() },
            body: vec![#(#body_atoms),*],
            id: 0
        }
    };

    expanded.into()
}

struct ProgramMacroInput {
    rules: syn::punctuated::Punctuated<RuleMacroInput, Token![,]>,
}

impl Parse for ProgramMacroInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let rules = input.parse_terminated(RuleMacroInput::parse)?;
        Ok(ProgramMacroInput { rules })
    }
}

#[proc_macro]
pub fn program(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as ProgramMacroInput);

    let rules: Vec<_> = input.rules.into_iter().map(|rule_input| {
        let head_name = &rule_input.head.name;
        let head_terms: Vec<_> = rule_input
            .head
            .args
            .iter()
            .map(|arg| match arg {
                TermArg::Variable(ident) => quote! { Term::Variable(stringify!(#ident).to_string()) } ,
                TermArg::Constant(lit) => quote! { Term::Constant(TypedValue::from(#lit)) },
                TermArg::SkolemFunction(ident, vars) => {
                    let var_strings: Vec<_> = vars.iter().map(|var| quote! { stringify!(#var).to_string() }).collect();

                    quote! { Term::Skolemizer(SkolemFunction { func: #ident, deps: vec![#(#var_strings),*]}) }
                }
            })
            .collect();

        let body_atoms: Vec<_> = rule_input
            .body
            .iter()
            .map(|atom| {
                let name = &atom.name;
                let terms: Vec<_> = atom
                    .args
                    .iter()
                    .map(|arg| match arg {
                        TermArg::Variable(ident) => quote! { Term::Variable(stringify!(#ident).to_string()) } ,
                        TermArg::Constant(lit) => quote! { Term::Constant(TypedValue::from(#lit)) },
                        TermArg::SkolemFunction(ident, vars) => {
                            let var_strings: Vec<_> = vars.iter().map(|var| quote! { stringify!(#var).to_string() }).collect();

                            quote! { Term::Skolemizer(SkolemFunction { func: #ident, deps: vec![#(#var_strings),*]}) }
                        }
                    })
                    .collect();
                quote! { Atom { terms: vec![#(#terms),*], symbol: stringify!(#name).to_string() } }
            })
            .collect();

        quote! {
            Rule {
                head: Atom { terms: vec![#(#head_terms),*], symbol: stringify!(#head_name).to_string() },
                body: vec![#(#body_atoms),*],
                id: 0
            }
        }
    }).collect();

    let expanded = quote! {
        Program::from( vec![#(#rules),*] )
    };

    expanded.into()
}
