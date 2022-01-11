use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream, Result};

use crate::widget::Widget;

#[derive(Debug, Clone)]
pub enum Child {
    Widget(Widget),
    RawBlock(syn::Block),
}

impl ToTokens for Child {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Widget(widget) => widget.to_tokens(tokens),
            Self::RawBlock(block) => {
                let ts = if block.stmts.len() == 1 {
                    let first = &block.stmts[0];
                    quote!(#first)
                } else {
                    quote!(#block)
                };
                ts.to_tokens(tokens);
            }
        }
    }
}

impl Parse for Child {
    fn parse(input: ParseStream) -> Result<Self> {
        match Widget::custom_parse(input, true, true) {
            Ok(widget) => Ok(Self::Widget(widget)),
            Err(_) => {
                let block = input.parse::<syn::Block>()?;
                Ok(Self::RawBlock(block))
            }
        }
    }
}

pub fn walk_block_to_variable(block: &syn::Block) -> Option<proc_macro2::TokenStream> {
    block.stmts.first().and_then(walk_statement)
}

pub fn walk_statement(statement: &syn::Stmt) -> Option<proc_macro2::TokenStream> {
    match statement {
        syn::Stmt::Expr(expr) => match expr {
            syn::Expr::Call(call) => Some(call.args.to_token_stream()),
            syn::Expr::Path(path) => Some(path.to_token_stream()),
            _ => None,
        },
        _ => None,
    }
}
