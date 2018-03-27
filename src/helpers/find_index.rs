pub fn find_index<I, F>(source: I, predicate: F) -> Option<usize>
    where I: IntoIterator, F: Fn(&I::Item) -> bool {
    source.into_iter().enumerate()
        .find(|&(_, ref item)| predicate(item))
        .map(|(index, _)| index)
}
