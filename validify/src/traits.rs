use indexmap::{IndexMap, IndexSet};
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::hash::Hash;

/// Trait to implement if one wants to make the `length` validator
/// work for more types
pub trait HasLen {
    fn length(&self) -> u64;
}

impl HasLen for String {
    fn length(&self) -> u64 {
        self.chars().count() as u64
    }
}

impl<'a> HasLen for &'a String {
    fn length(&self) -> u64 {
        self.chars().count() as u64
    }
}

impl<'a> HasLen for &'a str {
    fn length(&self) -> u64 {
        self.chars().count() as u64
    }
}

impl<'a> HasLen for Cow<'a, str> {
    fn length(&self) -> u64 {
        self.len() as u64
    }
}

impl<T> HasLen for Vec<T> {
    fn length(&self) -> u64 {
        self.len() as u64
    }
}

impl<'a, T> HasLen for &'a Vec<T> {
    fn length(&self) -> u64 {
        self.len() as u64
    }
}

impl<T> HasLen for &[T] {
    fn length(&self) -> u64 {
        self.len() as u64
    }
}

impl<T, const N: usize> HasLen for [T; N] {
    fn length(&self) -> u64 {
        N as u64
    }
}

impl<T, const N: usize> HasLen for &[T; N] {
    fn length(&self) -> u64 {
        N as u64
    }
}

impl<'a, K, V, S> HasLen for &'a HashMap<K, V, S> {
    fn length(&self) -> u64 {
        self.len() as u64
    }
}

impl<K, V, S> HasLen for HashMap<K, V, S> {
    fn length(&self) -> u64 {
        self.len() as u64
    }
}

impl<'a, T, S> HasLen for &'a HashSet<T, S> {
    fn length(&self) -> u64 {
        self.len() as u64
    }
}

impl<T, S> HasLen for HashSet<T, S> {
    fn length(&self) -> u64 {
        self.len() as u64
    }
}

impl<'a, K, V> HasLen for &'a BTreeMap<K, V> {
    fn length(&self) -> u64 {
        self.len() as u64
    }
}

impl<K, V> HasLen for BTreeMap<K, V> {
    fn length(&self) -> u64 {
        self.len() as u64
    }
}

impl<'a, T> HasLen for &'a BTreeSet<T> {
    fn length(&self) -> u64 {
        self.len() as u64
    }
}

impl<T> HasLen for BTreeSet<T> {
    fn length(&self) -> u64 {
        self.len() as u64
    }
}

impl<'a, K, V> HasLen for &'a IndexMap<K, V> {
    fn length(&self) -> u64 {
        self.len() as u64
    }
}

impl<K, V> HasLen for IndexMap<K, V> {
    fn length(&self) -> u64 {
        self.len() as u64
    }
}

impl<'a, T> HasLen for &'a IndexSet<T> {
    fn length(&self) -> u64 {
        self.len() as u64
    }
}

impl<T> HasLen for IndexSet<T> {
    fn length(&self) -> u64 {
        self.len() as u64
    }
}

/// Trait to implement if one wants to make the `contains` validator
/// work for more types
pub trait Contains {
    type Needle<'a>
    where
        Self: 'a;
    fn has_element(&self, needle: Self::Needle<'_>) -> bool;
}

impl<T> Contains for Vec<T>
where
    T: PartialEq,
{
    type Needle<'a> = &'a T where Self: 'a;
    fn has_element(&self, needle: Self::Needle<'_>) -> bool {
        self.iter().any(|a| a == needle)
    }
}

impl<T> Contains for &Vec<T>
where
    T: PartialEq,
{
    type Needle<'a> = &'a T where Self: 'a;
    fn has_element<'a>(&'a self, needle: Self::Needle<'a>) -> bool {
        self.iter().any(|a| a == needle)
    }
}

impl<T> Contains for &[T]
where
    T: PartialEq,
{
    type Needle<'a> = &'a T where Self: 'a;

    fn has_element<'a>(&'a self, needle: Self::Needle<'a>) -> bool {
        self.contains(needle)
    }
}

impl<T, V> Contains for HashMap<T, V>
where
    T: PartialEq + Eq + Hash,
{
    type Needle<'a> = &'a T where Self: 'a;
    fn has_element<'a>(&'a self, needle: Self::Needle<'a>) -> bool {
        self.contains_key(needle)
    }
}

impl<T, V> Contains for &HashMap<T, V>
where
    T: PartialEq + Eq + Hash,
{
    type Needle<'a> = &'a T where Self: 'a;
    fn has_element<'a>(&'a self, needle: Self::Needle<'a>) -> bool {
        self.contains_key(needle)
    }
}

impl Contains for String {
    type Needle<'a> = &'a str;
    fn has_element(&self, needle: &str) -> bool {
        self.contains(needle)
    }
}

impl Contains for &String {
    type Needle<'a> = &'a str where Self: 'a;
    fn has_element(&self, needle: &str) -> bool {
        self.contains(needle)
    }
}

impl Contains for &str {
    type Needle<'a> = &'a str where Self: 'a;
    fn has_element(&self, needle: &str) -> bool {
        self.contains(needle)
    }
}

impl Contains for Cow<'_, str> {
    type Needle<'a> = &'a str where Self: 'a;
    fn has_element(&self, needle: &str) -> bool {
        self.contains(needle)
    }
}
