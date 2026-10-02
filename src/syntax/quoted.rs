use crate::utf8::UTF8;

use super::*;

pub type QuotedStr<
    L = Tok<"\"">,
    R = Tok<"\"">,
    D = Basic,
> = Destruct<(Excl, Just, Excl),
    (L, Pipe<UTF8<Until<R, (Option<Tok<"\\">>, Next)>>, Unescape<D>>, R)>;

pub type QuotedString<L = Tok<"\"">, R = Tok<"\"">, Domain = Basic>
    = Map<QuotedStr<L, R, Domain>, Coax<String>>;
