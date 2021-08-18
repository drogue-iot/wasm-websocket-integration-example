use anyhow::Error;
use yew::services::{
    websocket::{WebSocketService, WebSocketTask},
    ConsoleService,
};
use yew::{format::Json, prelude::*};

enum Msg {
    Connect,
    Data(Result<String, Error>),
    Ignore,
}

struct Model {
    // It can be used to send messages to the component
    link: ComponentLink<Self>,
    socket: Option<WebSocketTask>,
    last_entry: String,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            socket: None,
            last_entry: String::from("No data"),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Ignore => false,
            Msg::Data(data) => {
                match data {
                    Ok(s) => {
                        ConsoleService::log("Data updated");
                        self.last_entry = s
                    }
                    Err(e) => {}
                }
                true
            }
            Msg::Connect => {
                let on_data = self.link.callback(|Json(data)| Msg::Data(data));
                let on_notify = self.link.callback(|input| {
                    ConsoleService::log(&format!("Notification: {:?}", input));
                    // TODO: Handle notification
                    Msg::Ignore
                });
                if self.socket.is_none() {
                    let task = WebSocketService::connect_text("wss://websocket-integration-drogue-dev.apps.wonderful.iot-playground.org/drogue-public-temperature", on_data, on_notify);

                    ConsoleService::log("Task created");
                    self.socket.replace(task.unwrap());
                }
                true
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        false
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <button onclick=self.link.callback(|_| Msg::Connect)>{ "Connect" }</button>
                <p>{ self.last_entry.clone() }</p>
            </div>
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}
