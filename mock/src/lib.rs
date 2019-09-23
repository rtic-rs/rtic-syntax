#![deny(rust_2018_compatibility)]
#![deny(rust_2018_idioms)]
#![deny(warnings)]

extern crate proc_macro;

use proc_macro::TokenStream;
use rtfm_syntax::Settings;

#[proc_macro_attribute]
pub fn app(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut settings = Settings::default();
    let mut rtfm_args = vec![];
    for arg in args.to_string().split(',') {
        if arg.trim() == "parse_cores" {
            settings.parse_cores = true;
        } else if arg.trim() == "parse_binds" {
            settings.parse_binds = true;
        } else if arg.trim() == "parse_extern_interrupt" {
            settings.parse_extern_interrupt = true;
        } else if arg.trim() == "parse_schedule" {
            settings.parse_schedule = true;
        } else if arg.trim() == "parse_impl_generator" {
            settings.parse_impl_generator = true;
        } else {
            rtfm_args.push(arg.to_string());
        }
    }

    if let Err(e) = rtfm_syntax::parse(rtfm_args.join(", ").parse().unwrap(), input, settings) {
        e.to_compile_error().into()
    } else {
        "fn main() {}".parse().unwrap()
    }
}
