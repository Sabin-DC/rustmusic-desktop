use uuid::Uuid;

use crate::entity::library::library_artist::LibraryArtist;
use crate::entity::library::library_artist::LibraryArtistCreate;
use crate::query::artist::artist_detail_row::ArtistDetailRow;
use crate::query::artist::artist_list_row::ArtistListRow;

pub struct LibraryArtistRepository;

impl LibraryArtistRepository {

    pub async fn insert_library_artist<'e, E>(
        exec: E,
        artist_data: LibraryArtistCreate
    ) -> Result<LibraryArtist, sqlx::Error>
    where
        E: sqlx::Executor<'e, Database = sqlx::Sqlite>
    {

        let id: String = Uuid::new_v4().to_string();

        let artist: LibraryArtist = sqlx::query_as::<_, LibraryArtist>(
            r#"
            INSERT INTO library_artists (
                id,
                library_id,
                artist_id,
                created_at,
                updated_at
            )
            VALUES (?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)

            ON CONFLICT(library_id, artist_id)
            DO UPDATE SET
                id = library_artists.id

            RETURNING *
            "#
        )
        .bind(&id)
        .bind(&artist_data.library_id)
        .bind(&artist_data.artist_id)
        .fetch_one(exec)
        .await?;

        Ok(artist)
    }

    pub async fn find_all_artists_by_library_id<'e, E>(
        exec: E,
        library_id: impl Into<crate::repository::library::scope::LibraryScope>,
    ) -> Result<Vec<ArtistListRow>, sqlx::Error>
    where
        E: sqlx::Executor<'e, Database = sqlx::Sqlite>
    {

        let artists: Vec<ArtistListRow> = sqlx::query_as::<_, ArtistListRow>(
            r#"
            SELECT
                a.id    AS id,
                a.name  AS name,
                COALESCE(album_stats.cnt, 0) AS total_albums,
                COALESCE(track_stats.cnt, 0) AS total_tracks,
                COALESCE(track_stats.dur, 0.0) AS total_duration,
                COALESCE(NULLIF(a.image_url, ''), cover.thumbnail_path) AS thumbnail_path

            FROM library_artists lart
            INNER JOIN artists a ON a.id = lart.artist_id

            -- Nombre d'albums (1 sous-requête pré-agrégée)
            LEFT JOIN (
                SELECT artist_id, COUNT(*) AS cnt
                FROM library_albums WHERE library_id IN (SELECT value FROM json_each(?1))
                GROUP BY artist_id
            ) album_stats ON album_stats.artist_id = a.id

            -- Les deux rattachements : `library_tracks.artist_id` (principal)
            -- et `library_track_artists` (tous). `UNION` et non `UNION ALL`,
            -- le principal figure souvent dans les deux.
            LEFT JOIN (
                SELECT artist_id, COUNT(*) AS cnt, COALESCE(SUM(duration), 0.0) AS dur
                FROM (
                    SELECT lt.artist_id AS artist_id, lt.id AS track_id, lt.duration AS duration
                    FROM library_tracks lt
                    WHERE lt.library_id IN (SELECT value FROM json_each(?1)) AND lt.artist_id IS NOT NULL

                    UNION

                    SELECT lta.artist_id, lt.id, lt.duration
                    FROM library_track_artists lta
                    INNER JOIN library_tracks lt ON lt.id = lta.library_track_id
                    WHERE lta.library_id IN (SELECT value FROM json_each(?1))
                )
                GROUP BY artist_id
            ) track_stats ON track_stats.artist_id = a.id

            -- Première cover trouvée
            LEFT JOIN (
                SELECT lt.artist_id, lc.thumbnail_path
                FROM library_tracks lt
                INNER JOIN library_cache lc ON lc.id = lt.cache_id
                WHERE lt.library_id IN (SELECT value FROM json_each(?1)) AND lc.thumbnail_path IS NOT NULL
                GROUP BY lt.artist_id
            ) cover ON cover.artist_id = a.id

            WHERE lart.library_id IN (SELECT value FROM json_each(?1))
              -- Écarte ceux qui n'ont plus rien ici : une cascade efface les
              -- pistes mais laisse la ligne `library_artists`.
              AND (COALESCE(album_stats.cnt, 0) > 0 OR COALESCE(track_stats.cnt, 0) > 0)
            GROUP BY a.id
            ORDER BY a.name COLLATE NOCASE ASC
            "#
        )
        .bind(library_id.into().json())
        .fetch_all(exec)
        .await?;

        Ok(artists)
    }

    /// Artistes similaires : même genre, même library, excluant l'artiste donné
    /// Requête légère : pas de stats tracks/duration, juste nom + image + nb albums
    pub async fn find_similar_artists<'e, E>(
        exec: E,
        library_id: impl Into<crate::repository::library::scope::LibraryScope> + Clone,
        artist_id: &str,
        limit: i64,
    ) -> Result<Vec<ArtistListRow>, sqlx::Error>
    where
        E: sqlx::Executor<'e, Database = sqlx::Sqlite>
    {
        sqlx::query_as::<_, ArtistListRow>(
            r#"
            SELECT
                a.id    AS id,
                a.name  AS name,
                COUNT(DISTINCT la_all.id) AS total_albums,
                (SELECT COUNT(DISTINCT lta_s.library_track_id) FROM library_track_artists lta_s
                 WHERE lta_s.artist_id = a.id AND lta_s.library_id IN (SELECT value FROM json_each(?1))
                ) AS total_tracks,
                0.0 AS total_duration,
                COALESCE(NULLIF(a.image_url, ''), (
                    SELECT lc.thumbnail_path
                    FROM library_tracks lt
                    INNER JOIN library_cache lc ON lc.id = lt.cache_id
                    WHERE lt.artist_id = a.id AND lt.library_id IN (SELECT value FROM json_each(?1)) AND lc.thumbnail_path IS NOT NULL
                    LIMIT 1
                )) AS thumbnail_path
            FROM library_albums la
            INNER JOIN artists a ON a.id = la.artist_id
            INNER JOIN library_artists lart ON lart.artist_id = a.id AND lart.library_id IN (SELECT value FROM json_each(?1))
            LEFT JOIN library_albums la_all ON la_all.artist_id = a.id AND la_all.library_id IN (SELECT value FROM json_each(?1))
            WHERE la.library_id IN (SELECT value FROM json_each(?1))
              AND a.id != ?2
              AND LOWER(la.genre) IN (
                SELECT DISTINCT LOWER(la2.genre)
                FROM library_albums la2
                WHERE la2.artist_id = ?2 AND la2.library_id IN (SELECT value FROM json_each(?1)) AND la2.genre IS NOT NULL
              )
            GROUP BY a.id
            ORDER BY RANDOM()
            LIMIT ?3
            "#
        )
        .bind(library_id.clone().into().json())
        .bind(artist_id)
        .bind(limit)
        .fetch_all(exec)
        .await
    }

    pub async fn find_artist_by_id<'e, E>(
        exec: E,
        library_artist_id: String,
    ) -> Result<ArtistDetailRow, sqlx::Error>
    where
        E: sqlx::Executor<'e, Database = sqlx::Sqlite>
    {

        let artist: ArtistDetailRow = sqlx::query_as::<_, ArtistDetailRow>(
            r#"
            SELECT
                a.id    AS id,
                a.name  AS name,

                (SELECT COUNT(*) FROM library_albums la2
                 WHERE la2.artist_id = a.id
                ) AS total_albums,

                (SELECT COUNT(DISTINCT lt2.id) FROM library_tracks lt2
                 LEFT JOIN library_track_artists lta2 ON lta2.library_track_id = lt2.id AND lta2.artist_id = a.id
                 WHERE lt2.artist_id = a.id OR lta2.artist_id IS NOT NULL
                ) AS total_tracks,

                (SELECT COALESCE(SUM(lt3.duration), 0.0) FROM library_tracks lt3
                 LEFT JOIN library_track_artists lta3 ON lta3.library_track_id = lt3.id AND lta3.artist_id = a.id
                 WHERE lt3.artist_id = a.id OR lta3.artist_id IS NOT NULL
                ) AS total_duration,

                COALESCE(NULLIF(a.image_url, ''), (SELECT lc4.thumbnail_path FROM library_tracks lt4
                 INNER JOIN library_cache lc4 ON lc4.id = lt4.cache_id
                 LEFT JOIN library_track_artists lta4 ON lta4.library_track_id = lt4.id AND lta4.artist_id = a.id
                 WHERE (lt4.artist_id = a.id OR lta4.artist_id IS NOT NULL)
                 AND lc4.thumbnail_path IS NOT NULL
                 LIMIT 1
                )) AS thumbnail_path

            FROM library_artists lart
            INNER JOIN artists a ON a.id = lart.artist_id
            WHERE lart.id = ?1 OR a.id = ?1
            LIMIT 1
            "#
        )
        .bind(library_artist_id)
        .fetch_one(exec)
        .await?;

        Ok(artist)
    }

    /// Albums de chaque artiste de la bibliothèque, recomptés en un seul passage.
    pub async fn recount_albums<'e, E>(exec: E, library_id: i64) -> Result<(), sqlx::Error>
    where
        E: sqlx::Executor<'e, Database = sqlx::Sqlite>,
    {
        sqlx::query(
            "UPDATE library_artists SET total_albums = COALESCE((
                SELECT c.n FROM (
                    SELECT library_id, artist_id, COUNT(*) AS n
                    FROM library_albums GROUP BY library_id, artist_id
                ) c
                WHERE c.library_id = library_artists.library_id
                  AND c.artist_id = library_artists.artist_id), 0)
             WHERE library_id = ?",
        )
        .bind(library_id)
        .execute(exec)
        .await
        .map(|_| ())
    }

    /// Pistes de chaque artiste de la bibliothèque, recomptées en un seul passage.
    pub async fn recount_tracks<'e, E>(exec: E, library_id: i64) -> Result<(), sqlx::Error>
    where
        E: sqlx::Executor<'e, Database = sqlx::Sqlite>,
    {
        sqlx::query(
            "UPDATE library_artists SET total_tracks = COALESCE((
                SELECT c.n FROM (
                    SELECT library_id, artist_id, COUNT(*) AS n
                    FROM library_tracks WHERE artist_id IS NOT NULL
                    GROUP BY library_id, artist_id
                ) c
                WHERE c.library_id = library_artists.library_id
                  AND c.artist_id = library_artists.artist_id), 0)
             WHERE library_id = ?",
        )
        .bind(library_id)
        .execute(exec)
        .await
        .map(|_| ())
    }

}

#[cfg(test)]
mod scope_tests {
    use super::*;
    use crate::repository::library::library_album_repository::LibraryAlbumRepository;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn two_libraries() -> sqlx::SqlitePool {
        let pool = SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./SQL/").run(&pool).await.unwrap();
        sqlx::query("INSERT OR IGNORE INTO profil (id, name) VALUES (1, 'Socle')").execute(&pool).await.unwrap();
        for (id, name) in [(1, "A"), (2, "B"), (3, "C")] {
            sqlx::query("INSERT INTO library (id, profil_id, name) VALUES (?, 1, ?)").bind(id).bind(name).execute(&pool).await.unwrap();
        }
        for (id, name) in [("shared", "Shared"), ("only2", "Only2")] {
            sqlx::query("INSERT INTO artists (id, name, name_normalized, sort_name) VALUES (?, ?, ?, ?)")
                .bind(id).bind(name).bind(name.to_lowercase()).bind(name).execute(&pool).await.unwrap();
        }
        // « shared » a un album dans chacune des bibliothèques 1 et 2 ; « only2 » seulement dans la 2.
        for (album, lib, artist) in [("a1", 1, "shared"), ("a2", 2, "shared"), ("a3", 2, "only2")] {
            sqlx::query("INSERT INTO library_albums (id, library_id, artist_id, title, title_normalized) VALUES (?, ?, ?, ?, ?)")
                .bind(album).bind(lib).bind(artist).bind(album).bind(album).execute(&pool).await.unwrap();
            sqlx::query("INSERT OR IGNORE INTO library_artists (id, library_id, artist_id) VALUES (?, ?, ?)")
                .bind(format!("{lib}-{artist}")).bind(lib).bind(artist).execute(&pool).await.unwrap();
        }
        pool
    }

    #[tokio::test]
    async fn one_library_lists_only_its_own() {
        let pool = two_libraries().await;
        let artists = LibraryArtistRepository::find_all_artists_by_library_id(&pool, 1i64).await.unwrap();
        assert_eq!(artists.iter().map(|a| a.id.as_str()).collect::<Vec<_>>(), ["shared"]);
        let albums = LibraryAlbumRepository::find_all_albums_by_library_id(&pool, 1i64, None).await.unwrap();
        assert_eq!(albums.len(), 1);
    }

    #[tokio::test]
    async fn an_artist_in_two_checked_libraries_shows_once_with_both_albums() {
        let pool = two_libraries().await;
        let artists = LibraryArtistRepository::find_all_artists_by_library_id(&pool, vec![1, 2]).await.unwrap();
        assert_eq!(artists.len(), 2, "un seul rang par artiste");
        let shared = artists.iter().find(|a| a.id == "shared").unwrap();
        assert_eq!(shared.total_albums, 2);
        let albums = LibraryAlbumRepository::find_all_albums_by_library_id(&pool, vec![1, 2], None).await.unwrap();
        assert_eq!(albums.len(), 3);
    }

    #[tokio::test]
    async fn an_unchecked_library_stays_out() {
        let pool = two_libraries().await;
        let albums = LibraryAlbumRepository::find_all_albums_by_library_id(&pool, vec![3], None).await.unwrap();
        assert!(albums.is_empty());
    }
}
