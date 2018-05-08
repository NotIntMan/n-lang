#[inline]
fn parse_begin<'a>(input: &'a str, begin: &str) -> Option<&'a str> {
    let begin_length = begin.len();
    if input.len() < begin_length {
        return None;
    }
    if &input[..begin_length] == begin {
        Some(&input[begin_length..])
    } else {
        None
    }
}

pub fn parse_index(mut input: &str) -> Option<usize> {
    input = parse_begin(input, "component")?;
    input.parse().ok()
}
