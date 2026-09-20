// src/app/view_schema.rs
use crate::app::{common::PosterImg, icons::EditIcon, model::MediaType};
use leptos::{either::Either, prelude::*};
use serde::{Serialize, de::DeserializeOwned};

pub trait CardData: Clone + Send + Sync + Serialize + DeserializeOwned + 'static {
    fn id(&self) -> u64;
    fn title(&self) -> &str;
    fn poster_url(&self) -> Option<&str>;
    fn media_type() -> MediaType;
    fn badge_icon() -> impl IntoView;
    fn placeholder_poster() -> impl IntoView;
    fn poster(self) -> impl IntoView {
        match self.poster_url() {
            Some(src) => Either::Left(view! {
                <PosterImg src=src.to_string()/>
            }),
            None => Either::Right(Self::placeholder_poster()),
        }
    }
}

#[component]
pub fn MediaCard<T: CardData>(
    item: T,
    #[prop(optional)] on_edit: Option<Callback<u64>>,
) -> impl IntoView {
    let id = item.id();
    let href = T::media_type().detail_href(id);
    let title = item.title().to_string();
    let icon = T::badge_icon();
    let title_overlay = title.clone();
    let title_footer = title;
    let p = item.poster();

    view! {
        <div class="group relative">
            <a href=href class="group/card relative flex flex-col overflow-hidden rounded-2xl bg-[#1a1a24]/80 backdrop-blur-sm border border-white/5 shadow-2xl hover:shadow-cyan-500/20 transition-all duration-500 hover:scale-[1.03]">
                <div class="aspect-[2/3] relative overflow-hidden">
                    {p}
                    <div class="absolute inset-0 bg-gradient-to-t from-black via-black/30 to-transparent opacity-0 group-hover/card:opacity-100 transition-opacity duration-500 flex flex-col justify-end p-4">
                        <h3 class="text-white font-bold text-lg leading-tight line-clamp-2 transform translate-y-4 group-hover/card:translate-y-0 transition-transform duration-500">
                            {title_overlay}
                        </h3>
                    </div>
                    <div class="absolute top-3 end-3 bg-black/70 backdrop-blur-md rounded-full px-2.5 py-1 text-xs font-bold text-white flex items-center gap-1.5 border border-white/10">
                        {icon}
                    </div>
                </div>
                <div class="p-4 flex flex-col gap-1">
                    <h3 class="text-white font-semibold truncate text-sm">{title_footer}</h3>
                </div>
            </a>
            {on_edit.map(|cb| view! {
                <button
                    type="button"
                    on:click=move |ev| { ev.prevent_default(); ev.stop_propagation(); cb.run(id); }
                    class="absolute top-2 start-2 z-10 p-2 rounded-xl bg-black/60 backdrop-blur-md hover:bg-black/80 text-white opacity-0 group-hover:opacity-100 transition"
                    aria-label="تعديل سريع"
                >
                    <EditIcon/>
                </button>
            })}
        </div>
    }
}

pub trait CardsList {
    fn cards_list(self) -> impl IntoView;
}

impl<L, T> CardsList for L
where
    T: CardData,
    L: IntoIterator<Item = T>,
{
    fn cards_list(self) -> impl IntoView {
        view! {
            <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4 md:gap-6">
                {self.into_iter().map(|x| view! { <MediaCard item=x/> }).collect_view()}
            </div>
        }
    }
}
