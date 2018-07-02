use std::{
    rc::Rc,
    cell::RefCell,
    fmt::{
        self,
        Display,
        Write,
    },
};

#[derive(Debug)]
pub struct CodeFormatter<'a, T: 'a + Write> {
    target: &'a mut T,
    pub indent_size: usize,
}

impl<'a, T: 'a + Write> CodeFormatter<'a, T> {
    pub fn new(target: &'a mut T) -> Self {
        Self {
            target,
            indent_size: 1,
        }
    }
    #[inline]
    fn write_indent(&mut self, indent_level: usize) -> fmt::Result {
        for _ in 0..indent_level {
            for _ in 0..self.indent_size {
                self.target.write_char(' ')?;
            }
        }
        Ok(())
    }
    pub fn write_line(&mut self, indent_level: usize, value: impl Display) -> fmt::Result {
        self.write_indent(indent_level)?;
        writeln!(&mut self.target, "{}", value)
    }
    pub fn root_block(self) -> (Rc<RefCell<Self>>, BlockFormatter<'a, T>) {
        let formatter = Rc::new(RefCell::new(self));
        let block = BlockFormatter {
            target: formatter.clone(),
            indent_level: 0,
        };
        (formatter, block)
    }
}

#[derive(Debug, Clone)]
pub struct BlockFormatter<'a, T: 'a + Write> {
    target: Rc<RefCell<CodeFormatter<'a, T>>>,
    indent_level: usize,
}

impl<'a, T: 'a + Write> BlockFormatter<'a, T> {
    pub fn write_line(&mut self, line: impl Display) -> fmt::Result {
        let mut f = self.target.borrow_mut();
        f.write_line(self.indent_level, line)
    }
    pub fn sub_block(&self) -> Self {
        Self {
            target: self.target.clone(),
            indent_level: self.indent_level + 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_class() -> fmt::Result {
        let mut result = String::new();
        {
            let (_formatter, mut block) = {
                let mut f = CodeFormatter::new(&mut result);
                f.indent_size = 2;
                f.root_block()
            };
            block.write_line("class X {")?;
            let mut sub_block = block.sub_block();
            sub_block.write_line("function a () {")?;
            let mut sub_sub_block = sub_block.sub_block();
            sub_sub_block.write_line(format_args!("return {};", true))?;
            sub_block.write_line("}")?;
            block.write_line("}")?;
        }
        assert_eq!("class X {\
        \n  function a () {\
        \n    return true;\
        \n  }\
        \n}\
        \n", result);
        Ok(())
    }
}
