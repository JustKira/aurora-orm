use std::collections::{BTreeSet, HashMap};

/// Where a key sits relative to two schemas being compared. Lets callers
/// match on `Added` / `Removed` / `Change` instead of decoding `Option`
/// tuples like `(None, Some)`.
pub(crate) enum Diff<'a, T> {
    /// Key is only on the new side — the item appeared.
    Added(&'a T),
    /// Key is only on the prev side — the item disappeared.
    Removed(&'a T),
    /// Key exists on both sides. Whether it actually *changed* is up to the
    /// caller to decide by comparing `(prev, new)`.
    Change(&'a T, &'a T),
}

/// Yields `(key, change)` for every key present in either map, sorted so
/// the resulting op order is deterministic. Used at every diff level
/// (tables, fields, indexes, analyzers).
pub(crate) fn diff_by_key<'a, K, V>(
    prev: &'a HashMap<K, &'a V>,
    new: &'a HashMap<K, &'a V>,
) -> impl Iterator<Item = (K, Diff<'a, V>)> + 'a
where
    K: Ord + Copy + Eq + std::hash::Hash,
{
    let keys: BTreeSet<K> = prev.keys().chain(new.keys()).copied().collect();
    keys.into_iter().map(move |k| {
        let change = match (prev.get(&k), new.get(&k)) {
            (None, Some(&n)) => Diff::Added(n),
            (Some(&p), None) => Diff::Removed(p),
            (Some(&p), Some(&n)) => Diff::Change(p, n),
            // Impossible: `k` was drawn from the union of both maps' keys.
            (None, None) => unreachable!("key came from union of both maps"),
        };
        (k, change)
    })
}
