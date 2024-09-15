use std::marker::PhantomData;

/// Ordered [`Vec<T>`] intended for fast lookup of items by key.
///
/// The key is typically stored inside `T` and extracted with the key function `K`.
/// (Different key functions may be created for the same data type.)
///
/// # Examples
///
/// ```
/// # use contiguous_collections::{OrdVec, OrdVecKey};
/// #[derive(Debug, Clone, PartialEq)]
/// struct User { uid: usize, name: String, zip: String };
///
/// struct UidKey;
/// impl OrdVecKey<User> for UidKey { type Key = usize; fn get_key(u: &User) -> &usize { return &u.uid; } }
/// struct ZipKey;
/// impl OrdVecKey<User> for ZipKey { type Key = str; fn get_key(u: &User) -> &str { return &u.zip; } }
///
/// let users = [
///     User { uid: 1, name: "Maya".into(), zip: "10030".into() },
///     User { uid: 0, name: "Ben".into(), zip: "11030".into() },
///     User { uid: 2, name: "Ariel".into(), zip: "11000".into() },
/// ];
/// let by_uid = users.iter().cloned().collect::<OrdVec<User, UidKey>>();
/// assert_eq!(Some(&users[0]), by_uid.get_by_key(&1));
/// let by_zip = users.iter().cloned().collect::<OrdVec<User, ZipKey>>();
/// assert_eq!(Some(&users[0]), by_zip.get_by_key("10030"));
/// ```
pub struct OrdVec<T, K: OrdVecKey<T>>(Vec<T>, PhantomData<K>);

/// Defines a key extraction function for type `T`.
/// If the key is not stored as part of the data, use a tuple of (key, data) as `T` and [`OrdVecKeyFst`] as `K`.
pub trait OrdVecKey<T> {
    type Key: Ord + ?Sized;
    fn get_key(item: &T) -> &Self::Key;
}

/// Provides a key extraction function that returns the first element of a two-element tuple.
pub struct OrdVecKeyFst;

impl<K: Ord, D> OrdVecKey<(K, D)> for OrdVecKeyFst {
    type Key = K;
    #[inline(always)]
    fn get_key(item: &(K, D)) -> &Self::Key {
        &item.0
    }
}

impl<T, K: OrdVecKey<T>> OrdVec<T, K> {
    /// Creates an empty [`OrdVec`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use contiguous_collections::{OrdVec, OrdVecKey, OrdVecKeyFst};
    /// let v: OrdVec<(u32, String), OrdVecKeyFst> = OrdVec::new();
    /// assert_eq!(0, v.len());
    /// ```
    pub const fn new() -> Self {
        OrdVec(Vec::new(), PhantomData)
    }

    /// Creates an [`OrdVec`] by taking ownership of the given vector
    /// and sorting it according to the key extraction function.
    ///
    /// Prefer this method if you want to be explicit about the
    /// underlying operations. Otherwise, use the more consice
    /// [`From<Vec<T>>`](struct.OrdVec.html#impl-From<Vec<T,+Global>>-for-OrdVec<T,+K>).
    ///
    /// # Examples
    ///
    /// ```
    /// # use contiguous_collections::{OrdVec, OrdVecKey, OrdVecKeyFst};
    /// let v = vec![(1, "B"), (0, "A"), (3, "D"), (2, "C")];
    /// let ov: OrdVec<_, OrdVecKeyFst> = OrdVec::new_from_unsorted(v);
    /// assert_eq!([(0, "A"), (1, "B"), (2, "C"), (3, "D")], ov[..]);
    /// ```
    pub fn new_from_unsorted(mut vec: Vec<T>) -> Self {
        vec.sort_unstable_by(|a, b| K::get_key(a).cmp(K::get_key(b)));
        assert!(
            vec.windows(2)
                .all(|pair| K::get_key(&pair[0]) != K::get_key(&pair[1])),
            "OrdVec must not contain duplicate keys"
        );
        OrdVec(vec, PhantomData)
    }

    /// Returns the number of elements in [`OrdVec`].
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn insert(&mut self, item: T) {
        let insert_idx = if let Some(last_item) = self.0.last() {
            let k = K::get_key(&item);
            if k <= K::get_key(&last_item) {
                match self.0.binary_search_by_key(&k, K::get_key) {
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

    pub fn get_by_key(&self, k: &<K as OrdVecKey<T>>::Key) -> Option<&T> {
        self.0
            .binary_search_by_key(&k, K::get_key)
            .ok()
            .map(|i| &self.0[i])
    }

    pub fn get_mut_by_key(&mut self, k: &<K as OrdVecKey<T>>::Key) -> Option<&mut T> {
        self.0
            .binary_search_by_key(&k, K::get_key)
            .ok()
            .map(|i| &mut self.0[i])
    }

    // pub fn get_index_by_key(&self, k: &<K as OrdVecKey<T>>::Key) -> Option<usize> {
    //     self.0.binary_search_by_key(k, K::get_key).ok()
    // }

    pub fn remove_by_key(&mut self, k: &<K as OrdVecKey<T>>::Key) -> Option<T> {
        self.0
            .binary_search_by_key(&k, K::get_key)
            .ok()
            .map(|i| self.0.remove(i))
    }

    /// Apply the function to each element of the [`OrdVec`] and depending on the return value:
    /// * Replace the element with the new value if the function returns Some(T),
    /// * Remove the element if the function returns None.
    ///
    /// The order of iteration may not follow the order of keys. Compare to
    /// [`Vec::retain_mut`](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.retain_mut).
    ///
    /// # Examples
    ///
    /// ```
    /// # use contiguous_collections::{OrdVec, OrdVecKey, OrdVecKeyFst};
    /// let mut ov: OrdVec<_, OrdVecKeyFst> = vec![
    ///     (0, "0"), (1, "1"), (2, "2"), (3, "3"),
    ///     (4, "4"), (5, "5"), (6, "6"), (7, "7")
    /// ].into();
    /// let mut retain_map_order = Vec::new();
    /// ov.retain_map(|(k, v)| {
    ///     retain_map_order.push(k);
    ///     if k % 2 == 0 { Some((6 - k, v)) } else { None }
    /// });
    /// assert_eq!(ov[..], [(0, "6"), (2, "4"), (4, "2"), (6, "0")]);
    /// assert_eq!(retain_map_order[..], [0, 1, 7, 6, 2, 3, 5, 4]);
    /// ```
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
        self.0
            .sort_unstable_by(|a, b| K::get_key(a).cmp(K::get_key(b)));
    }
}

/// Creates an [`OrdVec`] by taking ownership of the given vector
/// and sorting it according to the key extraction function.
///
/// Identical to [`new_from_unsorted`](struct.OrdVec.html#method.new_from_unsorted).
///
/// # Examples
///
/// ```
/// # use contiguous_collections::{OrdVec, OrdVecKey, OrdVecKeyFst};
/// let v = vec![(1, "B"), (0, "A"), (3, "D"), (2, "C")];
/// let ov: OrdVec<_, OrdVecKeyFst> = v.into();
/// assert_eq!([(0, "A"), (1, "B"), (2, "C"), (3, "D")], ov[..]);
/// ```
impl<T, K: OrdVecKey<T>> From<Vec<T>> for OrdVec<T, K> {
    fn from(value: Vec<T>) -> Self {
        Self::new_from_unsorted(value)
    }
}

/// Creates an [`OrdVec`] from an iterator.
///
/// # Examples
///
/// ```
/// # use contiguous_collections::{OrdVec, OrdVecKey, OrdVecKeyFst};
/// let iter = [(1, "B"), (0, "A"), (3, "D"), (2, "C")].into_iter();
/// let ov = iter.collect::<OrdVec<_, OrdVecKeyFst>>();
/// assert_eq!([(0, "A"), (1, "B"), (2, "C"), (3, "D")], ov[..]);
/// ```
impl<T, K: OrdVecKey<T>> FromIterator<T> for OrdVec<T, K> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::new_from_unsorted(Vec::from_iter(iter))
    }
}

/// Returns a slice of the underlying data, which is guaranteed
/// to be ordered according to the key extraction function.
impl<T, K: OrdVecKey<T>> std::ops::Deref for OrdVec<T, K> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        &self.0
    }
}

impl<T: Clone, K: OrdVecKey<T>> Clone for OrdVec<T, K> {
    fn clone(&self) -> Self {
        OrdVec(self.0.clone(), PhantomData)
    }
}

impl<T: Eq, K: OrdVecKey<T>> Eq for OrdVec<T, K> {}

impl<T: PartialEq, K: OrdVecKey<T>> PartialEq for OrdVec<T, K> {
    fn eq(&self, other: &Self) -> bool {
        self[..] == other[..]
    }
}

impl<T: std::fmt::Debug, K: OrdVecKey<T>> std::fmt::Debug for OrdVec<T, K> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self, f)
    }
}

#[cfg(feature = "serde")]
impl<T: serde::Serialize, K: OrdVecKey<T>> serde::Serialize for OrdVec<T, K> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T: serde::Deserialize<'de>, K: OrdVecKey<T>> serde::Deserialize<'de> for OrdVec<T, K> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let vec = Vec::deserialize(deserializer)?;
        Ok(OrdVec::new_from_unsorted(vec))
    }
}
