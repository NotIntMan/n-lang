/*!
    Сканер лексем

    Этот модуль разработан для преобразования исходных текстов языка N в набор лексем.
    Цель данного модуля: упростить архитектуру синтаксического анализатора.

    # Пример

    ```rust
    # use n_transpiler::lexeme_scanner::{
    #     TokenKind,
    #     Scanner,
    # };
    // Создадим сканер, передав ему ссылку на текст;
    let mut s: Scanner = Scanner::new("2+2");

    // Затем, вызовем next,
    let token = s.next()
    // затем распакуем Option<T> в T,
        .unwrap()
    // а затем распакуем Result<R, E> в R.
        .unwrap();
    // В итоге, в переменную token должна попасть структура типа Token следующего содержания.
    assert_eq!(token.kind, TokenKind::NumberLiteral { negative: false, radix: 10, fractional: false });
    assert_eq!(token.text, "2");
    ```

    # Пояснение

    В основе данного модуля лежит идея автоматов. Модуль `eaters` содержит набор
    функций, работающих по принципу детерминированных конечных автоматов.

    Они, приняв на вход `ScannerBatchIterator`, "съедают" столько символов, скольно необходимо
    для обработки лексемы, а затем возвращают тип обработанной лексемы (`TokenKind`). В случае, если ввод
    не является корректным, функция возвращает ошибку (`ScannerError`).

    Функция `scan`, располагающаяся в корне модуля `eaters`, реализует композицию всех функций модуля,
    поэтому её так же можно считать функцией-автоматом, и именно её использует сканер.

    Сканер, к слову, просто смотрит сколько символов было "потреблено" и, в зависимости от этого,
    генерирует текст и расположение для `Token`, который он вернёт позже.

    Некоторые типы лексем (`TokenKind`) отличаются от прочих:

    *   `TokenKind::EndOfInput` генерируется в случае, если ввод полностью и успешно "поглощён"
        и после этого токена сканер не сгенирирует ничего

    *   `TokenKind::Whitespace` никогда не генерируется т.к. существует указание сканеру
        игнорировать этот тип лексек
*/

pub mod eaters;
pub mod position;
pub mod scanner;
pub mod scan_error;
pub mod token;

pub use self::token::{
    Token,
    TokenKind,
};

pub use self::scanner::{
    Scanner,
    ScannerCursor,
};

pub use self::position::{
    SymbolPosition,
    ItemPosition,
};

pub use self::scan_error::{
    ScannerError,
    ScannerErrorKind,
};

pub type ScannerItem<'a> = Result<Token<'a>, ScannerError>;
pub type BatcherResult = Result<TokenKind, ScannerErrorKind>;

#[cfg(test)]
mod scanner_tests;