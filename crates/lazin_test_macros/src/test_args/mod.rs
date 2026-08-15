use syn::{
    Path, Token, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

mod kw {
    syn::custom_keyword!(before);
    syn::custom_keyword!(after);
}

#[derive(Default)]
pub(crate) struct LazinTestArgs {
    pub before: Vec<Path>,
    pub after: Vec<Path>,
}

impl LazinTestArgs {
    fn new(before: Option<Vec<Path>>, after: Option<Vec<Path>>) -> Self {
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

fn parse_fns(input: ParseStream) -> syn::Result<Punctuated<Path, Token![,]>> {
    Punctuated::parse_terminated(input)
}
