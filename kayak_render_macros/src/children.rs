use std::collections::HashSet;

use crate::{
    arc_function::build_arc_function,
    attribute::Attribute,
    child::{walk_block_to_variable, Child},
};
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream, Result};

#[derive(Debug, Clone)]
pub struct Children {
    pub nodes: Vec<Child>,
}

impl Children {
    pub fn new(nodes: Vec<Child>) -> Self {
        Children { nodes }
    }

    pub fn get_clonable_attributes(&self, index: usize) -> Vec<proc_macro2::TokenStream> {
        let mut tokens = Vec::new();

        let regular_tokens: Vec<_> = match &self.nodes[index] {
            Child::Widget(widget) => widget
                .attributes
                .attributes
                .iter()
                .filter_map(|attr| match attr {
                    Attribute::WithValue(_, block) => walk_block_to_variable(block),
                    _ => None,
                })
                .collect(),
            _ => vec![],
        };
        tokens.extend(regular_tokens);

        let children_tokens: Vec<proc_macro2::TokenStream> = match &self.nodes[index] {
            Child::Widget(widget) => (0..widget.children.nodes.len())
                .into_iter()
                .flat_map(|child_id| widget.children.get_clonable_attributes(child_id))
                .collect(),
            _ => vec![],
        };

        tokens.extend(children_tokens);

        tokens.dedup_by(|a, b| a.to_string().eq(&b.to_string()));

        tokens
    }

    pub fn as_option_of_tuples_tokens(&self) -> proc_macro2::TokenStream {
        let found_crate = proc_macro_crate::crate_name("kayak_core");
        let kayak_core = if let Ok(found_crate) = found_crate {
            match found_crate {
                proc_macro_crate::FoundCrate::Itself => quote! { crate },
                proc_macro_crate::FoundCrate::Name(name) => {
                    let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
                    quote!(#ident)
                }
            }
        } else {
            quote!(kayak_ui::core)
        };

        let children_quotes: Vec<_> = self
            .nodes
            .iter()
            .map(|child| {
                quote! { #child }
            })
            .collect();

        match children_quotes.len() {
            0 => quote! { None },
            1 => {
                let child = if children_quotes[0].to_string() == "{ }" {
                    quote! { None }
                } else {
                    let children_attributes: Vec<_> = self.get_clonable_attributes(0);

                    // I think this is correct.. It needs more testing though..
                    let clonable_children = children_attributes
                        .iter()
                        .filter(|ts| syn::parse_str::<syn::Path>(&ts.to_string()).is_ok())
                        .collect::<Vec<_>>();

                    let cloned_attrs = quote! {
                        #(let #clonable_children = #clonable_children.clone();)*;
                    };
                    if children_quotes[0].to_string() == "children" {
                        quote! {
                            #(#children_quotes)*.clone()
                        }
                    } else {
                        let children_builder = build_arc_function(
                            quote! { child_widget },
                            quote! { #(#children_quotes),* },
                            true,
                            0,
                            true,
                        );

                        quote! {
                            Some(std::sync::Arc::new(move |tree: #kayak_core::WidgetTree, parent_id: Option<#kayak_core::Index>, context: &mut #kayak_core::context::KayakContext| {
                                #cloned_attrs
                                #children_builder
                            }))
                        }
                    }
                };
                quote! {
                    #child
                }
            }
            _ => {
                // First get shared and non-shared attributes..
                let mut child_attributes_list = Vec::new();
                for i in 0..children_quotes.len() {
                    let ts_vec = self.get_clonable_attributes(i);

                    // I think this is correct.. It needs more testing though..
                    let clonable_children = ts_vec
                        .into_iter()
                        .filter(|ts| syn::parse_str::<syn::Path>(&ts.to_string()).is_ok())
                        .collect::<Vec<_>>();

                    child_attributes_list.push(clonable_children);
                }

                let mut all_attributes = HashSet::new();
                for child_attributes in child_attributes_list.iter() {
                    for child_attribute in child_attributes {
                        all_attributes.insert(child_attribute.to_string());
                    }
                }

                all_attributes.insert("children".to_string());

                let base_matching: Vec<proc_macro2::TokenStream> = all_attributes
                    .iter()
                    .map(|a| format!("base_{}", a).parse().unwrap())
                    .collect();

                let all_attributes: Vec<proc_macro2::TokenStream> =
                    all_attributes.iter().map(|a| a.parse().unwrap()).collect();

                let base_clone = quote! {
                    #(let #base_matching = #all_attributes.clone();)*
                };

                let base_clones_inner = quote! {
                    #(let #all_attributes = #base_matching.clone();)*
                };

                let mut output = Vec::new();
                output.push(quote! { #base_clone });
                for (i, children_quote) in children_quotes.iter().enumerate() {
                    output.push(quote! { #base_clones_inner });
                    let name: proc_macro2::TokenStream = format!("child{}", i).parse().unwrap();
                    let child =
                        build_arc_function(quote! { #name }, children_quote.clone(), true, i, true);
                    output.push(quote! { #child });
                }

                quote! {
                    Some(std::sync::Arc::new(move |tree: #kayak_core::WidgetTree, parent_id: Option<#kayak_core::Index>, context: &mut #kayak_core::context::KayakContext| {
                        #(#output)*
                    }))
                }
            }
        }
    }
}

impl Parse for Children {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut nodes = vec![];

        while !input.peek(syn::Token![<]) || !input.peek2(syn::Token![/]) {
            let child = input.parse::<Child>()?;
            nodes.push(child);
        }

        Ok(Self::new(nodes))
    }
}

impl ToTokens for Children {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.as_option_of_tuples_tokens().to_tokens(tokens);
    }
}
