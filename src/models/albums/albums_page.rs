use crate::models::shared::ListWindow;
pub struct AlbumsPage {
    pub library_id: Vec<i64>,
    pub sort: String,
    pub search: String,
    pub search_term: String,
    pub(crate) search_gen: u64,
    pub hires: bool,
    pub compilations: bool,
    pub without_cover: bool,
    pub genre: Option<String>,
    pub year: Option<i32>,
    /// Les éléments bâtis de la liste filtrée.
    pub window: ListWindow,
}

impl AlbumsPage {
    /// Identifie les bibliothèques affichées, pour renouveler la page quand elles changent.
    pub fn key(&self) -> String {
        self.library_id.iter().map(i64::to_string).collect::<Vec<_>>().join(",")
    }
}
