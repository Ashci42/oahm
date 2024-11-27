use std::{hash::{DefaultHasher, Hash, Hasher}, usize};

const EXTEND_LIMIT: f32 = 0.6;
const INITIAL_CAPACITY: usize = 64;

pub struct OAHashMap<K, V> where K: Hash + Eq {
    buffer: Vec<Option<Entry<K, V>>>
}

impl<K, V> OAHashMap<K, V> where K: Hash + Eq {
    pub fn new() -> Self {
        let mut buffer = Vec::with_capacity(INITIAL_CAPACITY);
        for _ in 0..INITIAL_CAPACITY {
            buffer.push(None);
        }

        Self {
            buffer
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        if self.needs_extending() {
            self.extend();
        }

        let entry = Entry::new(key, value);
        self.insert_unchecked(entry);
    }

    pub fn search(&self, key: &K) -> Option<&V> {
        self.find_index(key).map(|index| &self.buffer[index].as_ref().unwrap().value)
    }

    pub fn delete(&mut self, key: &K) {
        let mut index = self.starting_index(key);
        while let Some(entry) = &self.buffer[index] {
            if &entry.key == key {
                std::mem::take(&mut self.buffer[index]);

                return;
            }

            index += 1;
        }
    }

    fn needs_extending(&self) -> bool {
        let number_of_elements = self.buffer.iter().filter(|entry| entry.as_ref().is_some_and(|entry| !entry.is_deleted)).count();
        let percentage = number_of_elements as f32 /  self.buffer.capacity() as f32;

        percentage > EXTEND_LIMIT
    }

    fn extend(&mut self) {
        let current_capacity = self.buffer.capacity();

        if current_capacity == usize::MAX {
            panic!("Reached max capacity");
        }

        let new_capacity = current_capacity.checked_mul(2);
        let new_capacity = new_capacity.unwrap_or(usize::MAX);

        let mut new_buffer: Vec<Option<Entry<K, V>>> = Vec::with_capacity(new_capacity);
        for _ in 0..new_capacity {
            new_buffer.push(None);
        }
        let old_buffer = std::mem::replace(&mut self.buffer, new_buffer);
        for entry in old_buffer {
            if let Some(entry) = entry {
                self.insert_unchecked(entry);
            }
        }
    }

    fn insert_unchecked(&mut self, entry: Entry<K, V>) {
        if let Some(index) = self.find_index(&entry.key) {
            self.buffer[index] = Some(entry);
            
            return;
        }

        let mut index = self.starting_index(&entry);
        while let Some(existing_entry) = &self.buffer[index] {
            if existing_entry.is_deleted {
                self.buffer[index] = Some(entry);

                return;
            }

            index += 1;
        }

        self.buffer[index] = Some(entry);
    }

    fn starting_index<H>(&self, hashable: &H) -> usize where H: Hash{
        let h = calculate_hash(hashable);

        h as usize % self.buffer.capacity()
    }

    fn find_index(&self, key: &K) -> Option<usize> {
        let mut index = self.starting_index(key);
        while let Some(entry) = &self.buffer[index] {
            if &entry.key == key {
                return Some(index);
            }

            index += 1;
        }

        None
    }
}

struct Entry<K, V> where K: Hash + Eq {
    key: K,
    value: V,
    is_deleted: bool,
}

impl<K, V> Entry<K, V> where K: Hash + Eq {
    fn new(key: K, value: V) -> Self {
        Self {
            key,
            value,
            is_deleted: false,
        }
    }
}

impl<K, V> Hash for Entry<K, V> where K: Hash + Eq {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key.hash(state)
    }
}

fn calculate_hash<H>(hashable: &H) -> u64 where H: Hash {
    let mut hasher = DefaultHasher::new();
    hashable.hash(&mut hasher);

    hasher.finish()
}

#[cfg(test)]
mod tests {
    use crate::OAHashMap;

    #[test]
    fn basic() {
        let mut oa: OAHashMap<i32, i32> = OAHashMap::new();
        oa.insert(1, 10);

        assert_eq!(Some(&10), oa.search(&1));

        oa.delete(&1);

        assert_eq!(None, oa.search(&1));

        oa.insert(1, 10);
        oa.insert(1, 20);

        assert_eq!(Some(&20), oa.search(&1));

        for i in 0..100 {
            oa.insert(i, i);
        }

        for i in 0..100 {
            assert_eq!(Some(&i), oa.search(&i));
        }
    }
}
