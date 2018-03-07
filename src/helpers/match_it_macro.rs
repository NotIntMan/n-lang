#[macro_export]
macro_rules! match_it {
    ($value: expr, $pattern: pat => $then: expr) => {
        match $value {
            $pattern => $then,
            other => panic!("Pattern {} do not matches this value: {:#?}", stringify!($pattern), other),
        }
    };
}