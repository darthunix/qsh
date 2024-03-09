# Конвертер из QSH в CSV

Консольная утилита для конвертации [QScalp History](https://www.qscalp.ru/qsh-service) файлов (четвертая версия) со сделками (поток Deals) в CSV формат (с `;` в качестве разделителя).

## Как собрать?

1. Установить [git](https://git-scm.com/book/ru/v2/%D0%92%D0%B2%D0%B5%D0%B4%D0%B5%D0%BD%D0%B8%D0%B5-%D0%A3%D1%81%D1%82%D0%B0%D0%BD%D0%BE%D0%B2%D0%BA%D0%B0-Git).
1. Установить [rust](https://www.rust-lang.org/tools/install).
1. Вытянуть исходный код в локальный каталог:
   ```sh
   git clone https://github.com/darthunix/qsh.git
   cd qsh
   ```
1. Собрать исполняемый файл.
   ```sh
   cargo build --release
   ```

В результате, в каталоге `target/release` будет создан исполняемый файл `qsh` (он-то нам и нужен).

## Как использовать?

1. Посмотреть поддерживаемые флаги можно через `qsh --help`. Но там сейчас только один полезный флаг `--file` (или же в краткой версии `-f`) - путь до QSH файла,
   который мы хотим конвертировать.
1. Выходной поток пишется прямо в stdout (в терминал), так что нужно его не забыть перенаправить в целевой файл:
   ```sh
   qsh -f data/SBER.2024-02-20.Deals.qsh > SBER.2024-02-20.Deals.csv
   ```
