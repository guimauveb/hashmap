# Rust hash map
## Taken from this [great article](https://betterprogramming.pub/implementing-a-hashmap-in-rust-35d055b5ac2b) with some improvements:
- `hash_key` only requires a reference to `K`.
- `get` and `remove` take a reference to `K` like `std::collections::HashMap`.
- `K` must only implement `Hash` and `PartialEq`, which allows for a greater set of types to be used as keys.
- `V` has no bounds.
- Places that required an owned value of `V` use `std::mem::replace` instead (typically when returning the old value on an `insert` with an existing key).


# TODO 
- Add some tests

