use std::rc::Rc;

use bounce::prelude::*;
use gloo::storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};
use wasm_bindgen::UnwrapThrowExt;
use yew::prelude::*;

use crate::{app::CurrentIndex, modal::Dialog};

const STORAGE_KEY: &str = "settings";

#[derive(Clone, PartialEq, Atom, Serialize, Deserialize)]
#[observed]
pub struct Settings {
    pub ignore_language: bool,
    pub scenario_index: Option<usize>,
    pub scene_index: Option<usize>,
}

impl Default for Settings {
    fn default() -> Self {
        LocalStorage::get(STORAGE_KEY).unwrap_or_else(|error| {
            gloo::console::error!(format!("Error reading from storage: {:?}", error));
            Settings {
                ignore_language: false,
                scenario_index: None,
                scene_index: None,
            }
        })
    }
}
impl Observed for Settings {
    fn changed(self: Rc<Self>) {
        LocalStorage::set(STORAGE_KEY, &*self).expect_throw("failed to save settings")
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct SettingsDialogProps {
    pub onclose: Callback<MouseEvent>,
}

#[function_component(SettingsDialog)]
pub fn settings_dialog(props: &SettingsDialogProps) -> Html {
    let settings = use_atom::<Settings>();
    let index = use_atom_value::<CurrentIndex>();
    let index = Option::as_ref(&index);

    let scenario_options = index
        .map(|index| {
            index
                .scenarios
                .iter()
                .map(|s| s.0.name.clone())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let scene_options = settings
        .scenario_index
        .and_then(|ix| index.map(|index| (index, ix)))
        .and_then(|(index, ix)| index.scenarios.get(ix))
        .map(|(_, scenario)| {
            scenario
                .scenes
                .iter()
                .map(|s| s.name.clone())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let update_ignore_language = {
        let settings = settings.clone();
        Callback::from(move |e: Event| {
            let cb = e.target_unchecked_into::<web_sys::HtmlInputElement>();
            let mut new_settings = Settings::clone(&settings);
            new_settings.ignore_language = cb.checked();
            settings.set(new_settings);
        })
    };
    let update_scenario = {
        let settings = settings.clone();
        Callback::from(move |index| {
            // Only update if there's actually a change
            if settings.scenario_index != index {
                let mut new_settings = Settings::clone(&settings);
                new_settings.scenario_index = index;
                // We know the scenario changed, so we reset the scene index
                new_settings.scene_index = None;
                settings.set(new_settings);
            }
        })
    };
    let update_scene = {
        let settings = settings.clone();
        Callback::from(move |index| {
            let mut new_settings = Settings::clone(&settings);
            new_settings.scene_index = index;
            settings.set(new_settings);
        })
    };

    html! {
      <Dialog title="Settings">
        <div class="settings">
          <div class="setting">
            <span class="name">{ "Allow all languages" }</span>
            <input
              type="checkbox"
              checked={settings.ignore_language}
              onchange={update_ignore_language}
              />
          </div>
          <div class="setting">
            <span class="name">{ "Fixed scenario:" }</span>
            <Dropdown
              options={scenario_options}
              value={settings.scenario_index}
              onselect={update_scenario}
              disabled={index.is_none()}
              />
          </div>
          <div class="setting">
            <span class="name">{ "Fixed scene:" }</span>
            <Dropdown
              options={scene_options}
              value={settings.scene_index}
              onselect={update_scene}
              disabled={index.is_none() || settings.scenario_index.is_none()}
              />
          </div>
        </div>
        <div class="buttons">
          <span />
          <button onclick={&props.onclose}>{"Close"}</button>
        </div>
      </Dialog>
    }
}

#[derive(Clone, PartialEq, Properties)]
struct DropdownProps {
    options: Vec<String>,
    value: Option<usize>,
    onselect: Callback<Option<usize>>,
    disabled: bool,
}

#[function_component(Dropdown)]
fn dropdown(props: &DropdownProps) -> Html {
    let options = props.options.iter().enumerate().map(|(ix, option)| {
        let selected = props.value == Some(ix);
        html! {
          <option value={ix.to_string()} {selected}>
            { option.clone() }
          </option>
        }
    });
    let onchange = props.onselect.reform(|e: Event| {
        let val = e
            .target_unchecked_into::<web_sys::HtmlSelectElement>()
            .value();
        if val == "none" {
            None
        } else {
            val.parse().ok()
        }
    });
    html! {
      <select {onchange} disabled={props.disabled}>
        <option value="none" selected={props.value.is_none()}>
          { "None" }
        </option>
        { for options }
      </select>
    }
}
