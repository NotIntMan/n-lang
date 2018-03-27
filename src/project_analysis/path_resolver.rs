use parser_basics::StaticIdentifier;
use syntax_parser::others::StaticPath;
use project_analysis::error::SemanticError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathResolver<'a> {
    base: &'a [StaticIdentifier],
    base_index: usize,
    path: &'a [StaticIdentifier],
    path_index: usize,
}

impl<'a> PathResolver<'a> {
    pub fn new(mut base: &'a [StaticIdentifier], path: &'a StaticPath) -> Result<Self, SemanticError> {
        let mut dep_path: &[StaticIdentifier] = &path.path[..];

        match dep_path.get(0)
            .ok_or_else(|| SemanticError::item_name_not_specified(path.pos))?
            .get_text() {
            "self" => {
                dep_path = &dep_path[1..];
            }
            "super" => {
                'super_loop: loop {
                    dep_path = &dep_path[1..];
                    match base.len() {
                        0 => return Err(SemanticError::super_of_global(path.pos)),
                        length => base = &base[..(length - 1)],
                    }
                    if dep_path.len() == 0 {
                        return Err(SemanticError::item_name_not_specified(path.pos));
                    }
                    if "super" != dep_path.get(0)
                        .ok_or_else(|| SemanticError::item_name_not_specified(path.pos))?
                        .get_text() {
                        break 'super_loop;
                    }
                }
            }
            _ => {}
        }
        Ok(PathResolver {
            base,
            base_index: 0,
            path: dep_path,
            path_index: 0,
        })
    }
}

impl<'a> Iterator for PathResolver<'a> {
    type Item = &'a StaticIdentifier;
    fn next(&mut self) -> Option<&'a StaticIdentifier> {
        if self.base_index < self.base.len() {
            let result = &self.base[self.base_index];
            self.base_index += 1;
            return Some(result);
        }
        if self.path_index < self.path.len() {
            let result = &self.path[self.path_index];
            self.path_index += 1;
            return Some(result);
        }
        None
    }
}