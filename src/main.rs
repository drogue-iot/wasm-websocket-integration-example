mod moving_avg;

use anyhow::Error;
use yew::services::{
    websocket::{WebSocketService, WebSocketTask},
    ConsoleService,
};
use yew::{prelude::*};
use serde_json::Value;

use crate::moving_avg::MovingAverage;

enum Msg {
    Connect,
    Data(Result<String, Error>),
    Ignore,
}

struct Model {
    // It can be used to send messages to the component
    link: ComponentLink<Self>,
    socket: Option<WebSocketTask>,
    temperature_values: MovingAverage<f64>,
    average :f64,
    trend: String,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            socket: None,
            temperature_values: MovingAverage::new(5),
            average: 0.0,
            trend: String::from("No data"),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Ignore => false,
            Msg::Data(data) => {
                match data {
                    Ok(s) => {
                        ConsoleService::log("Data updated");
                        let json: Value = serde_json::from_str(s.as_str()).unwrap();

                        let temp = json["data"]["temp"].as_f64().unwrap();
                        self.update_data(temp);
                    }
                    Err(e) => {
                        ConsoleService::log(format!("Data ERROR: {}", e).as_str());
                    }
                }
                true
            }
            Msg::Connect => {
                let on_data = self.link.callback(|data| Msg::Data(data));
                let on_notify = self.link.callback(|input| {
                    ConsoleService::log(&format!("Notification: {:?}", input));
                    // TODO: Handle notification
                    Msg::Ignore
                });
                if self.socket.is_none() {
                    let task = WebSocketService::connect_text(
//                        "ws://localhost:8080/chat/me",
"wss://websocket-integration-drogue-dev.apps.wonderful.iot-playground.org/drogue-public-temperature",
on_data,
on_notify,
                    );

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
                <p>{"Last received value: "}{ self.temperature_values.last() }</p>
                <p>{"Last 5 values average: "}{ self.average }</p>
                <p>{ self.trend.clone() }</p>
            </div>
        }
    }
}

impl Model {
    fn update_data(&mut self, new_value: f64) {

        let average =self.temperature_values.add(new_value).clone();

            self.trend = if  average > self.average {
             String::from("Warming up")
        } else {
            String::from("cooling down")
        };

        self.average = average;
    }
}

fn main() {
    yew::start_app::<Model>();
}
