# Rust Borrowing

Borrowing is the mechanism that allows references to access data owned by
another variable. Immutable references can be shared, while mutable references
must be exclusive.

The borrow checker verifies these rules at compile time. This design prevents
data races and dangling references. Borrowing is closely related to ownership
because a reference must never outlive the value it points to.

中文补充：借用检查器可以在编译阶段发现不安全的数据访问。
