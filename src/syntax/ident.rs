use crate::utf8::UTF8;

use super::*;

/// rust style ident parsing.
pub type Ident = AsRule<
    UTF8<Span<(Alt<(Tok<"_">, Letter)>, Option<Slice<Alt<(Tok<"_">, Letter, Digit)>>>)>>,
    Txt<"ident">,
>;

#[test]
fn ident() {
    println!("{}", <Ident as Rule>::Fmt::realize());
    Ident::parse::<Normal>("_Hello_48".into())
        .expect("Ident must not error while matching \"_Hello_48\"")
        .expect("Ident must match \"_Hello_48\"");

    assert!(Ident::parse::<Normal>("8_Hello_48".into()).is_miss(),
        "Ident must not match \"8_Hello_48\"");
}
