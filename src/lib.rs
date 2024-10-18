use std::borrow::Borrow;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::mem;

const INITIAL_N_BUCKETS: usize = 1;

pub struct HashMap<K, V> {
    buckets: Vec<Vec<(K, V)>>,
    items: usize,
}

impl<K, V> HashMap<K, V>
where
    K: Hash + Eq,
{
    pub fn new() -> Self {
        HashMap {
            buckets: Vec::new(),
            items: 0,
        }
    }

    fn resize(&mut self) {
        let target_size = match self.buckets.len() {
            0 => INITIAL_N_BUCKETS,
            n => 2 * n,
        };

        let mut new_buckets = Vec::with_capacity(target_size);
        new_buckets.extend((0..target_size).map(|_| Vec::new()));
        for (key, value) in self.buckets.iter_mut().flat_map(|bucket| bucket.drain(..)) {
            let mut hasher = DefaultHasher::new();
            key.hash(&mut hasher);
            let bucket_idx = (hasher.finish() % (new_buckets.len() as u64)) as usize;
            new_buckets[bucket_idx].push((key, value));
        }

        self.buckets = new_buckets;
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if self.buckets.is_empty() || self.items > 3 * self.buckets.len() / 4 {
            self.resize();
        }

        let bucket_idx = self.bucket_idx(&key);
        let bucket = &mut self.buckets[bucket_idx];

        for &mut (ref ekey, ref mut evalue) in bucket.iter_mut() {
            if ekey == &key {
                return Some(mem::replace(evalue, value));
            }
        }

        self.items += 1;
        bucket.push((key, value));
        None
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Eq + Hash + ?Sized,
    {
        let bucket_idx = self.bucket_idx(key);
        self.buckets[bucket_idx]
            .iter()
            .find(|(ekey, _)| ekey.borrow() == key)
            .map(|(_, value)| value)
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: ?Sized + Eq + Hash,
    {
        let bucket_idx = self.bucket_idx(key);
        let i = self.buckets[bucket_idx]
            .iter()
            .position(|(ekey, _)| ekey.borrow() == key)?;
        let bucket = &mut self.buckets[bucket_idx];
        self.items -= 1;
        Some(bucket.swap_remove(i).1)
    }

    pub fn len(&self) -> usize {
        self.items
    }

    pub fn is_empty(&self) -> bool {
        self.items == 0
    }

    fn bucket_idx<Q>(&self, key: &Q) -> usize
    where
        K: Borrow<Q>,
        Q: Eq + Hash + ?Sized,
    {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        (hasher.finish() % (self.buckets.len() as u64)) as usize
    }

    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: ?Sized + Eq + Hash,
    {
        let bucket_idx = self.bucket_idx(key);
        self.buckets[bucket_idx]
            .iter()
            .any(|(ekey, _)| ekey.borrow() == key)
    }
}

pub struct HashIter<'a, K, V> {
    map: &'a HashMap<K, V>,
    current_bucket: usize,
    current_item: usize,
}

impl<'a, K, V> HashIter<'a, K, V> {
    pub fn new(hash_map: &'a HashMap<K, V>) -> Self {
        Self {
            map: hash_map,
            current_bucket: 0,
            current_item: 0,
        }
    }
}

impl<'a, K, V> Iterator for HashIter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.map.buckets.get(self.current_bucket) {
                Some(bucket) => match bucket.get(self.current_item) {
                    Some((k, v)) => {
                        self.current_item += 1;
                        break Some((k, v));
                    }
                    None => {
                        self.current_bucket += 1;
                        self.current_item = 0;
                        continue;
                    }
                },
                None => break None,
            }
        }
    }
}

impl<'a, K, V> IntoIterator for &'a HashMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = HashIter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        HashIter::new(self)
    }
}

impl<K, V> Default for HashMap<K, V>
where
    K: Hash + Eq,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_functionality() {
        let mut map = HashMap::new();
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
        map.insert("foo", 42);
        assert!(!map.is_empty());
        assert_eq!(map.len(), 1);
        assert_eq!(map.get("foo"), Some(&42));
        assert_eq!(map.remove("foo"), Some(42));
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
        assert_eq!(map.get("foo"), None);
    }
}
