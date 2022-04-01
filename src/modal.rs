use std::cell::RefCell;

use futures::{channel::mpsc, StreamExt};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct ModalProps {
    #[prop_or_default]
    pub onclose: Callback<MouseEvent>,
    pub children: Children,
}

#[function_component(Modal)]
pub fn modal(props: &ModalProps) -> Html {
    html! {
        <div class="modal">
            <div class="bg" onclick={&props.onclose} />
            <div class="content">
                { props.children.clone() }
            </div>
        </div>
    }
}

#[derive(Properties, Clone, PartialEq)]
pub struct DialogProps {
    pub title: String,
    pub children: Children,
}

#[function_component(Dialog)]
pub fn dialog(props: &DialogProps) -> Html {
    html! {
        <div class="dialog">
            <h1>{ props.title.as_str() }</h1>
            { props.children.clone() }
        </div>
    }
}

#[derive(Debug, Clone)]
pub enum ModalAction {
    Open(Html),
    Close,
}

#[derive(Debug, Clone)]
pub struct ModalSender(RefCell<mpsc::Sender<ModalAction>>);
impl ModalSender {
    fn try_send(&self, action: ModalAction) -> Result<(), String> {
        let mut sender = self
            .0
            .try_borrow_mut()
            .map_err(|error| format!("failed to borrow modal sender: {error}"))?;
        sender
            .try_send(action)
            .map_err(|error| format!("failed to send modal: {error}"))?;
        Ok(())
    }
    pub fn open(&self, modal: Html) {
        if let Err(e) = self.try_send(ModalAction::Open(modal)) {
            gloo::console::error!(e);
        }
    }
    pub fn close(&self) {
        if let Err(e) = self.try_send(ModalAction::Close) {
            gloo::console::error!(e);
        }
    }
    pub fn close_callback<T>(&self) -> Callback<T> {
        let sender = self.clone();
        Callback::from(move |_| sender.close())
    }
}
impl PartialEq for ModalSender {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ptr() == other.0.as_ptr()
    }
}

pub struct ModalHost {
    modal_content: Option<Html>,
    sender: ModalSender,
}
#[derive(Properties, Clone, PartialEq)]
pub struct ModalHostProps {
    pub children: Children,
}
impl Component for ModalHost {
    type Message = ModalAction;
    type Properties = ModalHostProps;

    fn create(ctx: &Context<Self>) -> Self {
        let (tx, mut rx) = mpsc::channel(1);
        let sender = ModalSender(RefCell::new(tx));
        let link = ctx.link().clone();
        spawn_local(async move {
            while let Some(action) = rx.next().await {
                link.send_message(action);
            }
        });
        Self {
            sender,
            modal_content: None,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: ModalAction) -> bool {
        match msg {
            ModalAction::Open(modal) => {
                self.modal_content = Some(modal);
                true
            }
            ModalAction::Close => self.modal_content.take().is_some(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let modal = self.modal_content.as_ref().map(|content| {
            html! {
                <Modal onclose={ctx.link().callback(|_| ModalAction::Close)}>
                    { content.clone() }
                </Modal>
            }
        });
        html! {
          <ContextProvider<ModalSender> context={self.sender.clone()}>
            { ctx.props().children.clone() }
            { modal.unwrap_or_default() }
          </ContextProvider<ModalSender>>
        }
    }
}
