use crate::app::audio::fetch_audio_groups;
use crate::app::common::{CardsLoading, MediaPageHeader};
use crate::app::model::{self, AudioGroup};
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
        let adapter = move |x| x;
        view! {
            <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                <MediaPageHeader title="صوتيات".to_string() icon=AudioGroup::icon()/>
                <ResourceView
                    resource=this.audio
                    view_fn=CardsList::cards_list
                    adapter=adapter
                    context="تحميل المجموعات الصوتية"
                    fallback=CardsLoading
                />
            </div>
        }
        .into_any()
    }
}

#[server]
pub async fn fetch_audio(
    offset: usize,
    size: usize,
) -> Result<Vec<model::AudioGroup>, ServerFnError> {
    use crate::app::delay;
    use crate::app::mockary;
    delay(300).await;
    let list = mockary::mock_audio_groups();
    let size = size.clamp(0, list.len());
    let offset = offset.clamp(0, list.len() - size);
    let end = (offset + size).clamp(0, list.len());

    Ok(list[offset..end].to_vec())
}
