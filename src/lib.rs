//! Taken from this [great article](https://betterprogramming.pub/implementing-a-hashmap-in-rust-35d055b5ac2b) with some small improvements:
//! - `hash_key` only requires a reference to `K`.
//! - `get` and `remove` take a reference to `K`.
//! - `K` should only implement `Hash` and `PartialEq`, which allows for a greater set of types to be used as keys.
//! - `V` has no bounds.
//! - Places that required an owned value of `V` use `std::mem::replace` instead (typically when returning the old value on an `insert` with an existing key).
use std::{
    collections::hash_map::DefaultHasher,
    fmt::Debug,
    hash::{Hash, Hasher},
};

// TODO - Use a prime?
const DEFAULT_MAX_SIZE: u64 = 256;

fn hash_key<K: Hash>(key: K) -> u64 {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    hasher.finish()
}

pub struct KeyValue<K, V>
where
    K: PartialEq,
{
    key: K,
    value: V,
    next: Option<Box<KeyValue<K, V>>>,
}

impl<K, V> Debug for KeyValue<K, V>
where
    K: Hash + PartialEq + Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{ {:?}: {:?} - next: {:?} }}",
            self.key, self.value, self.next
        )
    }
}

impl<K, V> KeyValue<K, V>
where
    K: PartialEq,
{
    pub fn new(key: K, value: V) -> Self {
        Self {
            key,
            value,
            next: None,
        }
    }
}

pub struct HashMap<K, V>
where
    K: Hash + PartialEq,
{
    curr_size: usize,
    array: [Option<KeyValue<K, V>>; DEFAULT_MAX_SIZE as usize],
}

impl<K, V> Debug for HashMap<K, V>
where
    K: Hash + PartialEq + Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let occupied = self
            .array
            .iter()
            .filter_map(|v| v.as_ref())
            .collect::<Vec<_>>();
        write!(f, "{occupied:?}")
    }
}

impl<K, V> HashMap<K, V>
where
    K: Hash + PartialEq,
{
    fn insert_new_value(&mut self, key: K, value: V, position: usize) {
        let new_entry = KeyValue::new(key, value);
        self.array[position].replace(new_entry);
        self.curr_size += 1;
    }

    /// Traverse the linked list until we either find the value and update it, or append the list with the new value.
    fn update_or_link_new_val(&mut self, key: K, value: V, position: usize) -> Option<V> {
        let mut current_kv = self.array[position].as_mut().unwrap();
        if current_kv.key == key {
            return Some(std::mem::replace(&mut current_kv.value, value));
        }

        while current_kv.next.is_some() {
            let node = current_kv.next.as_mut().unwrap();
            if node.key == key {
                return Some(std::mem::replace(&mut node.value, value));
            }
            current_kv = node;
        }

        // Append the new entry at the end of the linked list.
        current_kv.next.replace(KeyValue::new(key, value).into());
        self.curr_size += 1;
        None
    }

    /// Insert a key-value pair into the hashmap.
    /// Returns `None` if the value didn’t exist, or returns the old value if the key was present.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let hash = hash_key(&key);
        let position = (hash % DEFAULT_MAX_SIZE) as usize;
        if self.array[position].is_some() {
            self.update_or_link_new_val(key, value, position)
        } else {
            self.insert_new_value(key, value, position);
            None
        }
    }

    /// Get the value for a given key. Returns the value if it exists, or `None` otherwise.
    pub fn get(&self, key: &K) -> Option<&V> {
        let hash_val = hash_key(key);
        let position = (hash_val % DEFAULT_MAX_SIZE) as usize;

        if let Some(mut kv) = self.array[position].as_ref() {
            if &kv.key == key {
                return Some(&kv.value);
            }
            while let Some(node) = kv.next.as_ref() {
                if &node.key == key {
                    return Some(&node.value);
                }
                kv = node;
            }
        }
        None
    }

    /// Removes the key-value pair from the map for a given key.
    /// Returns the value if that key existed, `None` otherwise.
    pub fn remove(&mut self, key: &K) -> Option<V> {
        let hash = hash_key(key);
        let position = (hash % DEFAULT_MAX_SIZE) as usize;

        if let Some(mut kv) = self.array[position].as_mut() {
            if &kv.key == key {
                self.curr_size -= 1;
                if let Some(next) = kv.next.take() {
                    return self.array[position].replace(*next).map(|kv| kv.value);
                }
                return self.array[position].take().map(|kv| kv.value);
            }
            while let Some(node) = kv.next.as_mut() {
                if &node.key == key {
                    self.curr_size -= 1;
                    // Link the deleted node `next` node to the previous node.
                    if let Some(next) = node.next.take() {
                        return kv.next.replace(next).map(|kv| kv.value);
                    } else {
                        return kv.next.take().map(|kv| kv.value);
                    }
                }
                kv = kv.next.as_mut().unwrap();
            }
        }
        None
    }

    /// Clear the hashmap.
    pub fn clear(&mut self) {
        self.array = [Self::INIT; DEFAULT_MAX_SIZE as usize];
        self.curr_size = 0;
    }

    /// Declaring it as `const` to avoid the requirement for `KeyValue<K, V>` to implement `Copy`,
    const INIT: Option<KeyValue<K, V>> = None;

    pub fn new() -> Self {
        Self {
            curr_size: 0,
            array: [Self::INIT; DEFAULT_MAX_SIZE as usize],
        }
    }
}

// TODO - Add some tests (use some leetcode tests)
//      - Insert, get, update (insert a new value with an existing key).
//      - Insert keys whose hashes collide.
//      - Bench insert, get, delete, update against original impl
#[cfg(test)]
mod tests {
    use super::HashMap;

    #[test]
    fn insert_and_get_item() {
        let (key, value) = ("guimauve", 1);
        let mut hashmap: HashMap<&str, i32> = HashMap::new();
        hashmap.insert(key, value);
        let result = *hashmap.get(&key).unwrap();
        assert_eq!(result, value);

        let (key, value) = ("rust", 2);
        hashmap.insert(key, value);
        let result = *hashmap.get(&key).unwrap();
        assert_eq!(result, value);

        println!("HashMap: {hashmap:?}");
    }

    //#[test]
    //fn delete_key_value() {
    //    let (key, value) = ("guimauve", 1);
    //    let mut hashmap: HashMap<&str, i32> = HashMap::new();
    //    hashmap.insert(key, value);
    //    let result = *hashmap.get(&key).unwrap();
    //    assert_eq!(result, value);

    //    hashmap.remove(&key);
    //    assert!(hashmap.get(&key).is_none());
    //}

    //// TODO - Find some keys whoses hashes collide and see what happens.
    //#[test]
    //fn handle_colision() {
    //    let (key, value) = ("guimauve", 1);
    //    let mut hashmap: HashMap<&str, i32> = HashMap::new();
    //    hashmap.insert(key, value);
    //    let result = *hashmap.get(&key).unwrap();
    //    assert_eq!(result, value);

    //    let (key, value) = ("rust", 2);
    //    let mut hashmap: HashMap<&str, i32> = HashMap::new();
    //    hashmap.insert(key, value);
    //    let result = *hashmap.get(&key).unwrap();
    //    assert_eq!(result, value);

    //    println!("{hashmap:?}");
    //}
}
