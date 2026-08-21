use leptos::logging::log;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::app::icons::SearchIcon;
use crate::app::model::MediaType;

#[component]
pub fn SearchBox(search_term: RwSignal<String>, search_open: RwSignal<bool>) -> impl IntoView {
    let media_type = RwSignal::new(None);
    let class = move || {
        format!(
            "relative me-2 transition-all duration-500 ease-[cubic-bezier(0.34,1.56,0.64,1)] {}",
            if search_open.get() { "w-128" } else { "w-10" }
        )
    };
    view! {
        <div class=class>
            <SearchToggle search_open/>
            <SearchInput search_term search_open media_type/>
            <MediaTypeSearch search_open media_type/>
        </div>
    }
}

#[component]
fn MediaTypeSearch(
    search_open: RwSignal<bool>,
    media_type: RwSignal<Option<MediaType>>,
) -> impl IntoView {
    let select_any = move |_| media_type.set(None);
    let select_movie = move |_| media_type.set(Some(MediaType::Movie));
    let select_series = move |_| media_type.set(Some(MediaType::Series));
    view! {
        <Show when=move || search_open.get()>
            <select class="mx-5">
                <option on:click=select_any>"فيلم او مسلسل"</option>
                <option on:click=select_movie>"فيلم"</option>
                <option on:click=select_series>"مسلسل"</option>
            </select>
        </Show>
    }
}

#[component]
fn SearchToggle(search_open: RwSignal<bool>) -> impl IntoView {
    let on_click = move |_| search_open.set(!search_open.get());
    view! {
        <button type="button" on:click=on_click
            class="absolute start-1 top-1/2 -translate-y-1/2 p-1.5 rounded-full text-gray-400 hover:text-white hover:bg-white/10 transition-colors">
            <SearchIcon/>
        </button>
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Suggestion {
    pub id: usize,
    pub name: String,
    pub media_type: MediaType,
}

#[server]
async fn get_suggetions(
    term: String,
    media_type: Option<MediaType>,
) -> Result<Vec<Suggestion>, ServerFnError> {
    let movies = vec![
        Suggestion {
            id: 1,
            name: "Movie 1".to_string(),
            media_type: MediaType::Movie,
        },
        Suggestion {
            id: 2,
            name: "Movie 2".to_string(),
            media_type: MediaType::Movie,
        },
        Suggestion {
            id: 3,
            name: "Movie 3".to_string(),
            media_type: MediaType::Movie,
        },
        Suggestion {
            id: 4,
            name: "Movie 4".to_string(),
            media_type: MediaType::Movie,
        },
    ];
    let series = vec![
        Suggestion {
            id: 1,
            name: "Series 1".to_string(),
            media_type: MediaType::Series,
        },
        Suggestion {
            id: 2,
            name: "Series 2".to_string(),
            media_type: MediaType::Series,
        },
        Suggestion {
            id: 3,
            name: "Series 3".to_string(),
            media_type: MediaType::Series,
        },
        Suggestion {
            id: 4,
            name: "Series 4".to_string(),
            media_type: MediaType::Series,
        },
    ];
    let res = match media_type {
        Some(MediaType::Movie) => movies
            .into_iter()
            .filter(|x| x.name.to_lowercase().starts_with(&term))
            .collect(),
        Some(MediaType::Series) => series
            .into_iter()
            .filter(|x| x.name.to_lowercase().starts_with(&term))
            .collect(),
        None => movies
            .into_iter()
            .chain(series)
            .filter(|x| x.name.to_lowercase().starts_with(&term))
            .collect(),
    };
    Ok(res)
}

#[component]
fn SearchInput(
    search_term: RwSignal<String>,
    search_open: RwSignal<bool>,
    media_type: RwSignal<Option<MediaType>>,
) -> impl IntoView {
    let class = move || {
        format!("w-full bg-white/5 backdrop-blur-xl text-white placeholder-gray-500 rounded-full py-2.5 pe-4 ps-12 text-sm focus:outline-none focus:ring-2 focus:ring-cyan-400/50 focus:bg-white/10 transition-all duration-300 {}",
            if search_open.get() { "opacity-100 scale-100" } else { "opacity-0 scale-95 pointer-events-none" })
    };
    let placeholder = move || match media_type.get() {
        Some(MediaType::Movie) => "ابحث عن فيلم ...",
        Some(MediaType::Series) => "ابحث عن مسلسل...",
        None => "ابحث عن فيلم او مسلسل ...",
    };

    view! {
        <input
            type="text"
            prop:value=search_term
            on:input=move |ev| search_term.set(event_target_value(&ev))
            on:focus=move |_| search_open.set(true)
            on:blur=move |_| search_open.set(!search_term.read().is_empty())
            placeholder=placeholder
            class=class
        />
        <Suggestions search_open search_term media_type/>
    }
}

#[component]
pub fn Suggestions(
    search_open: RwSignal<bool>,
    search_term: RwSignal<String>,
    media_type: RwSignal<Option<MediaType>>,
) -> impl IntoView {
    let suggestions = Resource::new(
        move || (search_term.get(), media_type.get()),
        async move |x| {
            let (term, kind) = x;
            get_suggetions(term, kind).await
        },
    );
    let class = move || {
        format!("w-full bg-white/5 backdrop-blur-xl text-white placeholder-gray-500 rounded-full py-2.5 pe-4 ps-12 text-sm focus:outline-none focus:ring-2 focus:ring-cyan-400/50 focus:bg-white/10 transition-all duration-300 {}",
            if search_open.get() { "opacity-100 scale-100" } else { "opacity-0 scale-95 pointer-events-none" })
    };

    let helper = move || {
        suggestions
            .get()
            .transpose()
            .ok()
            .flatten()
            .unwrap_or_default()
    };

    Effect::new(move || log!("{:#?}", helper()));

    let href = move |kind: MediaType, id: usize| match kind {
        MediaType::Movie => format!("detail/movie/{}", id),
        MediaType::Series => format!("detail/series/{}", id),
    };
    view! {
        <Show when=move || search_open.get() && !search_term.read().is_empty()>
            <Suspense>
                <ul class=class>
                    <For
                        each=helper
                        key=|x| x.id
                        let:item
                    >
                        <li><a href=href(item.media_type,item.id)>{item.name}</a></li>
                    </For>
                </ul>
            </Suspense>
        </Show>
    }
}
