use yew::prelude::*;

use crate::{modal::ModalSender, settings::SettingsDialog};

#[function_component(Navbar)]
pub fn navbar() -> Html {
    let modal = use_context::<ModalSender>().unwrap();
    let open_settings = Callback::from(move |_| {
        modal.open(html! {
          <SettingsDialog onclose={modal.close_callback()} />
        });
    });
    html! {
        <nav class="navbar">
          <div class="title">
            <h1>{ "Pairandomizer" }</h1>
          </div>
          <div class="controls">
            <div class="ctrl" onclick={open_settings}>
              <span class="icon">{ "⚙️" }</span>
            </div>
          </div>
        </nav>
    }
}
