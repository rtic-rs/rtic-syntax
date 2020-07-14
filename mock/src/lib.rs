#![deny(rust_2018_compatibility)]
#![deny(rust_2018_idioms)]
#![deny(warnings)]

#[allow(unused_extern_crates)]
extern crate proc_macro;

use proc_macro::TokenStream;
use rtic_syntax::Settings;

#[proc_macro_attribute]
pub fn app(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut settings = Settings::default();
    let mut rtic_args = vec![];
    for arg in args.to_string().split(',') {
        if arg.trim() == "parse_cores" {
            settings.parse_cores = true;
        } else if arg.trim() == "parse_binds" {
            settings.parse_binds = true;
        } else if arg.trim() == "parse_extern_interrupt" {
            settings.parse_extern_interrupt = true;
        } else if arg.trim() == "parse_schedule" {
            settings.parse_schedule = true;
        } else {
            rtic_args.push(arg.to_string());
        }
    }

    if let Err(e) = rtic_syntax::parse(rtic_args.join(", ").parse().unwrap(), input, settings) {
        e.to_compile_error().into()
    } else {
        "fn main() {}".parse().unwrap()
    }
}
