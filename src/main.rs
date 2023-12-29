use bounce::BounceRoot;
use unic_langid::LanguageIdentifier;
use yew::prelude::*;

use crate::{app::App, modal::ModalHost, pwa::PwaHandler};

mod app;
mod modal;
mod pairandomizer_core;
mod pwa;
mod settings;

fn main() {
    yew::Renderer::<AppShell>::new().render();
}

struct AppShell {
    language: LanguageIdentifier,
}

impl Component for AppShell {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let navigator = gloo::utils::window().navigator();
        let language = navigator
            .language()
            .and_then(|lang| lang.parse().ok())
            .unwrap_or_else(|| unic_langid::langid!("en-US"));
        Self { language }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
          <BounceRoot>
            <ContextProvider<LanguageIdentifier> context={self.language.clone()}>
              <ModalHost>
                <App />
                <PwaHandler />
              </ModalHost>
            </ContextProvider<LanguageIdentifier>>
          </BounceRoot>
        }
    }
}
