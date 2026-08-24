use crate::app::audio::fetch_audio_groups;
use crate::app::common::{CardsLoading, MediaPageHeader};
use crate::app::model::AudioGroup;
use crate::app::resource_view::ResourceView;
use crate::app::view_schema::CardsList;
use leptos::prelude::*;
use leptos_router::{lazy_route, LazyRoute};
pub struct AudioGroupPage {
    pub audio: Resource<Result<Vec<AudioGroup>, ServerFnError>>,
}
use crate::app::view_schema::IconView;

#[lazy_route]
impl LazyRoute for AudioGroupPage {
    fn data() -> Self {
        let audio = Resource::new(|| (), async |_| fetch_audio_groups(0, 20).await);
        Self { audio }
    }

    fn view(this: Self) -> AnyView {
        view! {
            <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                <MediaPageHeader title="صوتيات".to_string() icon=AudioGroup::icon()/>
                <ResourceView
                    resource=this.audio
                    view_fn=CardsList::cards_list
                    adapter=|x| x
                    context="تحميل المجموعات الصوتية"
                    fallback=CardsLoading
                />
            </div>
        }
        .into_any()
    }
}
