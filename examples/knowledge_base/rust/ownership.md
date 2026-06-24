# Rust Ownership

Rust ownership means each value has one owner. When the owner goes out of scope,
Rust automatically drops the value. This rule helps the compiler manage memory
without a garbage collector.

Borrowing allows a function to read data without taking ownership. Mutable
borrowing allows controlled modification, but only one mutable reference can
exist at a time. The ownership system is the foundation of Rust memory safety.

中文补充：所有权机制用于管理内存安全，借用允许程序在不转移所有权的情况下访问数据。
