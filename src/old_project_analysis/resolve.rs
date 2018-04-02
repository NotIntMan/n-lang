use std::hash::{
    Hash,
    BuildHasher,
};
use indexmap::IndexMap;
use super::context::SemanticContext;

pub trait SemanticResolve {
    fn is_resolved(&self, context: &SemanticContext) -> bool;
    fn try_resolve(&mut self, context: &mut SemanticContext);
}

impl<T: SemanticResolve> SemanticResolve for [T] {
    fn is_resolved(&self, context: &SemanticContext) -> bool {
        self.iter()
            .all(|item| (*item).is_resolved(context))
    }
    fn try_resolve(&mut self, context: &mut SemanticContext) {
        for item in self.iter_mut() {
            item.try_resolve(context);
        }
    }
}

impl<T: SemanticResolve> SemanticResolve for Vec<T> {
    fn is_resolved(&self, context: &SemanticContext) -> bool { self.as_slice().is_resolved(context) }
    fn try_resolve(&mut self, context: &mut SemanticContext) { self.as_mut_slice().try_resolve(context) }
}

impl<K: Hash + Eq, V: SemanticResolve, S: BuildHasher> SemanticResolve for IndexMap<K, V, S> {
    fn is_resolved(&self, context: &SemanticContext) -> bool {
        self.iter()
            .all(|(_, value)| value.is_resolved(context))
    }
    fn try_resolve(&mut self, context: &mut SemanticContext) {
        for (_, value) in self.iter_mut() {
            value.try_resolve(context);
        }
    }
}
