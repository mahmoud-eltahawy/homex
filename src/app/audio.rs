use crate::app::{
    icons::{AudioIcon, MusicPosterSvg},
    model::{self, Audio, AudioGroup, MediaType},
    view_schema::CardData,
};
use leptos::prelude::*;

pub mod detail;
pub mod song;

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

#[server]
async fn fetch_audio_group_detail(id: u64) -> Result<AudioGroup, ServerFnError> {
    use crate::app::delay;
    use crate::app::mockary::mock_audio_groups;
    delay(200).await;
    mock_audio_groups()
        .into_iter()
        .find(|g| g.id == id)
        .ok_or(ServerFnError::new("audio group not found"))
}

#[server]
pub async fn fetch_audios(group_id: u64) -> Result<Vec<Audio>, ServerFnError> {
    use crate::app::delay;
    use crate::app::mockary::mock_audios;
    delay(200).await;
    Ok(mock_audios(group_id))
}

#[server]
pub async fn fetch_audio(group_id: u64, song_id: u64) -> Result<Audio, ServerFnError> {
    use crate::app::delay;
    use crate::app::mockary::mock_audios;
    delay(200).await;
    mock_audios(group_id)
        .into_iter()
        .find(|a| a.id == song_id)
        .ok_or(ServerFnError::new("audio not found"))
}
