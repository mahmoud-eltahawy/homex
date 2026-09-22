use leptos::prelude::*;
use leptos_router::{LazyRoute, hooks::use_navigate, lazy_route};

#[server(endpoint = "login")]
pub async fn login(token: String) -> Result<(), ServerFnError> {
    use crate::app::server::auth::expected_token;
    use axum::http::header;
    use leptos_axum::ResponseOptions;

    let Some(expected) = expected_token() else {
        return Err(ServerFnError::new("auth disabled"));
    };
    if token != expected {
        return Err(ServerFnError::new("رمز غير صحيح"));
    }

    let cookie = format!("homex_token={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age=2592000");
    let value = cookie
        .parse()
        .map_err(|_| ServerFnError::new("invalid cookie"))?;

    let response = expect_context::<ResponseOptions>();
    response.insert_header(header::SET_COOKIE, value);

    Ok(())
}

pub struct LoginPage;

#[lazy_route]
impl LazyRoute for LoginPage {
    fn data() -> Self {
        Self
    }

    fn view(_this: Self) -> AnyView {
        let token = RwSignal::new(String::new());
        let navigate = use_navigate();
        let submit = Action::new_local(|t: &String| login(t.clone()));

        Effect::new(move |_| {
            if matches!(submit.value().get(), Some(Ok(()))) {
                navigate("/", Default::default());
            }
        });

        let error = move || match submit.value().get() {
            Some(Err(e)) => Some(e.to_string()),
            _ => None,
        };

        let on_submit = move |ev: web_sys::SubmitEvent| {
            ev.prevent_default();
            let t = token.get_untracked();
            if !t.is_empty() {
                submit.dispatch(t);
            }
        };

        view! {
            <div class="min-h-[70vh] flex items-center justify-center px-4">
                <form
                    on:submit=on_submit
                    class="w-full max-w-sm bg-[#12121a] border border-white/10 rounded-2xl p-8 space-y-4 shadow-2xl"
                >
                    <h1 class="text-lg font-bold text-white text-center">
                        "HomeX — أدخل رمز الوصول"
                    </h1>

                    <input
                        type="password"
                        autofocus=true
                        autocomplete="current-password"
                        prop:value=move || token.get()
                        on:input=move |e| token.set(event_target_value(&e))
                        class="w-full bg-white/10 rounded-lg py-2 px-3 text-white \
                                   focus:outline-none focus:ring-2 focus:ring-cyan-400/50"
                    />

                    <button
                        type="submit"
                        disabled=move || submit.pending().get() || token.get().is_empty()
                        class="w-full py-2 rounded-lg font-bold text-white \
                                   bg-gradient-to-r from-cyan-500 to-blue-500 \
                                   hover:from-cyan-400 hover:to-blue-400 \
                                   disabled:opacity-50 transition"
                    >
                        {move || if submit.pending().get() { "جارٍ الدخول..." } else { "دخول" }}
                    </button>

                    {move || error().map(|e| view! {
                        <div class="text-red-300 text-sm text-center">{e}</div>
                    })}
                </form>
            </div>
        }.into_any()
    }
}
