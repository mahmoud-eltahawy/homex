#[cfg(feature = "ssr")]
use crate::app::delay;
use crate::app::{
    common::{PosterImg, PosterImgProps},
    icons::{SeriesIcon, SeriesPosterSvg},
    model::{MediaType, Season, Series},
    view_schema::{
        CardImageView, IconView, InfoView, MediaTypeT, OverPosterView, PosterSvgView, PosterView,
    },
};
use leptos::{either::Either, prelude::*};

pub mod details;

impl IconView for Series {
    fn icon() -> impl IntoView {
        SeriesIcon()
    }
}

impl MediaTypeT for Series {
    fn media_type() -> MediaType {
        MediaType::Series
    }
}

impl PosterSvgView for Series {
    fn svg_poster() -> impl IntoView {
        SeriesPosterSvg()
    }
}

impl PosterView for Series {
    fn poster(self) -> impl IntoView {
        match &self.poster {
            Some(poster) => Either::Left(PosterImg(PosterImgProps {
                src: poster.clone(),
            })),
            None => Either::Right(Self::svg_poster()),
        }
    }
}

impl OverPosterView for Series {
    fn over_poster(self) -> impl IntoView {
        let title = self.title.to_string();
        view! {
            <div class="absolute inset-0 bg-gradient-to-t from-black via-black/30 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-500 flex flex-col justify-end p-4">
                <div class="transform translate-y-4 group-hover:translate-y-0 transition-transform duration-500">
                    <h3 class="text-white font-bold text-lg leading-tight line-clamp-2">{title}</h3>
                </div>
            </div>
        }
    }
}

impl InfoView for Series {
    fn info_view(self) -> impl IntoView {
        let title = self.title.to_string();
        view! {
            <div class="p-4 flex flex-col gap-1">
                <h3 class="text-white font-semibold truncate text-sm">{title}</h3>
                <div class="flex items-center justify-between text-gray-500 text-xs">
                    <span class="text-cyan-400 text-xs font-medium opacity-0 group-hover:opacity-100 transition-opacity">
                        "← التفاصيل"
                    </span>
                </div>
            </div>
        }
    }
}

impl CardImageView for Series {
    fn card_image(self) -> impl IntoView {
        view! {
            <div class="aspect-[2/3] relative overflow-hidden">
                {self.clone().poster()}
                {self.over_poster()}
                <div class="absolute top-3 end-3 bg-black/70 backdrop-blur-md rounded-full px-2.5 py-1 text-xs font-bold text-white flex items-center gap-1.5 border border-white/10">
                    {Self::icon()}
                    "مسلسل"
                </div>
            </div>
        }
    }
}

#[server]
async fn fetch_series_detail(id: usize) -> Result<Series, ServerFnError> {
    use crate::app::mockary::mock_series;
    delay(200).await;
    let list = mock_series();
    list.into_iter()
        .find(|m| m.id.0 == id)
        .ok_or(ServerFnError::new("not found"))
}

#[server]
pub async fn fetch_season(series_id: usize, season_number: u32) -> Result<Season, ServerFnError> {
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
            .filter(|x| x.title.contains(&pat))
            .collect(),
    };

    Ok(list.len())
}
