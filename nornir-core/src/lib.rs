use std::collections::BTreeMap;
use natord::compare;
use std::cmp::Ordering;
use std::ops::Deref;
use std::fmt;
use pyo3::prelude::*;
pub mod inventory;


/// A wrapper type for strings that implements natural (alphanumeric) ordering.
///
/// `NatString` wraps a `String` and provides custom ordering behavior where
/// numeric portions of strings are compared numerically rather than lexicographically.
/// For example, "item2" will be ordered before "item10" (natural order) instead of
/// after it (lexicographic order).
///
/// This type is typically used as a key in ordered collections like `BTreeMap`
/// when natural sorting of string keys is desired.
///
/// # Examples
///
/// ```
/// # use nornir_core::NatString;
/// let s1 = NatString::new("file2".to_string());
/// let s2 = NatString::new("file10".to_string());
/// assert!(s1 < s2);
/// // s1 < s2 in natural order (2 < 10)
/// ```
#[derive(PartialEq, Eq)]
pub struct NatString(String);

impl NatString {
    pub fn new(s: String) -> Self {
        NatString(s)
    }
}

impl fmt::Debug for NatString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Use write! to format the fields directly without the struct wrapper
        write!(f, "{}", self.0)
    }
}
impl Ord for NatString {
    fn cmp(&self, other: &Self) -> Ordering {
        compare(&self.0, &other.0)
    }
}

impl PartialOrd for NatString {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// A wrapper around `BTreeMap` that uses natural ordering for string keys.
///
/// `MyTree` provides a map data structure where keys are automatically sorted
/// using natural (alphanumeric) ordering instead of lexicographic ordering.
/// For example, "host2" will come before "host10" in the natural order.
///
/// ## Fields
///
/// * `0` - The underlying `BTreeMap` with `NatString` keys and `String` values.
///
/// ## Examples
///
/// ```
/// # use nornir_core::CustomTreeMap;
/// let mut tree = CustomTreeMap::new();
/// tree.insert("host1", "value1".to_string());
/// tree.insert("host10", "value10".to_string());
/// // Keys will be ordered naturally: host1, host10
/// ```
pub struct CustomTreeMap<V>(BTreeMap<NatString, V>);


impl<V> Deref for CustomTreeMap<V> {
    // Specify the Target type, which is a reference to T
    type Target = BTreeMap<NatString, V>;

    // Implement the deref method, returning an immutable reference
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<V: fmt::Debug> fmt::Debug for CustomTreeMap<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            // pretty print the map using the debug_struct builder pattern
            f.debug_struct("CustomTreeMap")
            .field("BTreeMap", &self.0)
            .finish()
        } else {
            // Use write! to format the fields directly without the struct wrapper
            write!(f, "{:?}", self.0)
        }
    }
}

impl<V: fmt::Display + fmt::Debug> fmt::Display for CustomTreeMap<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Use write! to format the fields directly without the struct wrapper
        write!(f, "{:?}", self.0)
    }
}

impl<V> CustomTreeMap<V> {
    pub fn new() -> Self {
        CustomTreeMap(BTreeMap::new())
    }

    pub fn insert(&mut self, key: &str, value: V) {
        self.0.insert(NatString::new(key.to_string()), value);
    }
    
    pub fn get(&self, key: &str) -> Option<&V> {
        self.0.get(&NatString::new(key.to_string()))
    }
    
    pub fn get_mut(&mut self, key: &str) -> Option<&mut V> {
        self.0.get_mut(&NatString::new(key.to_string()))
    }

    pub fn remove(&mut self, key: &str) -> Option<V> {
        self.0.remove(&NatString::new(key.to_string()))
    }
    
    pub fn len(&self) -> usize {
        self.0.len()
    }
}



/// Formats the sum of two numbers as string.
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

/// A Python module implemented in Rust.
#[pymodule]
fn nornir_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nat_string_ordering() {
        let s1 = NatString::new("file2".to_string());
        let s2 = NatString::new("file10".to_string());
        assert!(s1 < s2);
    }

    #[test]
    fn test_custom_tree_map_ordering() {
        let mut tree = CustomTreeMap::new();
        tree.insert("host1", "one".to_string());
        tree.insert("host2", "two".to_string());
        tree.insert("host10", "three10".to_string());
        tree.insert("host4", "four1".to_string());
        tree.insert("host100", "100".to_string());
        assert_eq!(tree.get("host1").unwrap(), "one");
        assert_eq!(tree.get("host10").unwrap(), "three10");
    }
}