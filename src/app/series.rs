#[cfg(feature = "ssr")]
use crate::app::delay;
use leptos::prelude::*;

pub mod details;
pub mod listing;

#[server]
async fn fetch_series_detail(id: i64) -> Result<listing::Series, ServerFnError> {
    delay(200).await;
    let list = listing::mock_series();
    list.into_iter()
        .find(|m| m.id.0 == id)
        .ok_or(ServerFnError::new("not found"))
}

#[server]
pub async fn fetch_season(
    series_id: i64,
    season_number: u32,
) -> Result<listing::Season, ServerFnError> {
    delay(200).await;
    listing::mock_season(series_id, season_number).ok_or(ServerFnError::new("season not found"))
}

#[server]
async fn fetch_series() -> Result<Vec<listing::Series>, ServerFnError> {
    delay(300).await;
    Ok(listing::mock_series())
}
