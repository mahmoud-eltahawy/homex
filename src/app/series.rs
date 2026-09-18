#[cfg(feature = "ssr")]
use crate::app::delay;
use crate::app::{
    icons::{SeriesIcon, SeriesPosterSvg},
    model::{MediaType, Season, Series},
    view_schema::CardData,
};
use leptos::prelude::*;

pub mod details;

impl CardData for Series {
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
        MediaType::Series
    }

    fn badge_icon() -> impl IntoView {
        SeriesIcon()
    }

    fn placeholder_poster() -> impl IntoView {
        SeriesPosterSvg()
    }
}

#[server]
async fn fetch_series_detail(id: u64) -> Result<Series, ServerFnError> {
    use crate::app::mockary::mock_series;
    delay(200).await;
    let list = mock_series();
    list.into_iter()
        .find(|m| m.id == id)
        .ok_or(ServerFnError::new("not found"))
}

#[server]
pub async fn fetch_season(series_id: u64, season_number: u32) -> Result<Season, ServerFnError> {
    use crate::app::mockary::mock_season;
    delay(200).await;
    mock_season(series_id, season_number).ok_or(ServerFnError::new("season not found"))
}

#[server]
pub async fn fetch_series(
    offset: usize,
    size: usize,
    search_query: Option<String>,
) -> Result<Vec<Series>, ServerFnError> {
    use crate::app::mockary::mock_series;
    delay(300).await;

    let list = match search_query {
        None => mock_series(),
        Some(pat) => mock_series()
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
pub async fn fetch_series_count(search_query: Option<String>) -> Result<usize, ServerFnError> {
    use crate::app::mockary::mock_series;
    delay(300).await;
    let list = match search_query {
        None => mock_series(),
        Some(pat) => mock_series()
            .into_iter()
            .filter(|x| x.title.to_lowercase().contains(&pat.to_lowercase()))
            .collect(),
    };

    Ok(list.len())
}
