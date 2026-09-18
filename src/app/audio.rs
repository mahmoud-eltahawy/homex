use crate::app::{
    icons::{AudioIcon, MusicPosterSvg},
    model::{self, AudioGroup, MediaType},
    view_schema::CardData,
};
use leptos::prelude::*;

impl CardData for AudioGroup {
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
        MediaType::AudioGroup
    }

    fn badge_icon() -> impl IntoView {
        AudioIcon()
    }

    fn placeholder_poster() -> impl IntoView {
        MusicPosterSvg()
    }
}

#[server]
pub async fn fetch_audio_groups(
    offset: usize,
    size: usize,
    search_query: Option<String>,
) -> Result<Vec<model::AudioGroup>, ServerFnError> {
    use crate::app::delay;
    use crate::app::mockary::mock_audio_groups;
    delay(300).await;

    let list = match search_query {
        None => mock_audio_groups(),
        Some(pat) => mock_audio_groups()
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
pub async fn fetch_audio_groups_count(
    search_query: Option<String>,
) -> Result<usize, ServerFnError> {
    use crate::app::delay;
    use crate::app::mockary::mock_audio_groups;
    delay(300).await;

    let list = match search_query {
        None => mock_audio_groups(),
        Some(pat) => mock_audio_groups()
            .into_iter()
            .filter(|x| x.title.to_lowercase().contains(&pat.to_lowercase()))
            .collect(),
    };

    Ok(list.len())
}
