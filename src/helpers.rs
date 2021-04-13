use proc_macro2::{Ident, Span};
use unicode_xid::UnicodeXID;

pub fn is_ident_start(c: char) -> bool {
    ('a' <= c && c <= 'z')
        || ('A' <= c && c <= 'Z')
        || c == '_'
        || (c > '\x7f' && UnicodeXID::is_xid_start(c))
}

pub fn is_ident_continue(c: char) -> bool {
    ('a' <= c && c <= 'z')
        || ('A' <= c && c <= 'Z')
        || c == '_'
        || ('0' <= c && c <= '9')
        || (c > '\x7f' && UnicodeXID::is_xid_continue(c))
}

pub fn fix_ident(ident: &str) -> String {
    let mut new = String::new();
    let mut chars = ident.chars();
    let first = chars.next().unwrap();
    if !is_ident_start(first) {
        new.push('_');
    } else {
        new.push(first);
    }
    for ch in chars {
        if !is_ident_continue(ch) {
            new.push('_');
        } else {
            new.push(ch);
        }
    }
    new
}

pub fn create_ident(string: &str) -> Ident {
    Ident::new(
        &fix_ident(string).trim_start_matches('_'),
        Span::call_site(),
    )
}

// pub fn create_ident_untrimmed(string: &str) -> Ident {
//     Ident::new(&fix_ident(string), Span::call_site())
// }
