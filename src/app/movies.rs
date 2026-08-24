use crate::app::{
    common::{PosterImg, PosterImgProps},
    icons::{ClockIcon, MovieIcon, MoviePosterSvg},
    model::Movie,
    view_schema::{
        CardImageView, Href, IconView, InfoView, MediaTypeT, OverPosterView, PosterSvgView,
        PosterView,
    },
};
use leptos::{either::Either, prelude::*};

use super::model::MediaType;

pub mod detail;
pub mod listing;

impl IconView for Movie {
    fn icon() -> impl IntoView {
        MovieIcon()
    }
}

impl MediaTypeT for Movie {
    fn media_type() -> MediaType {
        MediaType::Movie
    }
}

impl PosterSvgView for Movie {
    fn svg_poster() -> impl IntoView {
        MoviePosterSvg()
    }
}

impl PosterView for Movie {
    fn poster(self) -> impl IntoView {
        match &self.poster {
            Some(poster) => Either::Left(PosterImg(PosterImgProps {
                src: poster.clone(),
            })),
            None => Either::Right(Self::svg_poster()),
        }
    }
}

impl OverPosterView for Movie {
    fn over_poster(self) -> impl IntoView {
        let title = self.title.to_string();
        let duration_display = self.duration.human_readable();
        view! {
            <div class="absolute inset-0 bg-gradient-to-t from-black via-black/30 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-500 flex flex-col justify-end p-4">
                <div class="transform translate-y-4 group-hover:translate-y-0 transition-transform duration-500">
                    <h3 class="text-white font-bold text-lg leading-tight line-clamp-2">{title}</h3>
                    <div class="flex items-center gap-2 mt-1 text-gray-300 text-sm">
                        <span class="flex items-center"><ClockIcon/>{duration_display}</span>
                    </div>
                </div>
            </div>
        }
    }
}

impl CardImageView for Movie {
    fn card_image(self) -> impl IntoView {
        view! {
            <div class="aspect-[2/3] relative overflow-hidden">
                {self.clone().poster()}
                {self.over_poster()}
                <div class="absolute top-3 end-3 bg-black/70 backdrop-blur-md rounded-full px-2.5 py-1 text-xs font-bold text-white flex items-center gap-1.5 border border-white/10">
                    {Self::icon()}
                    "فيلم"
                </div>
            </div>
        }
    }
}
impl Href for Movie {
    fn href(self) -> String {
        format!("/detail/movie/{}", self.id.0)
    }
}

impl InfoView for Movie {
    fn info_view(self) -> impl IntoView {
        let title = self.title.to_string();
        let size = self.file.size.human_readable();
        view! {
            <div class="p-4 flex flex-col gap-1">
                <h3 class="text-white font-semibold truncate text-sm">{title}</h3>
                <h4 class="text-white font-semibold truncate text-sm">{size}</h4>
                <div class="flex items-center justify-between text-gray-500 text-xs">
                    <span class="text-cyan-400 text-xs font-medium opacity-0 group-hover:opacity-100 transition-opacity">
                        "← التفاصيل"
                    </span>
                </div>
            </div>
        }
    }
}

#[server]
pub async fn fetch_movies(offset: usize, size: usize) -> Result<Vec<Movie>, ServerFnError> {
    use crate::app::delay;
    use crate::app::mockary;
    delay(300).await;
    let list = mockary::mock_movies();
    let size = size.clamp(0, list.len());
    let offset = offset.clamp(0, list.len() - size);
    let end = (offset + size).clamp(0, list.len());

    Ok(list[offset..end].to_vec())
}

#[server]
pub async fn fetch_movies_count() -> Result<usize, ServerFnError> {
    use crate::app::delay;
    use crate::app::mockary;
    delay(300).await;

    Ok(mockary::mock_movies().len())
}
