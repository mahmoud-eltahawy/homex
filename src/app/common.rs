use leptos::prelude::*;

#[component]
pub fn PosterImg(src: String) -> impl IntoView {
    view! {
        <img
            src=src
            class="w-full h-full object-cover transition-transform duration-700 ease-[cubic-bezier(0.34,1.56,0.64,1)] group-hover:scale-110"
            loading="lazy"
        />
    }
}

#[component]
pub fn CardsLoading() -> impl IntoView {
    let cards = (0..5).map(|_| CardSkeleton()).collect_view();
    view! {
        <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4 md:gap-6 my-15">
            {cards}
        </div>
    }
}

#[component]
pub fn CardSkeleton() -> impl IntoView {
    view! {
        <div class="animate-pulse rounded-2xl bg-[#1a1a24]/60 border border-white/5 overflow-hidden shadow-xl">
            <div class="aspect-[2/3] bg-gradient-to-b from-[#2a2a3a] to-[#1a1a24]"></div>
            <div class="p-4 space-y-2">
                <div class="h-3 bg-[#2a2a3a] rounded w-3/4"></div>
                <div class="h-2 bg-[#2a2a3a] rounded w-1/2"></div>
            </div>
        </div>
    }
}
