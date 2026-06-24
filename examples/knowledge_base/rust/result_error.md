# Result and Error Handling

Rust programs often use Result to represent operations that may fail. File
reading, JSON parsing, and command execution should return Result instead of
calling unwrap everywhere.

The question mark operator propagates errors clearly. A project can define an
application error enum to describe IO errors, invalid paths, empty queries, and
missing index files.
