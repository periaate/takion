use crate::*;

/// ```
/// use takion::*;
///
/// assert_eq!(bool::parse::<()>("true".into()).ok().unwrap(), true);
/// assert_eq!(bool::parse::<()>("false".into()).ok().unwrap(), false);
/// assert!(bool::parse::<()>("nah".into()).is_miss());
/// ```
parser! {[bool] {
    ("true")  => { true };
    ("false") => { false };
}}

macro impl_num($( [$parser:ident] { $($ty:ident),* $(,)? } )* ) {$( $(
    impl Rule for $ty {
        // type This = Txt<{ stringify!($ty) }>;
        type This = ($parser,);
    }

    impl<'a> Parse<'a, u8> for $ty {
        type Item = $ty;

        #[inline]
        fn parse<Cx: Ctx>(cursor: Cursor<'a, u8>) -> Ret<'a, u8, Self::Item, Cx> {
            let (new, val) = <UTF8<Span<$parser>>>::parse::<Cx>(cursor)
            .map_err(|e| Cx::Info::new::<$ty>(cursor, e.end().unwrap_or(cursor.index)))??;
            let Ok(res) = val.parse::<$ty>() else {
                return Miss(Cx::Info::new::<Self>(cursor, new.index))
            };
            Pass(new, res)
        }
    }
)* )*}

impl_num!{
    [Nat]   { u8, u16, u32, u64, u128, usize, }
    [Int]   { i8, i16, i32, i64, i128, isize, }
    [Float] {          f32, f64,              }
}


type Float = (Option<Tok<"-">>, Alt<(
    Tok<"Infinity">,
    Tok<"NaN">,
    (Nat, Option<(Tok<".">, Nat)>),
)>);

type Int = (Option<Tok<"-">>, Nat);
type Nat = Slice<Digit>;

#[test]
fn integers() {
    use std::assert_matches;
    type U64 = UseAll<u64>;
    type I64 = UseAll<i64>;
    type F64 = UseAll<f64>;

    for (byte, numb) in (b'0'..b'9').zip(0u64..=9u64) {
        assert_eq!(U64::parse::<()>([byte].as_slice().into()).ok(), Ok(numb));
    }
    assert_matches!(U64::parse::<()>("".into()).ok(), Err(_));

    for (byte, numb) in (b'0'..b'9').zip(0i64..=9i64) {
        assert_eq!(I64::parse::<()>([byte].as_slice().into()).ok(), Ok(numb));
        assert_eq!(I64::parse::<()>([b'-', byte].as_slice().into()).ok(), Ok(-numb));
    }
    assert_matches!(I64::parse::<()>("".into()).ok(), Err(_));


    let cases = (-100..=100)
        .flat_map(|lhs| (0..=333).map(move |rhs| format!("{lhs}.{rhs}")))
        .map(|f| (f.clone(), f.parse::<f64>().unwrap()))
        .collect::<Vec<_>>();

    for (s, v) in cases.iter().map(|(s, v)| (s.as_str(), v)) {
        assert_eq!(F64::parse::<()>(s.into()).ok(), Ok(v).copied());
    }

    assert!(f64::is_nan(F64::parse::<()>("NaN".into()).ok().unwrap()));
    assert!(f64::is_infinite(F64::parse::<()>("Infinity".into()).ok().unwrap()));
    assert!(f64::is_infinite(F64::parse::<()>("-Infinity".into()).ok().unwrap()));
    assert!(F64::parse::<()>(".".into()).is_miss());
}
