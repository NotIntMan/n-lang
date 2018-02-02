use super::{
    LexemeExact,
};

/// Тип правила, распознающего символы
pub type ConstSymbol = LexemeExact<'static>;

/// Правило, распознающее `,`
pub const COMMA: ConstSymbol = LexemeExact::group(",");

/// Правило, распознающее `;`
pub const SEMICOLON: ConstSymbol = LexemeExact::group(";");

/// Правило, распознающее `(`
pub const OPENING_ROUND_BRACKET: ConstSymbol = LexemeExact::group("(");

/// Правило, распознающее `)`
pub const CLOSING_ROUND_BRACKET: ConstSymbol = LexemeExact::group(")");

/// Правило, распознающее `<`
pub const OPENING_TRIANGULAR_BRACKET: ConstSymbol = LexemeExact::group("<");

/// Правило, распознающее `>`
pub const CLOSING_TRIANGULAR_BRACKET: ConstSymbol = LexemeExact::group(">");

/// Правило, распознающее `{`
pub const OPENING_BRACES_BRACKET: ConstSymbol = LexemeExact::group("{");

/// Правило, распознающее `}`
pub const CLOSING_BRACES_BRACKET: ConstSymbol = LexemeExact::group("}");
