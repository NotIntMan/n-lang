use std::mem::replace;

pub trait Lazy {
    type Item;
    fn get(&self) -> Option<&Self::Item>;
    fn get_mut(&mut self) -> Option<&mut Self::Item>;
    fn set(&mut self, value: Self::Item);

    fn init_if_not<'a>(&'a mut self, init: impl FnOnce() -> Self::Item) -> &'a mut Self::Item {
        if self.get().is_none() {
            self.set(init());
        }
        self.get_mut()
            .expect("Lazy-value cannot be uninitialized right after initialize")
    }

    fn init_as_part<'a, C>(
        _ctx: &'a mut C,
        _get_data: impl Fn(&'a mut C) -> &'a mut Self,
        _init: impl FnOnce(&'a mut C) -> Self::Item,
    ) -> &'a mut Self::Item
        where Self: 'a
    {
//        let is_empty = {
//            let data = get_data(ctx);
//            let is_empty = data.get().is_none();
//            drop(data);
//            is_empty
//        };
//        if is_empty {
//            let new_value = init(ctx);
//            get_data(ctx).set(new_value);
//        }
//        get_data(ctx).get_mut()
//            .expect("Lazy-value cannot be uninitialized right after initialize")
        unimplemented!()
    }
}

impl<T> Lazy for Option<T> {
    type Item = T;
    #[inline]
    fn get(&self) -> Option<&Self::Item> {
        match self {
            Some(item) => Some(item),
            None => None,
        }
    }
    #[inline]
    fn get_mut(&mut self) -> Option<&mut Self::Item> {
        match self {
            Some(item) => Some(item),
            None => None,
        }
    }
    #[inline]
    fn set(&mut self, value: Self::Item) {
        replace(self, Some(value));
    }
}

#[cfg(test)]
struct SuperLazy {
    a: Option<u32>,
    b: Option<u32>,
}

#[cfg(test)]
impl SuperLazy {
    #[inline]
    fn new() -> Self {
        SuperLazy {
            a: None,
            b: None,
        }
    }
    #[inline]
    fn get_a(&mut self) -> &mut u32 {
        self.a.init_if_not(|| 0)
    }
    #[inline]
    fn get_b(&mut self) -> &mut u32 {
//        self.b.init_if_not(|| self.get_a().clone())
        unimplemented!()
    }
}

#[test]
fn a() {
    let mut s = SuperLazy::new();
    *s.get_a() += 20;
    assert_eq!(*s.get_b(), 20);
}