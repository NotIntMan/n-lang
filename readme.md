# Язык N

Проект языка программирования хранимых процедур.

## Добро пожаловать.

На данный момент проект находится на стадии ранней разработки.
Это значит, что до ближайшей релизной версии предстоит ещё очень много работы.

## Цели реализации транслятора

- [x] Лексический анализатор текста
- [x] Синтаксический анализатор текста
- [ ] Семантический анализатор текста
- [ ] Генератор запросов
- [ ] Генератор вызывающего кода

## Цели реализации языка

- [ ] Синтаксис
- [ ] Цикл работы и взаимодействия программы на языке с БД и вызывающим кодом

## Цели оптимизации

- [ ] Продвинутая аллокация памяти для объектов и их наборов (напр., векторов).

    Во время парсинга постоянно происходит генерация сообщений о неудаче разбора,
    которые отбрасываются в угоду успешному варианту (если таковой будет иметь место).
    Загвоздка в том, что генерация таких ошибок требует аллокации памяти для хранения
    таких объектов как строки и множества.

    - [ ] ИЛИ Переделка модуля `parser_basics` так, чтобы сообщения об ошибках не требовали аллокации.

        Однако, из-за, например, строк, этого будет тяжело добиться т.к. имеет место форматирование текста,
        которое генерирует строки на лету. Собственно, форматирование - единственная причина наличия
        строк, а не ссылок на срез символов, в ошибках.

## Построен на

- Язык программирования [Rust]
- Пакетный менеджер [Cargo]
- Библиотека комбинаторов парсеров [Nom]
- Библиотека логгирования [Log]
- Библиотека форматирования вывода во время отладки [PrettyAssertion]
- Титанические усилия автора

## Авторы

- Дмитрий Демин ([github.com/0xDE11A][0xDE11A])

[Rust]: https://www.rust-lang.org
[Cargo]: https://crates.io
[Nom]: https://crates.io/crates/nom
[Log]: https://crates.io/crates/log
[PrettyAssertion]: https://crates.io/crates/pretty_assertions
[0xDE11A]: https://github.com/0xDE11A
