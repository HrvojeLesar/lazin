use syn::{
    Ident, Lit, LitStr, Path, Token, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

mod kw {
    syn::custom_keyword!(before);
    syn::custom_keyword!(after);
}

#[derive(Default)]
pub(crate) struct LazinTestArgs {
    pub before: Vec<FnCall>,
    pub after: Vec<FnCall>,
}

impl LazinTestArgs {
    fn new(before: Option<Vec<FnCall>>, after: Option<Vec<FnCall>>) -> Self {
        Self {
            before: before.unwrap_or_default(),
            after: after.unwrap_or_default(),
        }
    }

    fn try_from_before_fns(input: ParseStream) -> syn::Result<Self> {
        let before_fns = parse_fns(input)?.into_iter().collect();

        Ok(Self::new(Some(before_fns), None))
    }
}

impl Parse for LazinTestArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(LazinTestArgs::default());
        }

        if !input.peek(kw::before) && !input.peek(kw::after) {
            return LazinTestArgs::try_from_before_fns(input);
        }

        let mut before = Vec::new();
        let mut after = Vec::new();

        while !input.is_empty() {
            let lookahead = input.lookahead1();

            let target = if lookahead.peek(kw::before) {
                input.parse::<kw::before>()?;
                &mut before
            } else if lookahead.peek(kw::after) {
                input.parse::<kw::after>()?;
                &mut after
            } else {
                return Err(lookahead.error());
            };

            input.parse::<Token![:]>()?;

            let content;
            parenthesized!(content in input);
            target.extend(parse_fns(&content)?);

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(LazinTestArgs { before, after })
    }
}

fn parse_fns(input: ParseStream) -> syn::Result<Punctuated<FnCall, Token![,]>> {
    Punctuated::parse_terminated(input)
}

pub enum CallArg {
    Ident { mutability: bool, ident: Ident },
    Literal(Lit),
    Some(Lit),
    None,
}

impl Parse for CallArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(LitStr) {
            let lit = input.parse()?;
            return Ok(CallArg::Literal(lit));
        }

        if input.peek(Ident) {
            let fork = input.fork();
            let ident: Ident = fork.parse()?;

            if ident == "Some" && fork.peek(syn::token::Paren) {
                input.parse::<Ident>()?; // consume "Some"
                let content;
                parenthesized!(content in input);
                let inner = content.parse()?;
                return Ok(CallArg::Some(inner));
            }

            if ident == "None" {
                input.parse::<Ident>()?;
                return Ok(CallArg::None);
            }
        }

        let mutability = input.parse::<Token![mut]>().is_ok();
        let ident: Ident = input.parse()?;
        Ok(CallArg::Ident { mutability, ident })
    }
}

pub struct FnCall {
    pub path: Path,
    pub args: Vec<CallArg>,
}

impl Parse for FnCall {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path: Path = input.parse()?;

        let mut args = Vec::new();
        if input.peek(syn::token::Paren) {
            let content;
            parenthesized!(content in input);
            let idents: Punctuated<CallArg, Token![,]> = Punctuated::parse_terminated(&content)?;
            args.extend(idents);
        }

        Ok(FnCall { path, args })
    }
}
