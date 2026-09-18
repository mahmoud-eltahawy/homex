use super::model::MediaType;
use crate::app::{
    icons::{MovieIcon, MoviePosterSvg},
    model::Movie,
    view_schema::CardData,
};
use leptos::prelude::*;

pub mod detail;

impl CardData for Movie {
    fn id(&self) -> u64 {
        self.id
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn poster_url(&self) -> Option<&str> {
        self.poster.as_deref()
    }

    fn media_type() -> MediaType {
        MediaType::Movie
    }

    fn badge_icon() -> impl IntoView {
        MovieIcon()
    }

    fn placeholder_poster() -> impl IntoView {
        MoviePosterSvg()
    }
}

#[server]
pub async fn fetch_movies(
    offset: usize,
    size: usize,
    search_query: Option<String>,
) -> Result<Vec<Movie>, ServerFnError> {
    use crate::app::delay;
    use crate::app::mockary::mock_movies;
    delay(300).await;

    let list = match search_query {
        None => mock_movies(),
        Some(pat) => mock_movies()
            .into_iter()
            .filter(|x| x.title.to_lowercase().contains(&pat.to_lowercase()))
            .collect(),
    };

    let size = size.clamp(0, list.len());
    let offset = offset.clamp(0, list.len() - size);
    let end = (offset + size).clamp(0, list.len());

    Ok(list[offset..end].to_vec())
}

#[server]
pub async fn fetch_movies_count(search_query: Option<String>) -> Result<usize, ServerFnError> {
    use crate::app::delay;
    use crate::app::mockary::mock_movies;
    delay(300).await;

    let list = match search_query {
        None => mock_movies(),
        Some(pat) => mock_movies()
            .into_iter()
            .filter(|x| x.title.to_lowercase().contains(&pat.to_lowercase()))
            .collect(),
    };

    Ok(list.len())
}
