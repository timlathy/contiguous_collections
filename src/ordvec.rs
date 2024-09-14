use std::{fmt, fmt::Debug};

/// Ordered [`Vec`] intended for fast lookup of items by key.
pub struct OrdVec<T: OrdVecKey>(Vec<T>);

pub trait OrdVecKey {
    type Key: Ord;
    fn get_key(item: &Self) -> Self::Key;
}

impl<K: OrdVecKey, V> OrdVecKey for (K, V) {
    type Key = K::Key;
    #[inline(always)]
    fn get_key(item: &Self) -> Self::Key {
        OrdVecKey::get_key(&item.0)
    }
}

impl<T: OrdVecKey> OrdVec<T> {
    pub const fn new() -> Self {
        OrdVec(Vec::new())
    }

    pub fn from_unsorted(mut vec: Vec<T>) -> Self {
        vec.sort_unstable_by_key(<T as OrdVecKey>::get_key);
        assert!(
            vec.windows(2)
                .all(|pair| <T as OrdVecKey>::get_key(&pair[0])
                    != <T as OrdVecKey>::get_key(&pair[1])),
            "OrdVec must not contain duplicate keys"
        );
        OrdVec(vec)
    }

    pub fn insert(&mut self, item: T) {
        let insert_idx = if let Some(last_item) = self.0.last() {
            let k = <T as OrdVecKey>::get_key(&item);
            if k <= <T as OrdVecKey>::get_key(&last_item) {
                match self.0.binary_search_by_key(&k, <T as OrdVecKey>::get_key) {
                    Ok(_) => panic!("Cannot insert an item with a duplicate key"),
                    Err(i) => i,
                }
            } else {
                self.0.len()
            }
        } else {
            self.0.len()
        };
        self.0.insert(insert_idx, item);
    }

    pub fn get_by_key(&self, k: &<T as OrdVecKey>::Key) -> Option<&T> {
        self.0
            .binary_search_by_key(k, <T as OrdVecKey>::get_key)
            .ok()
            .map(|i| &self.0[i])
    }

    pub fn get_mut_by_key(&mut self, k: &<T as OrdVecKey>::Key) -> Option<&mut T> {
        self.0
            .binary_search_by_key(k, <T as OrdVecKey>::get_key)
            .ok()
            .map(|i| &mut self.0[i])
    }

    // pub fn get_index_by_key(&self, k: &<T as OrdVecKey>::Key) -> Option<usize> {
    //     self.0.binary_search_by_key(k, <T as OrdVecKey>::get_key).ok()
    // }

    pub fn remove_by_key(&mut self, k: &<T as OrdVecKey>::Key) -> Option<T> {
        self.0
            .binary_search_by_key(k, <T as OrdVecKey>::get_key)
            .ok()
            .map(|i| self.0.remove(i))
    }

    /// Apply the function to each element of the [`OrdVec`] and depending on the return value:
    /// * Replace the element with the new value if the function returns Some(T),
    /// * Remove the element if the function returns None.
    ///
    /// The order of iteration may not follow the order of keys. Compare to [`Vec::retain_mut`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use contiguous_collections::{OrdVec, OrdVecKey};
    /// # fn main() {
    /// # #[derive(Debug, PartialEq)]
    /// # struct K(u32);
    /// impl OrdVecKey for K {
    ///   type Key = u32;
    ///   fn get_key(item: &Self) -> Self::Key { item.0 }
    /// }
    /// let mut lv = OrdVec::from_unsorted(vec![
    ///     (K(0), "0"), (K(1), "1"), (K(2), "2"), (K(3), "3"),
    ///     (K(4), "4"), (K(5), "5"), (K(6), "6"), (K(7), "7")
    /// ]);
    /// let mut retain_map_order = Vec::new();
    /// lv.retain_map(|(K(i), v)| {
    ///     retain_map_order.push(i);
    ///     if i % 2 == 0 { Some((K(6 - i), v)) } else { None }
    /// });
    /// assert_eq!(&lv[..], &[(K(0), "6"), (K(2), "4"), (K(4), "2"), (K(6), "0")]);
    /// assert_eq!(&retain_map_order[..], &[0, 1, 7, 6, 2, 3, 5, 4]);
    /// # }
    /// ```
    ///
    /// [`Vec::retain_mut`]: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.retain_mut
    pub fn retain_map(&mut self, mut f: impl FnMut(T) -> Option<T>) {
        let mut i = 0;
        while i < self.0.len() {
            if let Some(new_item) = f(self.0.swap_remove(i)) {
                if i < self.0.len() {
                    let last = std::mem::replace(&mut self.0[i], new_item);
                    self.0.push(last);
                } else {
                    self.0.push(new_item);
                }
                i += 1;
            }
        }
        self.0.sort_unstable_by_key(<T as OrdVecKey>::get_key);
    }
}

impl<T: OrdVecKey> std::ops::Deref for OrdVec<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        &self.0
    }
}

impl<T: OrdVecKey> From<Vec<T>> for OrdVec<T> {
    fn from(value: Vec<T>) -> Self {
        Self::from_unsorted(value)
    }
}

impl<T: OrdVecKey> FromIterator<T> for OrdVec<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::from_unsorted(Vec::from_iter(iter))
    }
}

impl<T: OrdVecKey + Clone> Clone for OrdVec<T> {
    fn clone(&self) -> Self {
        OrdVec(self.0.clone())
    }
}

impl<T: OrdVecKey + Eq> Eq for OrdVec<T> {}

impl<T: OrdVecKey + PartialEq> PartialEq for OrdVec<T> {
    fn eq(&self, other: &Self) -> bool {
        self[..] == other[..]
    }
}

impl<T: OrdVecKey + Debug> Debug for OrdVec<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}

#[cfg(feature = "serde")]
impl<T: OrdVecKey + serde::Serialize> serde::Serialize for OrdVec<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T: OrdVecKey + serde::Deserialize<'de>> serde::Deserialize<'de> for OrdVec<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let vec = Vec::deserialize(deserializer)?;
        Ok(OrdVec::from_unsorted(vec))
    }
}
