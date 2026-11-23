use std::collections::HashMap;

#[derive(Default)]
pub struct YearsPage {
    pub library_id: Vec<i64>,
    pub recent_items: bool,
    pub choice: HashMap<i64, Option<i64>>,
    pub since: Option<i64>,
    pub start: HashMap<i64, usize>,
}

impl YearsPage {
    /// Identifie les bibliothèques affichées, pour renouveler la page quand elles changent.
    pub fn key(&self) -> String {
        self.library_id.iter().map(i64::to_string).collect::<Vec<_>>().join(",")
    }
}
