//! Les bibliothèques actives : une seule, ou plusieurs cochées ensemble.

/// Un ensemble de bibliothèques, lié à SQLite comme un tableau JSON.
///
/// Les requêtes de lecture écrivent `library_id IN (SELECT value FROM json_each(?))`
/// au lieu de `library_id = ?`. Un `i64` seul se convertit, les appelants d'une
/// bibliothèque unique restent donc tels quels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryScope(Vec<i64>);

impl LibraryScope {
    /// Le tableau JSON à lier : `[1,2,3]`.
    pub fn json(&self) -> String {
        let ids: Vec<String> = self.0.iter().map(i64::to_string).collect();
        format!("[{}]", ids.join(","))
    }

    pub fn ids(&self) -> &[i64] {
        &self.0
    }
}

impl From<i64> for LibraryScope {
    fn from(id: i64) -> Self {
        Self(vec![id])
    }
}

impl From<Vec<i64>> for LibraryScope {
    fn from(ids: Vec<i64>) -> Self {
        Self(ids)
    }
}

impl From<&[i64]> for LibraryScope {
    fn from(ids: &[i64]) -> Self {
        Self(ids.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::LibraryScope;

    #[test]
    fn one_id_is_a_one_element_array() {
        assert_eq!(LibraryScope::from(7).json(), "[7]");
    }

    #[test]
    fn several_ids_keep_their_order() {
        assert_eq!(LibraryScope::from(vec![3, 1, 2]).json(), "[3,1,2]");
    }

    #[test]
    fn no_id_matches_nothing() {
        assert_eq!(LibraryScope::from(Vec::new()).json(), "[]");
    }
}
