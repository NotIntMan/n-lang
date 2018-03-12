// TODO Подумать, а не запилить ли всё через вектор строк (так поиск и расположение будут такими же медленными, но оптимизация удлиннения имени будет работать чаще и запись имён в файле будет иметь более осмысленный вид)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamesStorage(String);

impl NamesStorage {
    fn new() -> Self {
        NamesStorage(String::new())
    }
    fn with_capacity(size: usize) -> Self {
        NamesStorage(String::with_capacity(size))
    }
    fn find(&self, name: &str) -> Option<usize> {
        self.0.find(name)
    }
    fn place(&mut self, name: &str) -> usize {
        if let Some(pos) = self.find(name) {
            return pos;
        }
        let store_len = self.0.len();
        {
            let mut temp_len = name.len();
            while temp_len > 1 {
                temp_len -= 1;
                let temp_name = &name[..temp_len];
                let position_at_the_right_of_the_store = store_len - temp_len;
                if temp_name == &self.0[position_at_the_right_of_the_store..] {
                    self.0.push_str(&name[temp_len..]);
                    return position_at_the_right_of_the_store;
                }
            }
        }
        self.0.push_str(name);
        store_len
   }
}
