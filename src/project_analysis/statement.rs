use std::ops::{
    Add,
    AddAssign,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatementFlowControlPosition {
    in_cycle: bool,
}

impl StatementFlowControlPosition {
    #[inline]
    pub fn new() -> Self {
        StatementFlowControlPosition {
            in_cycle: false,
        }
    }
    #[inline]
    pub fn in_cycle(mut self) -> Self {
        self.in_cycle = true;
        self
    }
    #[inline]
    pub fn is_in_cycle(&self) -> bool {
        self.in_cycle
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatementFlowControlJumping {
    Nothing,
    Sometimes {
        returns: bool,
        breaks: bool,
        continues: bool,
    },
    AlwaysReturns,
    AlwaysBreaks,
    AlwaysContinues,
}

impl StatementFlowControlJumping {
    #[inline]
    pub fn is_returns(&self) -> bool {
        match self {
            StatementFlowControlJumping::Sometimes {
                returns,
                breaks: _,
                continues: _,
            } => *returns,
            StatementFlowControlJumping::AlwaysReturns => true,
            _ => false,
        }
    }
    #[inline]
    pub fn is_breaks(&self) -> bool {
        match self {
            StatementFlowControlJumping::Sometimes {
                returns: _,
                breaks,
                continues: _,
            } => *breaks,
            StatementFlowControlJumping::AlwaysBreaks => true,
            _ => false,
        }
    }
    #[inline]
    pub fn is_continues(&self) -> bool {
        match self {
            StatementFlowControlJumping::Sometimes {
                returns: _,
                breaks: _,
                continues,
            } => *continues,
            StatementFlowControlJumping::AlwaysContinues => true,
            _ => false,
        }
    }
}

impl Add for StatementFlowControlJumping {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        if self == rhs { return self; }

        let mut result_returns = false;
        let mut result_breaks = false;
        let mut result_continues = false;

        match self {
            StatementFlowControlJumping::Nothing => {}
            StatementFlowControlJumping::Sometimes {
                returns,
                breaks,
                continues,
            } => {
                result_returns = returns;
                result_breaks = breaks;
                result_continues = continues;
            }
            StatementFlowControlJumping::AlwaysReturns => result_returns = true,
            StatementFlowControlJumping::AlwaysBreaks => result_breaks = true,
            StatementFlowControlJumping::AlwaysContinues => result_continues = true,
        }

        match rhs {
            StatementFlowControlJumping::Nothing => {}
            StatementFlowControlJumping::Sometimes {
                returns,
                breaks,
                continues,
            } => {
                result_returns = result_returns || returns;
                result_breaks = result_breaks || breaks;
                result_continues = result_continues || continues;
            }
            StatementFlowControlJumping::AlwaysReturns => result_returns = true,
            StatementFlowControlJumping::AlwaysBreaks => result_breaks = true,
            StatementFlowControlJumping::AlwaysContinues => result_continues = true,
        }

        if result_returns || result_breaks || result_continues {
            StatementFlowControlJumping::Sometimes {
                returns: result_returns,
                breaks: result_breaks,
                continues: result_continues,
            }
        } else {
            StatementFlowControlJumping::Nothing
        }
    }
}

impl AddAssign for StatementFlowControlJumping {
    fn add_assign(&mut self, rhs: StatementFlowControlJumping) {
        *self = *self + rhs;
    }
}
