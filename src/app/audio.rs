use crate::app::{
    common::{PosterImg, PosterImgProps},
    icons::{AudioIcon, MusicPosterSvg},
    model::{self, AudioGroup, MediaType},
    view_schema::{
        CardImageView, IconView, InfoView, MediaTypeT, OverPosterView, PosterSvgView, PosterView,
    },
};
use leptos::{either::Either, prelude::*};

impl IconView for AudioGroup {
    fn icon() -> impl IntoView {
        AudioIcon()
    }
}

impl MediaTypeT for AudioGroup {
    fn media_type() -> MediaType {
        MediaType::AudioGroup
    }
}

impl PosterSvgView for AudioGroup {
    fn svg_poster() -> impl IntoView {
        MusicPosterSvg()
    }
}

impl PosterView for AudioGroup {
    fn poster(self) -> impl IntoView {
        match &self.poster {
            Some(poster) => Either::Left(PosterImg(PosterImgProps {
                src: poster.clone(),
            })),
            None => Either::Right(Self::svg_poster()),
        }
    }
}

impl OverPosterView for AudioGroup {
    fn over_poster(self) -> impl IntoView {
        view! {
            <div class="absolute inset-0 bg-gradient-to-t from-black via-black/30 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-500 flex flex-col justify-end p-4">
                <div class="transform translate-y-4 group-hover:translate-y-0 transition-transform duration-500">
                    <h3 class="text-white font-bold text-lg leading-tight line-clamp-2">{self.title.clone()}</h3>
                </div>
            </div>

        }
    }
}

impl CardImageView for AudioGroup {
    fn card_image(self) -> impl IntoView {
        view! {
            <div class="aspect-[2/3] relative overflow-hidden">
                {self.clone().poster()}
                {self.over_poster()}
                <div class="absolute top-3 end-3 bg-black/70 backdrop-blur-md rounded-full px-2.5 py-1 text-xs font-bold text-white flex items-center gap-1.5 border border-white/10">
                    {Self::icon()}
                </div>
            </div>
        }
    }
}

impl InfoView for AudioGroup {
    fn info_view(self) -> impl IntoView {
        let title = self.title.to_string();
        view! {
            <div class="p-4 flex flex-col gap-1">
                <h3 class="text-white font-semibold truncate text-sm">{title}</h3>
            </div>
        }
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
