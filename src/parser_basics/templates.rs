//! Набор примитивных шаблонов образования языка

//use syntax_parser::basics::{
//    LexemeCursor,
//    LexemeParser,
//    LexemeParserResult,
//};
//
//use syntax_parser::basics::constants::{
//    ConstSymbol,
//    COMMA,
//    SEMICOLON,
//    CLOSING_BRACES_BRACKET,
//    CLOSING_ROUND_BRACKET,
//    CLOSING_TRIANGULAR_BRACKET,
//    OPENING_BRACES_BRACKET,
//    OPENING_ROUND_BRACKET,
//    OPENING_TRIANGULAR_BRACKET,
//};

/*
    Это дело не скопилируется потому, что, похоже, шаблонам придётся учитывать подмакросы.
    Надо подумать над этим т.к. я не хочу реализовывать экспоненциальное количество вариантов макроса.
    TODO Решить задачку с макросами-шаблонами
*/

#[macro_export]
macro_rules! call_template {
    ($i: expr, $name: ident) => { $name($i) };
    ($i: expr, $name: ident! ( ( $arg: expr ),* )) => { $name!($i, ( $arg: expr ),*) };
}

//#[test]
//fn a() {
//    fn inc(i: u8) -> u8 { i + 1 }
//    assert_eq!(call_template!(5, inc), 6);
//    assert_eq!(call_template!(5, call_template!(inc)), 6);
//}

#[macro_export]
macro_rules! list {
    ($i: expr, $($element:tt)* , $($delimiter:tt)*) => {{
        use nom::IResult;
        use parser_basics::ParserInput;
        let mut result = Vec::new();
        let mut input = $i.clone();
        'parse_cycle: loop {
            match call_template!(input, $($element)*) {
                IResult::Done(I, O) => {
                    input = I;
                    result.push(O);
                },
                _ => { break 'parse_cycle },
            }
            match call_template!(input, $($delimiter)*) {
                IResult::Done(I, _) => {
                    input = I;
                },
                _ => { break 'parse_cycle },
            }
        }
        input.ok(0, result)
    }};
}

//#[test]
//fn x() {
//    use nom;
//    use parser_basics;
//    use parser_basics::symbols;
//    parser_rule!(pluses(i, kind: TokenKindLess) -> &'source str {
//        do_parse!(i,
//            x: list!(
//                apply!(symbols, "+"),
//                apply!(symbols, ",")
//            ) >>
//            (x)
//        )
//    });
//    use lexeme_scanner::Scanner;
//    let buf = Scanner::scan("+, + , +")
//        .expect("Scanner result must be ok");
//    let input = buf.as_slice();
//    assert_eq!(
//        pluses(input)
//            .to_result()
//            .expect("Parser result must be ok"),
//        vec![]
//    );
//}

// /**
//    Шаблон "Список".
//    Используется для разбора списка `Element`, разделённых `Delimiter`.
//    В конце списка `Delimiter` является опциональным.
//    Возвращает вектор успешно разобранных значений (`Vec<Element::Result>`).
//    Никогда не возвращает ошибку.
//*/
//#[derive(Debug, Clone, PartialEq, Eq)]
//pub struct TemplateList<Element, Delimiter>(pub Element, pub Delimiter);
//
//impl<Element> TemplateList<Element, ConstSymbol> {
//    /// Создаёт правило, разбирающее список `Element`, разделённых символом `,`.
//    #[inline]
//    pub const fn comma(e: Element) -> Self {
//        TemplateList(e, COMMA)
//    }
//    /// Создаёт правило, разбирающее список `Element`, разделённых символом `;`.
//    #[inline]
//    pub const fn semicolon(e: Element) -> Self {
//        TemplateList(e, SEMICOLON)
//    }
//}
//
//impl<'a, 'b, Element, Delimiter> LexemeParser<'a, 'b> for TemplateList<Element, Delimiter>
//    where Element: LexemeParser<'a, 'b>,
//          Delimiter: LexemeParser<'a, 'b>,
//{
//    type Result = Vec<Element::Result>;
//    fn parse(&self, cursor: &mut LexemeCursor<'a, 'b>) -> LexemeParserResult<Self::Result> {
//        let mut result = Vec::new();
//        let mut begin;
//        'parse_loop: loop {
//            begin = cursor.index;
//            match self.0.parse(cursor) {
//                Ok(r) => result.push(r),
//                Err(_) => break 'parse_loop,
//            }
//            if let Err(_) = self.1.parse(cursor) {
//                break 'parse_loop;
//            }
//        }
//        cursor.index = begin;
//        Ok(result)
//    }
//}

// /**
//    Шаблон "Обёртка".
//    Используется, в основном, для скобок.
//    Представляет собой последовательность открывающейся скобки, элемента и закрывающейся скобки.
//    Возвращает `Element::Result` в случае успеха.
//    В случае обнаружения ошибки разбора возвращает её.
//*/
//#[derive(Debug, Clone, PartialEq, Eq)]
//pub struct TemplateWrap<OpenBracket, Element, CloseBracket>(pub OpenBracket, pub Element, pub CloseBracket);
//
//impl<Element> TemplateWrap<ConstSymbol, Element, ConstSymbol> {
//    /// Создаёт правило, разбирающее список `Element`, обёрнутый символами `(` и `)`.
//    #[inline]
//    pub const fn round(e: Element) -> Self {
//        TemplateWrap(OPENING_ROUND_BRACKET, e, CLOSING_ROUND_BRACKET)
//    }
//    /// Создаёт правило, разбирающее список `Element`, обёрнутый символами `<` и `>`.
//    #[inline]
//    pub const fn triangular(e: Element) -> Self {
//        TemplateWrap(OPENING_TRIANGULAR_BRACKET, e, CLOSING_TRIANGULAR_BRACKET)
//    }
//    /// Создаёт правило, разбирающее список `Element`, обёрнутый символами `{` и `}`.
//    #[inline]
//    pub const fn braces(e: Element) -> Self {
//        TemplateWrap(OPENING_BRACES_BRACKET, e, CLOSING_BRACES_BRACKET)
//    }
//}
//
//impl<'a, 'b, OpenBracket, Element, CloseBracket> LexemeParser<'a, 'b> for TemplateWrap<OpenBracket, Element, CloseBracket>
//    where Element: LexemeParser<'a, 'b>,
//          OpenBracket: LexemeParser<'a, 'b>,
//          CloseBracket: LexemeParser<'a, 'b>,
//{
//    type Result = Element::Result;
//    fn parse(&self, cursor: &mut LexemeCursor<'a, 'b>) -> LexemeParserResult<Self::Result> {
//        self.0.parse(cursor)?;
//        let result = self.1.parse(cursor)?;
//        self.2.parse(cursor)?;
//        Ok(result)
//    }
//}
