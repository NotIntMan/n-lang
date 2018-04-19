//use std::sync::Arc;
//use std::collections::HashMap;
//use helpers::into_static::IntoStatic;
//use parser_basics::StaticIdentifier;
//use super::project::ModulePathSlice;
//
//#[derive(Debug, Clone, PartialEq, Eq, Hash)]
//pub struct Text {
//    pub name: String,
//    pub text: String,
//}
//
//impl Text {
//    pub fn new<A: ToString, B: ToString>(name: A, text: B) -> Self {
//        Text { name: name.to_string(), text: text.to_string() }
//    }
//}
//
//pub trait TextSource {
//    fn get_text(&self, path: &ModulePathSlice) -> Option<Arc<Text>>;
//}
//
//#[derive(Clone)]
//pub struct HashMapSource {
//    map: HashMap<Vec<StaticIdentifier>, Arc<Text>>,
//}
//
//impl HashMapSource {
//    pub fn new() -> Self {
//        HashMapSource {
//            map: HashMap::new(),
//        }
//    }
//    pub fn simple_insert(&mut self, path: Vec<&str>, name: &str, text: &str) {
//        self.map.insert(
//            path.into_iter()
//                .map(|name|
//                    StaticIdentifier::new(name).into_static()
//                )
//                .collect(),
//            Arc::new(Text {
//                name: name.to_string(),
//                text: text.to_string(),
//            }),
//        );
//    }
//}
//
//impl TextSource for HashMapSource {
//    fn get_text(&self, path: &ModulePathSlice) -> Option<Arc<Text>> {
//        let result = self.map.get(path).map(Clone::clone);
//        println!("For {:?} got {:?}", path, result);
//        result
//    }
//}
