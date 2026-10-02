use crate::*;

// TODO: docs
pub macro pat($($rest:tt)*) {
    pat_inner!{ Alt: [ ] [ ] $($rest)* }
}

// TODO: docs
pub macro seq($($rest:tt)*) {
    pat_inner!{ Seq: [ ] [ ] $($rest)* }
}

type Thing = pat!{
    | { b'A'..=b'Z' | b'a'..=b'z' }
    | ("Hello", {","|", "|" "}, "World", "!"?)
};


pub macro tk( $lhs:literal ..= $rhs:literal ) {
    Between<
        { match stringify!($lhs).as_bytes()[..] {
            [b'\'', byt, b'\''] => byt as u8,
            [byt] => byt as u8,
            [byt, ..] => byt as u8,
            _ => panic!("Token had invalid input!")
        }},
        { match stringify!($rhs).as_bytes()[..] {
            [b'\'', byt, b'\''] => byt as u8,
            [byt] => byt as u8,
            _ => panic!("Token had invalid input!")
        }},
    >
}


// TODO: tests
macro pat_inner {
    ( $tar:ident:  [ $($acc:tt)* ] [ ] ) => { $tar<( $($acc)* )> },
    ( $tar:ident:  [ ] [ ] ) => { },
    ( $tar:ident: [ $($acc:tt)* ] [ ] $v:tt $($rest:tt)* ) => {
        pat_inner!{ $tar: [ $($acc)*] [ $v ] $($rest)* } },

    // Punctuation: `|`
    ( Alt: [ $($acc:tt)* ] [ | ] $($rest:tt)* ) => {
        pat_inner!{ Alt: [ $($acc)* ] [ ] $($rest)* } },
    ( Alt: [ $($acc:tt)* ] [ , ] $($rest:tt)* ) => {
        pat_inner!{ Seq: [ $($acc)* ] [ ] $($rest)* } },
    // Punctuation: `,`
    ( Seq: [ $($acc:tt)* ] [ , ] $($rest:tt)* ) => {
        pat_inner!{ Seq: [ $($acc)*] [ ] $($rest)* } },

    ( $tar:ident: [ $($acc:tt)* ] [ ] ( $($body:tt)* ) $($rest:tt)* ) => {
        pat_inner!{ $tar: [ $($acc)* ] [ ( $($body)* ) ] $($rest)* }
    },

    ( $tar:ident:  [ $($acc:tt)* ] [ $lhs:literal ] - $rhs:literal * $($rest:tt)* ) => {
        pat_inner!{ $tar: [ $($acc)* Option<Vec<tk![$lhs ..= $rhs]>>, ] [ ] $($rest)* } },
    ( $tar:ident:  [ $($acc:tt)* ] [ $lhs:literal ] - $rhs:literal ? $($rest:tt)* ) => {
        pat_inner!{ $tar: [ $($acc)* Option<tk![$lhs ..= $rhs]>, ] [ ] $($rest)* } },
    ( $tar:ident:  [ $($acc:tt)* ] [ $lhs:literal ] - $rhs:literal + $($rest:tt)* ) => {
        pat_inner!{ $tar: [ $($acc)* Vec<tk![$lhs ..= $rhs]>, ] [ ] $($rest)* } },
    ( $tar:ident:  [ $($acc:tt)* ] [ $lhs:literal ] - $rhs:literal $($rest:tt)* ) => {
        pat_inner!{ $tar: [ $($acc)* tk![$lhs ..= $rhs], ] [ ] $($rest)* } },

    // Between<$lhs, $rhs>
    ( $tar:ident:  [ $($acc:tt)* ] [ $lhs:literal ] ..= $rhs:literal * $($rest:tt)* ) => {
        pat_inner!{ $tar: [ $($acc)* Option<Vec<Between<$lhs, $rhs>>>, ] [ ] $($rest)* } },
    ( $tar:ident:  [ $($acc:tt)* ] [ $lhs:literal ] ..= $rhs:literal ? $($rest:tt)* ) => {
        pat_inner!{ $tar: [ $($acc)* Option<Between<$lhs, $rhs>>, ] [ ] $($rest)* } },
    ( $tar:ident:  [ $($acc:tt)* ] [ $lhs:literal ] ..= $rhs:literal + $($rest:tt)* ) => {
        pat_inner!{ $tar: [ $($acc)* Vec<Between<$lhs, $rhs>>, ] [ ] $($rest)* } },
    ( $tar:ident:  [ $($acc:tt)* ] [ $lhs:literal ] ..= $rhs:literal $($rest:tt)* ) => {
        pat_inner!{ $tar: [ $($acc)* Between<$lhs, $rhs>, ] [ ] $($rest)* } },

    // Tok<$lit>
    ( $tar:ident:  [ $($acc:tt)* ] [ $lit:literal ] * $($rest:tt)* ) => {
        pat_inner!{ $tar: [ $($acc)* Option<Vec<Tok<$lit>>>, ] [ ] $($rest)* } },
    ( $tar:ident:  [ $($acc:tt)* ] [ $lit:literal ] ? $($rest:tt)* ) => {
        pat_inner!{ $tar: [ $($acc)* Option<Tok<$lit>>, ] [ ] $($rest)* } },
    ( $tar:ident:  [ $($acc:tt)* ] [ $lit:literal ] + $($rest:tt)* ) => {
        pat_inner!{ $tar: [ $($acc)* Vec<Tok<$lit>>, ] [ ] $($rest)* } },
    ( $tar:ident:  [ $($acc:tt)* ] [ $lit:literal ]   $($rest:tt)* ) => {
        pat_inner!{ $tar: [ $($acc)* Tok<$lit>, ] [ ] $($rest)* } },


    ( $tar:ident:  [ $($acc:tt)* ] [ { $($body:tt)* } ] * $($rest:tt)* ) => {
        pat_inner!{ $tar: [ $($acc)* Option<Vec<pat_inner!{ Alt: [] [] $($body)* }>>, ] [ ] $($rest)* } },
    ( $tar:ident:  [ $($acc:tt)* ] [ { $($body:tt)* } ] ? $($rest:tt)* ) => {
        pat_inner!{ $tar: [ $($acc)* Option<pat_inner!{ Alt: [] [] $($body)* }>, ] [ ] $($rest)* } },
    ( $tar:ident:  [ $($acc:tt)* ] [ { $($body:tt)* } ] + $($rest:tt)* ) => {
        pat_inner!{ $tar: [ $($acc)* Vec<pat_inner!{ Alt: [] [] $($body)* }>, ] [ ] $($rest)* } },
    ( $tar:ident:  [ $($acc:tt)* ] [ { $($body:tt)* } ] $($rest:tt)* ) => {
        pat_inner!{ $tar: [ $($acc)* pat_inner!{ Alt: [] [] $($body)* }, ] [ ] $($rest)* } },

    ( $tar:ident:  [ $($acc:tt)* ] [ ( $($body:tt)* ) ] * $($rest:tt)* ) => {
        pat_inner!{ $tar: [ $($acc)* Option<Vec<pat_inner!{ Seq: [] [] $($body)* }>>, ] [ ] $($rest)* } },
    ( $tar:ident:  [ $($acc:tt)* ] [ ( $($body:tt)* ) ] ? $($rest:tt)* ) => {
        pat_inner!{ $tar: [ $($acc)* Option<pat_inner!{ Seq: [] [] $($body)* }>, ] [ ] $($rest)* } },
    ( $tar:ident:  [ $($acc:tt)* ] [ ( $($body:tt)* ) ] + $($rest:tt)* ) => {
        pat_inner!{ $tar: [ $($acc)* Vec<pat_inner!{ Seq: [] [] $($body)* }>, ] [ ] $($rest)* } },
    ( $tar:ident:  [ $($acc:tt)* ] [ ( $($body:tt)* ) ] $($rest:tt)* ) => {
        pat_inner!{ $tar: [ $($acc)* pat_inner!{ Seq: [] [] $($body)* }, ] [ ] $($rest)* } },



    ( $tar:ident:  [ $($acc:tt)* ] [ $typ:ty ] $($rest:tt)* ) => {
        pat_inner!{ $tar: [ $($acc)* $typ, ] [ ] $($rest)* }
    },
}


