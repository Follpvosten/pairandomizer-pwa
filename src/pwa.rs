use wasm_bindgen::{closure::Closure, JsCast, JsValue, UnwrapThrowExt};
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{ServiceWorker, ServiceWorkerState};
use yew::prelude::*;

use crate::modal::{Dialog, Modal};

pub struct PwaHandler {
    update_available: bool,
    new_worker: Option<ServiceWorker>,
}

pub enum Msg {
    // We mostly handle update related stuff here
    UpdateInstalling(ServiceWorker),
    UpdateAvailable,
    ActivateUpdate,
    DismissUpdate,
    UpdateInstalled,
}

impl Component for PwaHandler {
    type Message = Msg;

    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let navigator = gloo::utils::window().navigator();
        if JsValue::from_str("serviceWorker").js_in(&navigator) {
            let update_cb = ctx.link().callback(|_| Msg::UpdateAvailable);
            let updated_cb = ctx.link().callback(|_| Msg::UpdateInstalled);
            let installing_cb = ctx.link().callback(Msg::UpdateInstalling);
            spawn_local(async move {
                gloo::console::log!("Attempting to register service worker");
                let container = navigator.service_worker();
                container.set_oncontrollerchange(Some(
                    Closure::wrap(Box::new(move || updated_cb.emit(())) as Box<dyn Fn()>)
                        .into_js_value()
                        .unchecked_ref(),
                ));
                let reg = JsFuture::from(container.register("/sw.js"))
                    .await
                    .unwrap_throw()
                    .unchecked_into::<web_sys::ServiceWorkerRegistration>();
                reg.clone().set_onupdatefound(Some(
                    Closure::wrap(Box::new(move || {
                        let worker = reg.installing().unwrap_throw();
                        installing_cb.emit(worker.clone());
                        let callback = update_cb.clone();
                        worker.clone().set_onstatechange(Some(
                            Closure::wrap(Box::new(move || {
                                if worker.state() == ServiceWorkerState::Installed {
                                    callback.emit(());
                                }
                            }) as Box<dyn Fn()>)
                            .into_js_value()
                            .unchecked_ref(),
                        ));
                    }) as Box<dyn Fn()>)
                    .into_js_value()
                    .unchecked_ref(),
                ));
            });
        }
        Self {
            update_available: false,
            new_worker: None,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Msg) -> bool {
        match msg {
            Msg::UpdateInstalling(worker) => {
                self.new_worker = Some(worker);
                false
            }
            Msg::UpdateAvailable => {
                gloo::console::log!("Update found!");
                self.update_available = true;
                true
            }
            Msg::ActivateUpdate => {
                if let Some(worker) = self.new_worker.take() {
                    worker
                        .post_message(&JsValue::from_str("force_update"))
                        .unwrap_throw();
                }
                true
            }
            Msg::DismissUpdate => {
                self.new_worker = None;
                self.update_available = false;
                true
            }
            Msg::UpdateInstalled => {
                gloo::utils::window()
                    .location()
                    .reload_with_forceget(true)
                    .unwrap_throw();
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if !(self.update_available && self.new_worker.is_some()) {
            return html! {};
        }
        let close = ctx.link().callback(|_| Msg::DismissUpdate);
        let update = ctx.link().callback(|_| Msg::ActivateUpdate);
        html! {
          <Modal onclose={&close}>
            <Dialog title="Update available!">
              <div>{"Update now?"}</div>
              <div class="buttons">
                <button onclick={close}>{"Ignore"}</button>
                <button onclick={update}>{"Yes"}</button>
              </div>
            </Dialog>
          </Modal>
        }
    }
}
