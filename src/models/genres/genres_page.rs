use crate::models::shared::ListWindow;

pub struct GenresPage {
    pub library_id: Vec<i64>,
    pub sort: String,
    pub search: String,
    pub main_items: bool,
    pub without_genre: bool,
    /// Les éléments bâtis de la liste filtrée.
    pub window: ListWindow,
}

impl GenresPage {
    /// Identifie les bibliothèques affichées, pour renouveler la page quand elles changent.
    pub fn key(&self) -> String {
        self.library_id.iter().map(i64::to_string).collect::<Vec<_>>().join(",")
    }
}
