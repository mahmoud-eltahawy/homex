use leptos::prelude::*;

#[component]
pub fn Breadcrumb(rel_path: Signal<String>, on_navigate: Callback<String>) -> impl IntoView {
    let segments = Memo::new(move |_| {
        let path = rel_path.get();
        let mut out: Vec<(String, String)> = vec![("Incoming".into(), String::new())];
        if !path.is_empty() {
            let mut acc = String::new();
            for part in path.split('/') {
                if !acc.is_empty() {
                    acc.push('/');
                }
                acc.push_str(part);
                out.push((part.to_string(), acc.clone()));
            }
        }
        out
    });

    view! {
        <nav class="flex items-center gap-1 text-sm mb-4 flex-wrap">
            {move || {
                let segs = segments.get();
                let count = segs.len();
                segs.into_iter().enumerate().map(|(i, (label, path))| {
                    let target = path.clone();
                    let is_last = i + 1 == count;
                    let class = if is_last {
                        "px-2 py-1 rounded transition text-cyan-300 font-medium"
                    } else {
                        "px-2 py-1 rounded transition text-gray-300 hover:bg-white/10 hover:text-white"
                    };
                    view! {
                        <div class="flex items-center gap-1">
                            <button type="button"
                                on:click=move |_| on_navigate.run(target.clone())
                                class=class>{label}</button>
                            {(!is_last).then(|| view! { <span class="text-gray-600">"/"</span> })}
                        </div>
                    }
                }).collect_view()
            }}
        </nav>
    }
}
