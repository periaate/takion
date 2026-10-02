use std::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};

use super::*;

#[test]
fn ranges_enum() {
    type Ranger = UseAll<RangesP<usize, Tok<"..">, Tok<"..=">>>;
    assert_eq!( Ranger::parse::<Traced>("0..2".into()).must(),
                             RangesEnum::Both(0..2usize) );
    assert_eq!( Ranger::parse::<Traced>("0..=2".into()).must(),
                             RangesEnum::Span(0..=2usize) );
    assert_eq!( Ranger::parse::<Traced>("..=2".into()).must(),
                             RangesEnum::Upto(..=2usize) );
    assert_eq!( Ranger::parse::<Traced>("..2".into()).must(),
                             RangesEnum::To  (..2usize) );
    assert_eq!( Ranger::parse::<Traced>("2..".into()).must(),
                             RangesEnum::From(2usize..) );
    assert_eq!( Ranger::parse::<Traced>("..".into()).must(),
                             RangesEnum::Full(..) );
}

impl<Idx> RangesEnum<Idx> {
    pub fn may_enumerate(self) -> Option<Vec<Idx>>
    where
        RangeInclusive<Idx>: Iterator<Item = Idx>, 
        Range<Idx>: Iterator<Item = Idx>,
    {
        match self {
            RangesEnum::Span(range) => Some(range.collect()),
            RangesEnum::Both(range) => Some(range.collect()),
            _ => None,
        }
    }
}



/// Parses index-range syntax like `1..5`, `1..`, or `..5`.
/// ```ignore
/// use takion::*;
///
/// type Idx = Ranges;
/// assert!(Idx::parse::<()>("1..5".into()).is_pass());
/// assert!(Idx::parse::<()>("1..".into()).is_pass());
/// assert!(Idx::parse::<()>("..5".into()).is_pass());
/// assert!(Idx::parse::<()>("nope".into()).is_miss());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ranges<Idx, const SEP: &'static str = ".."> {
    From(Idx     ),
    Upto(     Idx),
    Span(Idx, Idx),
    Just(Idx     ),
}


macro impl_ranges($( $name:ident<$Idx:ident, $Sep:ident> -> $targ:ty [$input:pat = $rule:ty] $body:tt ;)*) {$(
    struct $name<$Idx, $Sep>(pub $Idx, pub $Sep);
    impl<'a, $Idx: Rule, $Sep: Rule> Rule for $name<$Idx, $Sep> { type This = ($Idx, $Sep); }
    impl<'a, T, $Idx: Parse<'a, T>, $Sep: Parse<'a, T>> Parse<'a, T> for $name<$Idx, $Sep> {
        type Item = $targ;
        fn parse<Cx: Ctx>(cursor: Cur<'a, T>) -> Ret<'a, T, Self::Item, Cx> {
            let (new, $input) = <$rule>::parse::<Cx>(cursor)
                .map_err(|e| {
                    let end = e.end().unwrap_or(cursor.index);
                    e.wrap::<Self>(cursor.index, end)
                })??;
            Ret::Pass(new, $body)
        }
    }
)*}

struct RangesFull<Sep>(pub Sep);
impl<Sep: Rule> Rule for RangesFull<Sep> { type This = (Txt<"RangeFull<">, Sep, Txt<">">); }
impl<'a, T, Sep: Parse<'a, T>> Parse<'a, T> for RangesFull<Sep> {
    type Item = RangeFull;
    fn parse<Cx: Ctx>(cursor: Cur<'a, T>) -> Ret<'a, T, Self::Item, Cx> {
        let (new, _) = <Sep>::parse::<Cx>(cursor)
            .map_err(|e| {
                let end = e.end().unwrap_or(cursor.index);
                e.wrap::<Self>(cursor.index, end)
            })??;
        Ret::Pass(new, (..))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RangesEnum<Idx> {
    From(RangeFrom<Idx>),
    To  (RangeTo<Idx>),
    Upto(RangeToInclusive<Idx>),
    Both(Range<Idx>),
    Span(RangeInclusive<Idx>),
    Full(RangeFull),
}

pub struct RangesP<Idx, Sep, Inc = Sep>(pub Idx, pub Sep, pub Inc);

impl<Idx, Sep, Inc> Rule for RangesP<Idx, Sep, Inc>
where Idx: Rule, Sep: Rule, Inc: Rule
    { type This = (Txt<"Range<">, Idx, Sep, Inc, Txt<">">); }

impl<'a, T, Idx, Sep, Inc> Parse<'a, T> for RangesP<Idx, Sep, Inc>
where Idx: Parse<'a, T>, Sep: Parse<'a, T>, Inc: Parse<'a, T>
{
    type Item = RangesEnum<Idx::Item>;

    fn parse<Cx: Ctx>(cursor: Cur<'a, T>) -> Ret<'a, T, Self::Item, Cx> {
        parse!{ [Cx] [cursor] {
            (range: RangesUpto<Idx, Inc>) => RangesEnum::Upto(range);
            (range: RangesSpan<Idx, Inc>) => RangesEnum::Span(range);
            (range: RangesBoth<Idx, Sep>) => RangesEnum::Both(range);
            (range: RangesFrom<Idx, Sep>) => RangesEnum::From(range);
            (range: RangesTo  <Idx, Sep>) => RangesEnum::To  (range);
            (range: RangesFull<Sep     >) => RangesEnum::Full(range);
        }}

        Ret::Miss(Cx::Info::new::<Self>(cursor, cursor.index))
    }
}

impl_ranges!{
    RangesFrom<Idx, Sep> -> RangeFrom<Idx::Item> [(from, _) = (Idx, Sep)]
        (from..);

    RangesBoth<Idx, Sep> -> Range<Idx::Item> [(from, _, upto) = (Idx, Sep, Idx)]
        (from..upto);
    RangesSpan<Idx, Sep> -> RangeInclusive<Idx::Item> [(from, _, upto) = (Idx, Sep, Idx)]
        (from..=upto);

    RangesUpto<Idx, Sep> -> RangeToInclusive<Idx::Item> [(_, val) = (Sep, Idx)]
        (..=val);
    RangesTo<Idx, Sep> -> RangeTo<Idx::Item> [(_, val) = (Sep, Idx)]
        (..val);
}

// struct RangesFrom<Idx, Sep>(pub Idx, pub Sep);
// impl<'a, Idx: Rule, Sep: Rule> Rule for RangesFrom<Idx, Sep> { type This = (Idx, Sep); }
// impl<'a, Idx: Parse<'a, u8>, Sep: Parse<'a, u8>> Parse<'a, u8>
//
// // parser! {('a, Idx: Parse<'a, u8>, Sep: Parse<'a, u8>)
// //     [RangesFrom<'a, u8>] -> [Ranges<Idx::Item>] {
// //
// // }}

impl<Idx, const SEP: &'static str> From<Range<Idx>> for Ranges<Idx, SEP> {
    fn from(value: Range<Idx>) -> Self { Self::Span(value.start, value.end) }
}

impl<Idx, const SEP: &'static str> From<RangeInclusive<Idx>> for Ranges<Idx, SEP>
where Idx: Clone {
    fn from(value: RangeInclusive<Idx>) -> Self {
        Self::Span(value.start().clone(), value.end().clone())
    }
}

impl<Idx, const SEP: &'static str> From<RangeFrom<Idx>> for Ranges<Idx, SEP> {
    fn from(value: RangeFrom<Idx>) -> Self { Self::From(value.start) }
}

impl<Idx, const SEP: &'static str> From<RangeTo<Idx>> for Ranges<Idx, SEP> {
    fn from(value: RangeTo<Idx>) -> Self { Self::Upto(value.end) }
}

impl<Idx, const SEP: &'static str> From<RangeToInclusive<Idx>> for Ranges<Idx, SEP> {
    fn from(value: RangeToInclusive<Idx>) -> Self { Self::Upto(value.end) }
}



impl<Idx: Rule, const SEP: &'static str> Rule for Ranges<Idx, SEP> {
    type This = Join<(Txt<"${">, Join<(
        (Idx, Tok<SEP>, Idx),
        (Idx, Tok<SEP>     ),
        (     Tok<SEP>, Idx),
    ), Txt<" | ">>, Txt<"}">), Txt<" ">>;
}

impl<'a, Idx: Parse<'a, u8>, const SEP: &'static str> Parse<'a, u8> for Ranges<Idx, SEP> {
    const NULLABLE: bool = { Idx::NULLABLE || Tok::<SEP>::NULLABLE };

    type Item = Ranges<Idx::Item>;

    #[inline]
    fn parse<Cx: Ctx>(cursor: Cursor<'a, u8>) -> Ret<'a, u8, Self::Item, Cx> {
        if let SubRet::Pass(cursor, (l, _, r)) = <(Idx, Tok<{SEP}>, Idx)>::parse::<Cx>(cursor)? {
            return Pass(cursor, Ranges::Span(l, r))
        }

        if let SubRet::Pass(cursor, (l, _)) = <(Idx, Tok<{SEP}>)>::parse::<Cx>(cursor)? {
            return Pass(cursor, Ranges::From(l))
        }

        if let SubRet::Pass(cursor, (_, r)) = <(Tok<{SEP}>, Idx)>::parse::<Cx>(cursor)? {
            return Pass(cursor, Ranges::Upto(r))
        }

        Miss(Cx::Info::new::<Self>(cursor, cursor.index))
    }

}
