//#[allow(unused_macros)]
//macro_rules! universal_assert {
//    ($name: ident = $value: expr => {}) => {};
//    ($name: ident = $value: expr => { $prop: expr => $assertion: expr }) => {{
//        let $name = &$value;
//        universal_assert!($prop => $assertion);
//    }};
//    ($name: ident = $value: expr => { $prop: expr => $assertion: expr, $($rest: tt),* }) => {{
//        let $name = &$value;
//        universal_assert!($prop => $assertion);
//        universal_assert!(*$name => { $($rest),* });
//    }};
//    ($value: expr => $assertion: expr) => {{
//        assert_eq!($value, $assertion);
//    }};
//}
//
//#[test]
//fn a() {
//    universal_assert!(vector = vec![1, 2, 3] => {
//        vector => [1, 2, 3],
//        vector.len() => 4
//    });
//}
