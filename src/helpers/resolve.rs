use std::hash::Hash;
use indexmap::IndexMap;

pub trait Resolve<Context = ()>: Sized {
    type Result;
    type Error;
    fn resolve(&self, ctx: &mut Context) -> Result<Self::Result, Vec<Self::Error>>;

    fn map<F, R>(self, mapper: F) -> Map<Self, F>
        where F: Fn(Self::Result, &mut Context) -> R,
    { Map { resolver: self, mapper } }
}

#[derive(Debug, Clone)]
pub struct Value<T>(pub T);

impl<C, T> Resolve<C> for Value<T>
    where T: Clone
{
    type Result = T;
    type Error = ();
    #[inline]
    fn resolve(&self, _ctx: &mut C) -> Result<Self::Result, Vec<Self::Error>> {
        Ok(self.0.clone())
    }
}

#[derive(Debug, Clone)]
pub struct Map<T, F> {
    resolver: T,
    mapper: F,
}

impl<C, T, F, R> Resolve<C> for Map<T, F>
    where
        T: Resolve<C>,
        F: Fn(T::Result, &mut C) -> R,
{
    type Result = R;
    type Error = T::Error;
    fn resolve(&self, ctx: &mut C) -> Result<Self::Result, Vec<Self::Error>> {
        Ok((self.mapper)(self.resolver.resolve(ctx)?, ctx))
    }
}

impl<C, T> Resolve<C> for Vec<T>
    where T: Resolve<C>
{
    type Result = Vec<T::Result>;
    type Error = T::Error;
    fn resolve(&self, ctx: &mut C) -> Result<Self::Result, Vec<Self::Error>> {
        let mut results = Vec::with_capacity(self.len());
        let mut errors = Vec::new();
        for item in self.iter() {
            match item.resolve(ctx) {
                Ok(result) => results.push(result),
                Err(mut err) => errors.append(&mut err),
            }
        }
        if errors.is_empty() {
            Ok(results)
        } else {
            Err(errors)
        }
    }
}

impl<C, K, T> Resolve<C> for IndexMap<K, T>
    where T: Resolve<C>,
          K: Hash + Eq + Clone,
{
    type Result = IndexMap<K, T::Result>;
    type Error = T::Error;
    fn resolve(&self, ctx: &mut C) -> Result<Self::Result, Vec<Self::Error>> {
        let mut results = IndexMap::new();
        let mut errors = Vec::new();
        for (name, item) in self.iter() {
            match item.resolve(ctx) {
                Ok(result) => { results.insert(name.clone(), result); }
                Err(mut err) => errors.append(&mut err),
            }
        }
        if errors.is_empty() {
            Ok(results)
        } else {
            Err(errors)
        }
    }
}

impl<C, E, T0, T1> Resolve<C> for (T0, T1)
    where
        T0: Resolve<C, Error=E>,
        T1: Resolve<C, Error=E>,
{
    type Result = (
        T0::Result,
        T1::Result,
    );
    type Error = E;
    fn resolve(&self, ctx: &mut C) -> Result<Self::Result, Vec<Self::Error>> {
        Ok((
            self.0.resolve(ctx)?,
            self.1.resolve(ctx)?,
        ))
    }
}
