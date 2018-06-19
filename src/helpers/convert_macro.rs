#[macro_export]
macro_rules! convert (
    (
        ($field: expr)
        is mappable
        $($modifier: ident)*
    ) => {
        $field.map(|element| convert!(
            (element) is $($modifier)*
        ))
    };

    (
        ($field: expr)
        is iterable
        $($modifier: ident)*
    ) => {
        $field.into_iter().map(|element| convert!(
            (element) is $($modifier)*
        )).collect()
    };

    (
        ($field: expr)
        is moveable value
    ) => { $field };

    (
        ($field: expr)
        is value
    ) => { $field.into() };

    (($field: expr)) => { $field.into() };
);

#[macro_export]
macro_rules! convert_struct (
    (
        $source: expr =>
        $typename: path
        { $(
            $field: ident
            $($modifier: ident)*
        ),+ }
    ) => {{
        $typename { $(
            $field: convert!(($source.$field) $($modifier)*)
        ),+ }
    }};
);

#[macro_export]
macro_rules! derive_convert {
    (
        $source: ty =>
        $target: ident
        { $(
            $field: ident
            $($modifier: ident)*
        ),+ }
    ) => {
        impl<'token, 'source> From<$source> for $target {
            fn from(source: $source) -> Self {
                convert_struct!(source => $target { $(
                    $field $($modifier)*
                ),+ })
            }
        }
    };

    (
        $source: ty =>
        $target: ident
        { $(
            $field: ident
            $($modifier: ident)*
        ),+ ,}
    ) => {
        derive_convert!($source => $target { $(
            $field $($modifier)*
        ),+ });
    }
}
