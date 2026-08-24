use crate::app::{
    common::{CardsLoading, MediaPageHeader},
    icons::SeriesIcon,
    model::Series,
    resource_view::ResourceView,
    series::fetch_series,
    view_schema::CardsList,
};
use leptos::prelude::*;
use leptos_router::{lazy_route, LazyRoute};

pub struct SeriesPage {
    pub data: Resource<Result<Vec<Series>, ServerFnError>>,
}

#[lazy_route]
impl LazyRoute for SeriesPage {
    fn data() -> Self {
        Self {
            data: Resource::new(|| (), |_| fetch_series(0, 20)),
        }
    }

    fn view(this: Self) -> AnyView {
        view! {
        <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
            <MediaPageHeader title="مسلسلات".to_string() icon=SeriesIcon()/>
            <ResourceView
                resource=this.data
                view_fn=CardsList::cards_list
                fallback=CardsLoading
                adapter=|x| x
                context="تحميل مسلسلات"
            />
        </div>
        }
        .into_any()
    }
}
