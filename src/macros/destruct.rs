use crate::*;

pub macro destruct( $args:tt = $($rest:tt)* ) {
    Destruct<destruct_inner!($args), $($rest)*>
}

macro destruct_inner( ( $($pat:tt),* $(,)? ) ) {
    destruct_shape!( [] [] $($pat)* )
}

macro destruct_shape {
    ( [$($marks:tt)*] [$($shape:tt)*] ) => {
        destruct_finish!( [$($marks)*] $($shape)* )
    },
    ( [$($marks:tt)*] [$($shape:tt)*] _ $($rest:tt)* ) => {
        destruct_shape!( [$($marks)*] [$($shape)* Excl,] $($rest)* )
    },
    ( [$($marks:tt)*] [$($shape:tt)*] $bound:tt $($rest:tt)* ) => {
        destruct_shape!( [$($marks)* .] [$($shape)* ?,] $($rest)* )
    },
}

macro destruct_finish {
    ( [] $($shape:tt)* ) => {
        ( $($shape)* )
    },
    ( [.] $($shape:tt)* ) => {
        destruct_replace!( [Just] () $($shape)* )
    },
    ( [. . $($more:tt)*] $($shape:tt)* ) => {
        destruct_replace!( [Incl] () $($shape)* )
    },
}

macro destruct_replace {
    ( [$word:ident] ($($done:tt)*) ) => {
        ( $($done)* )
    },
    ( [$word:ident] ($($done:tt)*) ?, $($rest:tt)* ) => {
        destruct_replace!( [$word] ($($done)* $word,) $($rest)* )
    },
    ( [$word:ident] ($($done:tt)*) $keep:tt, $($rest:tt)* ) => {
        destruct_replace!( [$word] ($($done)* $keep,) $($rest)* )
    },
}

#[test]
fn destructure_macro() {
    macro assert_shape($pat:tt => $expect:tt) {
        const _: () = {
            trait Eq2<T> {}
            impl<T> Eq2<T> for T {}
            fn check<A: Eq2<B>, B>() {}
            fn go() { check::<destruct_inner!($pat), $expect>(); }
        };
    }

    assert_shape!((J)          => (Just,));
    assert_shape!((_, _)       => (Excl, Excl));
    assert_shape!((_, _, _)    => (Excl, Excl, Excl));
    assert_shape!((J, _)       => (Just, Excl));
    assert_shape!((J, _, _)    => (Just, Excl, Excl));
    assert_shape!((a, _, b)    => (Incl, Excl, Incl));
    assert_shape!((_, J)       => (Excl, Just));
    assert_shape!((_, J, _)    => (Excl, Just, Excl));
    assert_shape!((_, _, J)    => (Excl, Excl, Just));
    assert_shape!((_, _, J, _) => (Excl, Excl, Just, Excl));
}
