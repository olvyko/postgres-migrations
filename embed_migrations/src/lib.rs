extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

mod embed_migrations;
mod migrations;
mod util;

use proc_macro::TokenStream;
use syn::DeriveInput;

#[proc_macro_derive(EmbedMigrations, attributes(embed_migrations_options))]
pub fn derive_embed_migrations(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as DeriveInput);
    embed_migrations::derive_embed_migrations(&item)
        .to_string()
        .parse()
        .unwrap()
}
