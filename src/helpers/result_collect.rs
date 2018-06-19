pub fn deep_result_collect<R, E>(
    input: impl IntoIterator<Item=Result<R, E>>,
) -> Result<Vec<R>, Vec<E>>
{
    let mut results = Vec::new();
    let mut errors = Vec::new();
    for item in input {
        match item {
            Ok(item_result) => {
                if errors.is_empty() {
                    results.push(item_result);
                }
            }
            Err(item_error) => {
                errors.push(item_error);
            }
        }
    }
    if errors.is_empty() {
        Ok(results)
    } else {
        Err(errors)
    }
}

pub fn accumulative_result_collect<R, E>(
    input: impl IntoIterator<Item=Result<R, Vec<E>>>,
) -> Result<Vec<R>, Vec<E>>
{
    let mut results = Vec::new();
    let mut errors = Vec::new();
    for item in input {
        match item {
            Ok(item_result) => {
                if errors.is_empty() {
                    results.push(item_result);
                }
            }
            Err(mut item_errors) => {
                errors.append(&mut item_errors);
            }
        }
    }
    if errors.is_empty() {
        Ok(results)
    } else {
        Err(errors)
    }
}
