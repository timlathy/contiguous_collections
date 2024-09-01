use std::{
    iter::FusedIterator,
    ops::{Index, IndexMut},
};

/// Fixed-size two-dimensional array stored as a flat boxed slice in row-major order.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Array2<T> {
    data: Box<[T]>,
    num_columns: usize,
}

impl<T: Clone> Array2<T> {
    /// Creates an [`Array2`] of the given dimensions with all elements set to the given value.
    ///
    /// # Examples
    ///
    /// ```
    /// # use contiguous_collections::Array2;
    /// let a2 = Array2::new(4, 2, false);
    /// assert_eq!(a2.row(0), Some(&[false, false, false, false][..]));
    /// assert_eq!(a2.row(1), Some(&[false, false, false, false][..]));
    /// assert_eq!(a2.row(2), None);
    /// ```
    pub fn new(num_columns: usize, num_rows: usize, init_value: T) -> Self {
        Array2 {
            data: vec![init_value; num_columns * num_rows].into_boxed_slice(),
            num_columns,
        }
    }

    /// Creates an [`Array2`] from the given rows. All rows must have identical lengths.
    ///
    /// # Examples
    ///
    /// ```
    /// # use contiguous_collections::Array2;
    /// let a2 = Array2::new_from_rows(&[&[1, 2, 3, 4], &[5, 6, 7, 8]]);
    /// assert_eq!(a2.row(0), Some(&[1, 2, 3, 4][..]));
    /// assert_eq!(a2.row(1), Some(&[5, 6, 7, 8][..]));
    /// assert_eq!(a2.row(2), None);
    /// ```
    ///
    /// # Panics
    ///
    /// ```should_panic
    /// # use contiguous_collections::Array2;
    /// let a2 = Array2::new_from_rows(&[&[1, 2][..], &[1, 2, 3][..]]);
    /// ```
    pub fn new_from_rows(rows: &[&[T]]) -> Self {
        let num_columns = rows.first().map_or(0, |r| r.len());
        assert!(
            rows.iter().all(|r| r.len() == num_columns),
            "Rows must have identical lengths"
        );
        Array2 {
            data: rows.iter().flat_map(|r| r.iter().cloned()).collect(),
            num_columns,
        }
    }
}

impl<T> Array2<T> {
    /// Returns the number of columns (elements per row).
    ///
    /// # Examples
    ///
    /// ```
    /// # use contiguous_collections::Array2;
    /// let a2 = Array2::new_from_rows(&[&[1, 2, 3, 4], &[5, 6, 7, 8]]);
    /// assert_eq!(a2.num_columns(), 4);
    /// ```
    pub const fn num_columns(&self) -> usize {
        self.num_columns
    }

    /// Returns the number of rows.
    ///
    /// # Examples
    ///
    /// ```
    /// # use contiguous_collections::Array2;
    /// let a2 = Array2::new_from_rows(&[&[1, 2, 3, 4], &[5, 6, 7, 8]]);
    /// assert_eq!(a2.num_rows(), 2);
    /// ```
    pub const fn num_rows(&self) -> usize {
        self.data.len() / self.num_columns
    }

    /// Returns the number of elements across all rows.
    ///
    /// # Examples
    ///
    /// ```
    /// # use contiguous_collections::Array2;
    /// let a2 = Array2::new_from_rows(&[&[1, 2, 3, 4], &[5, 6, 7, 8]]);
    /// assert_eq!(a2.num_elements(), 8);
    /// ```
    pub const fn num_elements(&self) -> usize {
        self.data.len()
    }

    /// Returns a slice of the underlying buffer (row-major order).
    pub const fn elements(&self) -> &[T] {
        &self.data
    }

    /// Returns a mutable slice of the underlying buffer (row-major order).
    pub fn elements_mut(&mut self) -> &mut [T] {
        &mut self.data
    }

    /// Returns a slice of the underlying buffer with elements of the row
    /// at the given index, or None if the row index is out of bounds.
    pub fn row(&self, row_index: usize) -> Option<&[T]> {
        let start = row_index * self.num_columns;
        let end = (row_index + 1) * self.num_columns;
        if end <= self.data.len() {
            Some(&self.data[start..end])
        } else {
            None
        }
    }

    /// Returns a mutable slice of the underlying buffer with elements
    /// of the row at the given index, or None if the row index is out of bounds.
    pub fn row_mut(&mut self, row_index: usize) -> Option<&mut [T]> {
        let start = row_index * self.num_columns;
        let end = (row_index + 1) * self.num_columns;
        if end <= self.data.len() {
            Some(&mut self.data[start..end])
        } else {
            None
        }
    }

    /// Returns an iterator over rows. Each item is a slice of all elements
    /// in the corresponding row.
    ///
    /// # Examples
    ///
    /// ```
    /// # use contiguous_collections::Array2;
    /// let a2 = Array2::new_from_rows(&[&[1, 2, 3, 4], &[5, 6, 7, 8]]);
    /// let mut rows_iter = a2.rows();
    /// let row0 = rows_iter.next().unwrap();
    /// assert_eq!(row0, &[1, 2, 3, 4][..]);
    /// let row1 = rows_iter.next().unwrap();
    /// assert_eq!(row1.split_at(2), (&[5, 6][..], &[7, 8][..]));
    /// assert!(rows_iter.next().is_none());
    /// ```
    pub fn rows(
        &self,
    ) -> impl ExactSizeIterator<Item = &[T]> + DoubleEndedIterator + FusedIterator {
        self.data.chunks(self.num_columns)
    }
}

impl<T> Index<usize> for Array2<T> {
    type Output = [T];

    /// Returns a slice of the underlying buffer with elements of the row at the given index.
    ///
    /// Panics if the index is out of bounds. See [`row`](struct.Array2.html#method.row) for a non-panicking version.
    fn index(&self, row_index: usize) -> &Self::Output {
        self.row(row_index)
            .unwrap_or_else(|| panic!("Row index {} is out of bounds", row_index))
    }
}

impl<T> IndexMut<usize> for Array2<T> {
    /// Returns a mutable slice of the underlying buffer with elements of the row at the given index.
    ///
    /// Panics if the index is out of bounds. See [`row_mut`](struct.Array2.html#method.row) for a non-panicking version.
    fn index_mut(&mut self, row_index: usize) -> &mut Self::Output {
        self.row_mut(row_index)
            .unwrap_or_else(|| panic!("Row index {} is out of bounds", row_index))
    }
}
