use crate::models::shared::ListWindow;
use crate::service::library::composer::ComposerView;

pub struct ArtistsPage {
    pub library_id: Vec<i64>,
    pub sort: String,
    pub search: String,
    pub search_term: String,
    pub(crate) search_gen: u64,
    pub without_portrait: bool,
    /// Les éléments bâtis de la liste filtrée.
    pub window: ListWindow,
    pub composers: Vec<ComposerView>,
}

impl ArtistsPage {
    /// Identifie les bibliothèques affichées, pour renouveler la page quand elles changent.
    pub fn key(&self) -> String {
        self.library_id.iter().map(i64::to_string).collect::<Vec<_>>().join(",")
    }
}
