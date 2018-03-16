use helpers::storage::{
    SourceStorage,
    Storage,
};

pub type NameStorage = Vec<String>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NameIndex {
    index: usize,
    offset: usize,
    length: usize,
}

impl SourceStorage<NameIndex> for NameStorage {
    type Element = str;
    fn get_element(&self, index: NameIndex) -> Option<&str> {
        let NameIndex { index, offset, length } = index;
        let string: &String = self.get(index)?;
        Some(&string[offset..(offset+length)])
    }
}

impl<'a> Storage<&'a str, NameIndex> for NameStorage {
    fn store_element(&mut self, element: &'a str) -> NameIndex {
        let element_length = element.len();
        for (index, item) in self.iter_mut().enumerate() {
            let item_length = item.len();
            if item_length < element_length {
                // item коточе, чем element
                let mut temp_length = element_length;
                while temp_length > 1 {
                    temp_length -= 1;
                    // Отсекаем один элемент справа от element и смотрим не совпадает ли он с концом item
                    let offset = item_length - temp_length;
                    if &element[..temp_length] == &item[offset..] {
                        // Совпадает - увеличиваем item и возвращаем NameIndex
                        item.push_str(&element[temp_length..]);
                        return NameIndex { index, offset, length: element_length }
                    }
                }
            } else {
                // item длиньше, чем element, либо их длины равны
                if let Some(offset) = item.find(element) {
                    // item содержит в себе подстроку element
                    return NameIndex { index, offset, length: element_length }
                }
            }
        }
        // Если мы дошли сюда, то долгий цикл не нашёл куда пристроить имя.
        // Придётся создать новый элемент.
        let index = self.len();
        self.push(element.to_string());
        NameIndex { index, offset: 0, length: element_length }
    }
}
