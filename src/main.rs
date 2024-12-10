use bsky::get_like_image_urls;
use leptos::{prelude::*, reactive::actions::Action, task::spawn_local};
use leptos_use::use_window_scroll;
use reqwest::Client;
use serde_json::json;

mod bsky;

fn main() {
    leptos::mount::mount_to_body(App)
}

#[component]
fn App() -> impl IntoView {
    let (images, set_images) = signal(None);
    let (handle, set_handle) = signal(String::new());

    view! {
        <input type="text"
            on:input:target=move |ev| {
                set_handle.set(ev.target().value());
            }
        />

        <button
            on:click=move |_| {
                spawn_local(async move {
                    let Ok(urls) = get_like_image_urls(&handle.get()).await else {
                        return;
                    };

                    let images = urls.into_iter().map(|url| {
                        view! { <img src = {url}/> }
                    }).collect::<Vec<_>>();

                    set_images.set(Some(images));
                });
            }
        >

            "Click me: "
        </button>

        {images}
    }
}
