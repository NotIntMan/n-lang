use lexeme_scanner::ItemPosition;

pub trait SourceStorage<Element: ? Sized, Index = usize> {
    fn get_element(&self, index: Index) -> Option<&Element>;
}

pub trait Storage<Element, Index = usize>: SourceStorage<Element, Index> {
    fn store_element(&mut self, element: Element) -> Index;
}

impl<T> SourceStorage<T> for Vec<T> {
    fn get_element(&self, index: usize) -> Option<&T> {
        self.get(index)
    }
}

impl<T> Storage<T> for Vec<T> {
    fn store_element(&mut self, element: T) -> usize {
        let result = self.len();
        self.push(element);
        result
    }
}

impl<T> SourceStorage<T, (usize, usize)> for Vec<Vec<T>>
{
    fn get_element(&self, index: (usize, usize)) -> Option<&T> {
        let (index, sub_index) = index;
        let sub_store: &Vec<T> = self.get_element(index)?;
        sub_store.get_element(sub_index)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TextIndex<Index> {
    index: Index,
    position: ItemPosition,
}

impl<Index> TextIndex<Index> {
    fn new(index: Index, position: ItemPosition) -> Self {
        TextIndex { index, position }
    }
}

impl<Index, Store> SourceStorage<str, TextIndex<Index>> for Store
    where Store: SourceStorage<String, Index> {
    fn get_element(&self, index: TextIndex<Index>) -> Option<&str> {
        let TextIndex { index, position } = index;
        let string: &String = self.get_element(index)?;
        Some(&string[position])
    }
}

#[test]
fn matrix_store_returns_elements_correctly() {
    let store = vec![
        vec![1],
        vec![0, 1],
        vec![0, 0, 1],
    ];
    assert_eq!(store.get_element((0, 0)), Some(&1));
    assert_eq!(store.get_element((0, 1)), None);
    assert_eq!(store.get_element((0, 2)), None);
    assert_eq!(store.get_element((1, 0)), Some(&0));
    assert_eq!(store.get_element((1, 1)), Some(&1));
    assert_eq!(store.get_element((1, 2)), None);
    assert_eq!(store.get_element((2, 0)), Some(&0));
    assert_eq!(store.get_element((2, 1)), Some(&0));
    assert_eq!(store.get_element((2, 2)), Some(&1));
    assert_eq!(store.get_element((1, 3)), None);
    assert_eq!(store.get_element((3, 0)), None);
}

#[test]
fn matrix_text_store_returns_elements_correctly() {
    let store = vec![
        vec!["Hi".to_string()],
        vec!["my".to_string(), "name".to_string()],
        vec!["is".to_string(), "John".to_string(), "Cena".to_string()],
    ];
    assert_eq!(store.get_element((0, 0)), Some(&"Hi".to_string()));
    assert_eq!(store.get_element((0, 1)), None);
    assert_eq!(store.get_element((0, 2)), None);
    assert_eq!(store.get_element((1, 0)), Some(&"my".to_string()));
    assert_eq!(store.get_element((1, 1)), Some(&"name".to_string()));
    assert_eq!(store.get_element((1, 2)), None);
    assert_eq!(store.get_element((2, 0)), Some(&"is".to_string()));
    assert_eq!(store.get_element((2, 1)), Some(&"John".to_string()));
    assert_eq!(store.get_element((2, 2)), Some(&"Cena".to_string()));
    assert_eq!(store.get_element((1, 3)), None);
    assert_eq!(store.get_element((3, 0)), None);
}

#[test]
fn matrix_text_store_returns_slices_correctly() {
    let store = vec![
        vec!["world".to_string()],
        vec!["Hello".to_string(), "world".to_string()],
        vec!["Hello".to_string(), "my".to_string(), "world".to_string()],
    ];
    assert_eq!(
        store.get_element(TextIndex::new(
            (0usize, 0usize),
            ItemPosition::new("w", "orl"),
        )),
        Some("orl")
    );
    assert_eq!(
        store.get_element(TextIndex::new(
            (1, 0),
            ItemPosition::new("", "Hel"),
        )),
        Some("Hel")
    );
    assert_eq!(
        store.get_element(TextIndex::new(
            (1, 1),
            ItemPosition::new("", "w"),
        )),
        Some("w")
    );
    assert_eq!(
        store.get_element(TextIndex::new(
            (2, 0),
            ItemPosition::new("Hello", ""),
        )),
        Some("")
    );
    assert_eq!(
        store.get_element(TextIndex::new(
            (2, 1),
            ItemPosition::new("m", "y"),
        )),
        Some("y")
    );
    assert_eq!(
        store.get_element(TextIndex::new(
            (2, 2),
            ItemPosition::new("", "world"),
        )),
        Some("world")
    );
}
