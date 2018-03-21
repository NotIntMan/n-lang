#[macro_use]
extern crate pretty_assertions;
//extern crate supercow;

use std::borrow::Cow;
//use supercow::InlineNonSyncSupercow;

//type StringCow<'a> = InlineNonSyncSupercow<'a, String, str>;
type StringCow<'a> = Cow<'a, str>;

#[test]
fn a() {
    let unknown: StringCow = Cow::Borrowed("My name is ");
    let mut known: StringCow = unknown.clone();
    (*known.to_mut()).push_str("John");
    assert_eq!(&*unknown, "My name is ");
    assert_eq!(&*known, "My name is John");
}

#[test]
fn b() {
    let one: Cow<[u8]> = Cow::Owned(vec![1, 2, 3]);
    let two = Cow::Borrowed(&*one);
    assert_eq!(&*one as *const [u8], &*two as *const [u8]);
}

//fn to_static<T: ToOwned>(data: Cow<[Cow<T>]>) -> Cow<'static, [Cow<'static, T>]> {
//    let vec = data.into_owned().into_iter()
//        .map(|item| Cow::Owned(item.into_owned()))
//        .collect();
//    Cow::Owned(vec)
//}
