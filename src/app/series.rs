#[cfg(feature = "ssr")]
use crate::app::delay;
use crate::app::model::{Season, Series};
use leptos::prelude::*;

pub mod details;
pub mod listing;

#[server]
async fn fetch_series_detail(id: i64) -> Result<Series, ServerFnError> {
    use crate::app::mockary::mock_series;
    delay(200).await;
    let list = mock_series();
    list.into_iter()
        .find(|m| m.id.0 == id)
        .ok_or(ServerFnError::new("not found"))
}

#[server]
pub async fn fetch_season(series_id: i64, season_number: u32) -> Result<Season, ServerFnError> {
    use crate::app::mockary::mock_season;
    delay(200).await;
    mock_season(series_id, season_number).ok_or(ServerFnError::new("season not found"))
}

#[server]
pub async fn fetch_series() -> Result<Vec<Series>, ServerFnError> {
    use crate::app::mockary::mock_series;
    delay(300).await;
    Ok(mock_series())
}
