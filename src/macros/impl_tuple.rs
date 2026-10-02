pub(crate) macro tuples {
    ( $cb:ident ! { }) => { },
    ( $cb:ident ! { $z:ident $($a:tt)* }) => {
        $cb! { $z $($a)* }
        tuples! { $cb! {$($a)*} }
    },
}

pub(crate) macro impl_tuples($cb:ident) {
    tuples! { $cb! { A B C D E F G H I J K L M N O P Q R S T U V W X Y Z }}
}
