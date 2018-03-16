pub trait SourceStorage<Index = usize> {
    type Element: ? Sized;
    fn get_element(&self, index: Index) -> Option<&Self::Element>;
}

pub trait Storage<Element, Index = usize>: SourceStorage<Index> {
    fn store_element(&mut self, element: Element) -> Index;
}

impl<T> SourceStorage for Vec<T> {
    type Element = T;
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

impl<T> SourceStorage<(usize, usize)> for Vec<Vec<T>>
{
    type Element = T;
    fn get_element(&self, index: (usize, usize)) -> Option<&T> {
        let (index, sub_index) = index;
        let sub_store: &Vec<T> = self.get_element(index)?;
        sub_store.get_element(sub_index)
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
