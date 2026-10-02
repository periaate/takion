use super::*;

pub mod tokens;
pub use tokens::*;

pub mod whitespace;
pub use whitespace::*;

pub mod unescape;
pub use unescape::*;

pub mod utf8;
pub use utf8::*;

#[test]
fn hex() {
    let input = (b'a'..=b'f').chain(b'0'..=b'9').collect::<Vec<_>>();
    assert!(<UseAll<Slice<Hex<{ Case::Lower }>>>>::parse::<Normal>(input.as_slice().into()).is_pass());

    let input = (b'A'..=b'F').chain(b'0'..=b'9').collect::<Vec<_>>();
    assert!(<UseAll<Slice<Hex<{ Case::Upper }>>>>::parse::<Normal>(input.as_slice().into()).is_pass());

    let input = (b'A'..=b'F').chain(b'a'..=b'f').chain(b'0'..=b'9').collect::<Vec<_>>();
    assert!(<UseAll<Slice<Hex<{ Case::Either }>>>>::parse::<Normal>(input.as_slice().into()).is_pass());
}

#[derive(Clone, Copy, PartialEq, Eq, ConstParamTy)]
pub enum Case { Either, Upper, Lower }

/// ```
/// use takion::*;
///
/// assert!(<Hex>::parse::<()>("f".into()).is_pass()); // default case is lower
/// assert!(<Hex>::parse::<()>("F".into()).is_miss());
/// assert!(Hex::<{ Case::Upper }>::parse::<()>("F".into()).is_pass());
/// assert!(Hex::<{ Case::Either }>::parse::<()>("F".into()).is_pass());
/// ```
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Hex<const CASE: Case = { Case::Lower }>;

parser!{[Hex<{ Case::Lower }>] -> [u8] {
    (v: Between<b'a', b'f'>) => { v };
    (v: Between<b'0', b'9'>) => { v };
}}

parser!{[Hex<{ Case::Upper }>] -> [u8] {
    (v: Between<b'A', b'F'>)  => { v };
    (v: Between<b'0', b'9'>) => { v };
}}

parser!{[Hex<{ Case::Either }>] -> [u8] {
    (v: Between<b'A', b'F'>)  => { v };
    (v: Between<b'a', b'f'>) => { v };
    (v: Between<b'0', b'9'>) => { v };
}}
