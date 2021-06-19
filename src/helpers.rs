use proc_macro2::{Ident, Span};
use unicode_xid::UnicodeXID;

pub fn is_ident_start(c: char) -> bool {
    ('a'..='z').contains(&c)
        || ('A'..='Z').contains(&c)
        || c == '_'
        || (c > '\x7f' && UnicodeXID::is_xid_start(c))
}

pub fn is_ident_continue(c: char) -> bool {
    ('a'..='z').contains(&c)
        || ('A'..='Z').contains(&c)
        || c == '_'
        || ('0'..='9').contains(&c)
        || (c > '\x7f' && UnicodeXID::is_xid_continue(c))
}

/// strict and reserved rust keywords:
/// https://doc.rust-lang.org/reference/keywords.html
const RESTRICTED_KEYWORDS: &[&str] = &[
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for",
    "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return",
    "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where",
    "while", "async", "await", "dyn", "abstract", "become", "box", "do", "final", "macro",
    "override", "priv", "typeof", "unsized", "virtual", "yield", "try", "_"
];

pub fn fix_ident(ident: &str) -> String {
    let mut new = String::new();
    let mut chars = ident.chars();
    let first = chars.next().unwrap();
    if !is_ident_start(first) {
        new.push('_');
        if is_ident_continue(first) {
            new.push(first);
        }
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
    if RESTRICTED_KEYWORDS.contains(&new.as_str()) {
        new.insert(0, '_');
    }
    new
}

pub fn create_ident(string: &str) -> Ident {
    Ident::new(&fix_ident(string), Span::call_site())
}

pub fn create_ident_trimmed(string: &str) -> Ident {
    create_ident(string.trim_start_matches('_'))
}
