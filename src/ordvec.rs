use std::marker::PhantomData;

/// Ordered [`Vec<T>`] intended for fast lookup of items by key.
///
/// The key is stored inside `T` and extracted with the key function `K`.
/// Different key functions may be created for the same `T`: see [`OrdVecKey`].
/// A predefined key function for (K,V) tuples is available as [`OrdVecKeyFst`].
///
/// Restrictions:
/// * Multiple items with the same key are not allowed and will result
/// in a panic on construction.
/// * The items must not be modified in a way that changes their key
/// ordering relative to other items. To modify the keys safely, use
/// [`retain_map`](struct.OrdVec.html#method.retain_map).
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
/// assert_eq!(by_uid.get_by_key(&1), Some(&users[0]));
/// let by_zip = users.iter().cloned().collect::<OrdVec<User, ZipKey>>();
/// assert_eq!(by_zip.get_by_key("10030"), Some(&users[0]));
/// ```
pub struct OrdVec<T, K: OrdVecKey<T>>(Vec<T>, PhantomData<K>);

/// Trait for [`OrdVec`] key extraction functions.
/// If the key is not stored alongside data, use a tuple of (key, data) as `T` and [`OrdVecKeyFst`] as `K`.
pub trait OrdVecKey<T> {
    /// The type of keys extracted from values of type `T`. Must implement [`Ord`].
    type Key: Ord + ?Sized;
    /// Extracts the key from a value of type `T`.
    fn get_key(item: &T) -> &Self::Key;
}

/// Key extraction function for [`OrdVec`] that returns the first element of a two-element tuple.
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
    /// let ov: OrdVec<(u32, String), OrdVecKeyFst> = OrdVec::new();
    /// assert_eq!(ov.len(), 0);
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
    /// assert_eq!(ov[..], [(0, "A"), (1, "B"), (2, "C"), (3, "D")]);
    /// ```
    ///
    /// # Panics
    ///
    /// ```should_panic
    /// # use contiguous_collections::{OrdVec, OrdVecKey, OrdVecKeyFst};
    /// let duplicate_keys = vec![(0, "A"), (0, "B")];
    /// let v: OrdVec<_, OrdVecKeyFst> = OrdVec::new_from_unsorted(duplicate_keys);
    /// ```
    pub fn new_from_unsorted(mut vec: Vec<T>) -> Self {
        vec.sort_unstable_by(|a, b| K::get_key(a).cmp(K::get_key(b)));
        assert!(
            vec.windows(2)
                .all(|pair| K::get_key(&pair[0]) != K::get_key(&pair[1])),
            "Duplicate keys are not allowed"
        );
        OrdVec(vec, PhantomData)
    }

    /// Returns the number of items in [`OrdVec`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use contiguous_collections::{OrdVec, OrdVecKey, OrdVecKeyFst};
    /// let ov: OrdVec<_, OrdVecKeyFst> = vec![(1, "B"), (0, "A")].into();
    /// assert_eq!(ov.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the [`OrdVec`] contains no items.
    ///
    /// # Examples
    ///
    /// ```
    /// # use contiguous_collections::{OrdVec, OrdVecKey, OrdVecKeyFst};
    /// let mut ov: OrdVec<(u32, &str), OrdVecKeyFst> = OrdVec::new();
    /// assert!(ov.is_empty());
    /// ov.insert((1, "A"));
    /// assert!(!ov.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Inserts a new item into [`OrdVec`].
    /// Panics if there is an existing item with the same key.
    ///
    /// # Examples
    ///
    /// ```
    /// # use contiguous_collections::{OrdVec, OrdVecKey, OrdVecKeyFst};
    /// let mut ov: OrdVec<(u32, &str), OrdVecKeyFst> = OrdVec::new();
    /// ov.insert((5, "B"));
    /// ov.insert((3, "A"));
    /// ov.insert((7, "C"));
    /// assert_eq!(ov[..], [(3, "A"), (5, "B"), (7, "C")]);
    /// ```
    ///
    /// # Panics
    ///
    /// ```should_panic
    /// # use contiguous_collections::{OrdVec, OrdVecKey, OrdVecKeyFst};
    /// let mut ov: OrdVec<(u32, &str), OrdVecKeyFst> = OrdVec::new();
    /// ov.insert((5, "B"));
    /// ov.insert((5, "A"));
    /// ```
    pub fn insert(&mut self, item: T) {
        let insert_idx = if let Some(last_item) = self.0.last() {
            let k = K::get_key(&item);
            if k <= K::get_key(last_item) {
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

    /// Looks up an item by key.
    /// See also [`get_mut_by_key`](struct.OrdVec.html#method.get_mut_by_key).
    ///
    /// # Examples
    ///
    /// ```
    /// # use contiguous_collections::{OrdVec, OrdVecKey, OrdVecKeyFst};
    /// let ov: OrdVec<_, OrdVecKeyFst> = vec![(1, "B"), (0, "A")].into();
    /// assert_eq!(ov.get_by_key(&0), Some(&(0, "A")));
    /// assert_eq!(ov.get_by_key(&2), None);
    /// ```
    pub fn get_by_key(&self, k: &<K as OrdVecKey<T>>::Key) -> Option<&T> {
        self.get_index_by_key(k).map(|i| &self.0[i])
    }

    /// Returns a mutable reference to an item looked up by key.
    ///
    /// Warning: the behavior of the collection is undefined if the item's key
    /// is changed in a way that affects its ordering relative to other items.
    ///
    /// # Examples
    ///
    /// ```
    /// # use contiguous_collections::{OrdVec, OrdVecKey, OrdVecKeyFst};
    /// let mut ov: OrdVec<_, OrdVecKeyFst> = vec![(1, "B"), (0, "A")].into();
    /// assert_eq!(ov.get_mut_by_key(&0), Some(&mut (0, "A")));
    /// assert_eq!(ov.get_mut_by_key(&2), None);
    /// ```
    pub fn get_mut_by_key(&mut self, k: &<K as OrdVecKey<T>>::Key) -> Option<&mut T> {
        self.get_index_by_key(k).map(|i| &mut self.0[i])
    }

    /// Returns the index of the item with the given key
    /// in the underlying ordered array.
    ///
    /// # Examples
    ///
    /// ```
    /// # use contiguous_collections::{OrdVec, OrdVecKey, OrdVecKeyFst};
    /// let ov: OrdVec<_, OrdVecKeyFst> = vec![(20, "B"), (10, "A")].into();
    /// assert_eq!(ov.get_index_by_key(&0), None);
    /// assert_eq!(ov.get_index_by_key(&10), Some(0));
    /// assert_eq!(ov.get_index_by_key(&10).map(|i| ov[i]), Some((10, "A")));
    /// ```
    pub fn get_index_by_key(&self, k: &<K as OrdVecKey<T>>::Key) -> Option<usize> {
        self.0.binary_search_by_key(&k, K::get_key).ok()
    }

    /// Removes an item with the given key from [`OrdVec`] and returns it,
    /// or None if such an item is not found.
    ///
    /// # Examples
    ///
    /// ```
    /// # use contiguous_collections::{OrdVec, OrdVecKey, OrdVecKeyFst};
    /// let mut ov: OrdVec<_, OrdVecKeyFst> = vec![(20, "B"), (10, "A")].into();
    /// assert_eq!(ov.remove_by_key(&10), Some((10, "A")));
    /// assert_eq!(ov.remove_by_key(&10), None);
    /// ```
    pub fn remove_by_key(&mut self, k: &<K as OrdVecKey<T>>::Key) -> Option<T> {
        self.0
            .binary_search_by_key(&k, K::get_key)
            .ok()
            .map(|i| self.0.remove(i))
    }

    /// Apply the function to each [`OrdVec`] item and depending on the return value:
    /// * Replace the item with the new value if the function returns Some(T),
    /// * Remove the item if the function returns None.
    ///
    /// If the new value has a different key from the old item, its position in
    /// the [`OrdVec`] will change accordingly.
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
/// assert_eq!(ov[..], [(0, "A"), (1, "B"), (2, "C"), (3, "D")]);
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
/// assert_eq!(ov[..], [(0, "A"), (1, "B"), (2, "C"), (3, "D")]);
/// ```
impl<T, K: OrdVecKey<T>> FromIterator<T> for OrdVec<T, K> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::new_from_unsorted(Vec::from_iter(iter))
    }
}

/// Creates an empty [`OrdVec`].
impl<T, K: OrdVecKey<T>> Default for OrdVec<T, K> {
    fn default() -> Self {
        Self::new()
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
