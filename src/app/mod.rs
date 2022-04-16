use std::rc::Rc;

use bounce::prelude::*;
use gloo::storage::{LocalStorage, Storage};
use itertools::Itertools;
use rand::prelude::SliceRandom;
use unic_langid::LanguageIdentifier;
use wasm_bindgen::UnwrapThrowExt;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::{
    modal::{Dialog, ModalSender},
    pairandomizer_core::{Index, LoadedIndex},
    settings::Settings,
};

mod navbar;

#[cfg(debug_assertions)]
const DEFAULT_SERVER: &str = "http://127.0.0.1:8080";
#[cfg(not(debug_assertions))]
const DEFAULT_SERVER: &str = "https://pr.karp.lol";

#[derive(PartialEq, Default, Atom)]
pub struct CurrentIndex(Option<LoadedIndex>);
impl std::ops::Deref for CurrentIndex {
    type Target = Option<LoadedIndex>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(PartialEq, Atom)]
#[observed]
pub struct Names(Vec<String>);
impl Default for Names {
    fn default() -> Self {
        Self(LocalStorage::get("names").unwrap_or_default())
    }
}
impl Observed for Names {
    fn changed(self: Rc<Self>) {
        LocalStorage::set("names", &self.0).expect_throw("failed to save names");
    }
}

#[derive(Debug, Clone)]
enum State {
    Initial,
    Loading,
    Loaded,
    Error(std::sync::Arc<anyhow::Error>),
}
impl State {
    pub fn is_initial(&self) -> bool {
        matches!(self, Self::Initial)
    }
}

#[function_component(App)]
pub fn app() -> Html {
    // Context
    let lang_id = use_context::<LanguageIdentifier>().unwrap();
    let modal = use_context::<ModalSender>().unwrap();

    // Data
    let index = use_atom::<CurrentIndex>();
    let names = use_atom::<Names>();

    // Settings
    let settings = use_atom_value::<Settings>();

    // State
    let loading_state = use_state(|| State::Initial);
    if loading_state.is_initial() {
        let index = index.clone();
        loading_state.set(State::Loading);
        let loading_state = loading_state.setter();
        spawn_local(async move {
            match Index::load(DEFAULT_SERVER).await {
                Ok(ix) => {
                    index.set(CurrentIndex(Some(ix)));
                    loading_state.set(State::Loaded);
                }
                Err(err) => loading_state.set(State::Error(std::sync::Arc::new(err))),
            }
        });
    }

    let status_text = match &*loading_state {
        State::Initial | State::Loading => "⏳ Loading…".to_string(),
        State::Loaded => {
            if let Some(index) = &**index {
                format!("✅ Loaded from server: {}", index.inner.server_name)
            } else {
                "✅ Loaded, waiting for data...".to_string()
            }
        }
        State::Error(error) => format!("❌ Error: {}", error),
    };

    let update_names = {
        let names = names.clone();
        Callback::from(move |e: InputEvent| {
            let ta = e.target_unchecked_into::<web_sys::HtmlTextAreaElement>();
            let new_names = ta
                .value()
                .trim()
                .split('\n')
                .map(str::trim)
                .filter(|n| !n.is_empty())
                .unique()
                .map(str::to_string)
                .collect();
            names.set(Names(new_names))
        })
    };

    let generate = index
        .is_some()
        .then(|| {
            let index = index.clone();
            let names = names.clone();
            let modal = modal.clone();
            Callback::from(move |_| {
                let index = match &**index {
                    Some(index) => index,
                    None => return,
                };
                let mut rng = rand::thread_rng();
                let (meta, scenario) = index.pick_scenario(&mut rng, &*settings, &lang_id);
                let scene = settings
                    .scene_index
                    .and_then(|index| scenario.scenes.get(index))
                    .unwrap_or_else(|| scenario.scenes.choose(&mut rng).unwrap());
                let messages = scene.randomize(&names.0);
                let title = meta.name.clone() + " - " + &scene.name;
                let msgs = messages.into_iter().map(|msg| {
                    html! { <>
                      { msg.to_string() }
                      <br/>
                    </> }
                });
                modal.open(html! {
                  <Dialog {title}>
                    <div>{ for msgs }</div>
                    <div class="buttons">
                      <span />
                      <button onclick={modal.close_callback()}>{"Close"}</button>
                    </div>
                  </Dialog>
                });
            })
        })
        .unwrap_or_default();

    html! {
      <div class="layout-container">
        <navbar::Navbar />
        <div class="input-container">
          <textarea oninput={update_names} value={names.0.clone().join("\n")} />
        </div>
        <div class="buttons">
          <span>{ status_text }</span>
          <button disabled={index.is_none()} onclick={generate}>
            { format!("Randomize! ({} names)", names.0.len()) }
          </button>
        </div>
      </div>
    }
}
