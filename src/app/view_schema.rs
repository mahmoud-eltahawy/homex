use leptos::prelude::*;

use crate::app::model::MediaType;

pub trait IconView {
    fn icon() -> impl IntoView;
}
pub trait MediaTypeT {
    fn media_type() -> MediaType;
}

pub trait PosterSvgView {
    fn svg_poster() -> impl IntoView;
}

pub trait PosterView: PosterSvgView {
    fn poster(self) -> impl IntoView;
}

pub trait OverPosterView: PosterSvgView {
    fn over_poster(self) -> impl IntoView;
}

pub trait CardImageView: IconView + PosterSvgView + OverPosterView {
    fn card_image(self) -> impl IntoView;
}

pub trait InfoView {
    fn info_view(self) -> impl IntoView;
}

pub trait IdT {
    fn id(&self) -> usize;
}

pub trait Card: CardImageView + InfoView + MediaTypeT + IdT {
    fn card(self) -> impl IntoView;
}

impl<T> Card for T
where
    T: IdT + CardImageView + InfoView + MediaTypeT + Clone,
{
    fn card(self) -> impl IntoView {
        let href = T::media_type().detail_href(self.id());
        view! {
            <a
                href=href
                class="group relative flex flex-col overflow-hidden rounded-2xl bg-[#1a1a24]/80 backdrop-blur-sm border border-white/5 shadow-2xl hover:shadow-cyan-500/20 transition-all duration-500 hover:scale-[1.03] hover:-translate-y-2">
                {self.clone().card_image()}
                {self.info_view()}
            </a>
        }
    }
}

pub trait CardsList {
    fn cards_list(self) -> impl IntoView;
}

impl<L, T> CardsList for L
where
    T: Card,
    L: IntoIterator<Item = T>,
{
    fn cards_list(self) -> impl IntoView {
        let core = self.into_iter().map(|item| item.card()).collect_view();
        view! {
            <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4 md:gap-6">
                {core}
            </div>
        }
    }
}
