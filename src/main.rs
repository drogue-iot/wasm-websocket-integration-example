mod moving_avg;

use anyhow::Error;
use chrono::{DateTime, TimeZone, Utc};
use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use serde_json::Value;
use std::collections::{HashMap, VecDeque};
use yew::prelude::*;
use yew::services::{
    websocket::{WebSocketService, WebSocketTask},
    ConsoleService,
};

enum Msg {
    Connect,
    UpdateUrl(String),
    UpdateGraph,
    Data(Result<String, Error>),
    Ignore,
}

struct Model {
    // It can be used to send messages to the component
    link: ComponentLink<Self>,
    socket: Option<WebSocketTask>,
    temperature_dataset: HashMap<String, (usize, VecDeque<(i64, f64)>)>,
    state: String,
    url: String,
}

const CAPACITY: usize = 100;

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            socket: None,
            temperature_dataset: HashMap::new(),
            state: String::from("No Connected"),
            url: "wss://ws-integration.sandbox.drogue.cloud/drogue-public-temperature".to_string(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Ignore => false,
            Msg::UpdateUrl(url) => {
                self.url = url;
                true
            }
            Msg::Data(data) => {
                self.state = String::from("Receiving data");
                match data {
                    Ok(s) => {
                        ConsoleService::log("Data updated");
                        let json: Value = serde_json::from_str(s.as_str()).unwrap();

                        ConsoleService::log(&format!("Received: {:?}", &json));

                        json["time"].as_str().map(|time| {
                            ConsoleService::log(&format!("Time: {:?}", time));
                            DateTime::parse_from_rfc3339(&time).map(|r| {
                                ConsoleService::log(&format!(
                                    "Successfully parsed, timestamp: {}",
                                    r.timestamp()
                                ));
                                if let Some(device) = json["device"].as_str() {
                                    let (_, dataset) =
                                        if self.temperature_dataset.contains_key(device) {
                                            self.temperature_dataset.get_mut(device).unwrap()
                                        } else {
                                            let color = pick_random_color();
                                            self.temperature_dataset.insert(
                                                device.to_string(),
                                                (color, VecDeque::with_capacity(CAPACITY)),
                                            );
                                            self.temperature_dataset.get_mut(device).unwrap()
                                        };
                                    let temp = &json["data"]["temp"];
                                    match temp {
                                        Value::Number(num) => {
                                            num.as_f64().map(|temp| {
                                                if dataset.len() >= CAPACITY {
                                                    dataset.pop_front();
                                                }
                                                dataset.push_back((r.timestamp(), temp));
                                            });
                                        }
                                        Value::String(s) => {
                                            let s = s.as_str();
                                            if let Ok(temp) = s.parse::<f64>() {
                                                if dataset.len() >= CAPACITY {
                                                    dataset.pop_front();
                                                }
                                                dataset.push_back((r.timestamp(), temp));
                                            }
                                        }
                                        _ => {}
                                    }
                                    self.link.send_message(Msg::UpdateGraph);
                                }
                            });
                        });
                    }
                    Err(e) => {
                        ConsoleService::log(format!("Data ERROR: {}", e).as_str());
                    }
                }
                true
            }
            Msg::Connect => {
                self.state = String::from("Connecting");
                let on_data = self.link.callback(|data| Msg::Data(data));
                let on_notify = self.link.callback(|input| {
                    ConsoleService::log(&format!("Notification: {:?}", input));
                    // TODO: Handle notification
                    Msg::Ignore
                });
                if self.socket.is_none() {
                    let task = WebSocketService::connect_text(&self.url, on_data, on_notify);
                    ConsoleService::log("Task created");
                    self.socket.replace(task.unwrap());
                    self.state = String::from("Connected, waiting for data...");
                }
                true
            }
            Msg::UpdateGraph => {
                let mut first: Option<i64> = None;
                let mut last: Option<i64> = None;

                // Find the first and last datapoints in dataset
                for (_, (_, dataset)) in &self.temperature_dataset {
                    if !dataset.is_empty() {
                        if let Some(f) = first {
                            first.replace(core::cmp::min(dataset[0].0, f));
                        } else {
                            first.replace(dataset[0].0);
                        }

                        if let Some(l) = last {
                            last.replace(core::cmp::min(dataset[dataset.len() - 1].0, l));
                        } else {
                            last.replace(dataset[dataset.len() - 1].0);
                        }
                    }
                }

                if first.is_some() && last.is_some() {
                    let start_date = Utc.timestamp(first.unwrap(), 0);
                    let end_date = Utc.timestamp(last.unwrap(), 0);

                    let backend = CanvasBackend::new("temperature").expect("cannot find canvas");
                    let root = backend.into_drawing_area();
                    let font: FontDesc = ("sans-serif", 16.0).into();

                    root.fill(&WHITE).unwrap();

                    let mut chart = ChartBuilder::on(&root)
                        .margin(20)
                        .caption("Temperature", ("sans-serif", 40))
                        .set_label_area_size(LabelAreaPosition::Left, 40)
                        .set_label_area_size(LabelAreaPosition::Bottom, 40)
                        .build_cartesian_2d(start_date..end_date, -20.0..40.0)
                        .unwrap();

                    chart
                        .configure_mesh()
                        .x_labels(5)
                        .y_labels(5)
                        .draw()
                        .unwrap();

                    for (device, (color, dataset)) in &self.temperature_dataset {
                        if !dataset.is_empty() {
                            let series: LineSeries<_, _> = LineSeries::new(
                                dataset.iter().map(|(date, temp)| {
                                    let date: DateTime<Utc> = Utc.timestamp(*date, 0);
                                    (date, *temp)
                                }),
                                COLORS[*color],
                            );
                            ConsoleService::log(&format!(
                                "Drawing line graph for device '{}'",
                                device
                            ));
                            let c = *color;
                            chart.draw_series(series).unwrap().label(device).legend(
                                move |(x, y)| {
                                    PathElement::new(vec![(x, y), (20 + x, y)], &COLORS[c])
                                },
                            );
                        }
                    }
                    chart
                        .configure_series_labels()
                        .border_style(&BLACK)
                        .background_style(&WHITE.mix(0.8))
                        .draw()
                        .unwrap();
                    root.present().unwrap();
                    true
                } else {
                    false
                }
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
                <div>
                    <input placeholder="Url" value=self.url.clone() size="100" oninput=self.link.callback(|e: InputData| Msg::UpdateUrl(e.value)) />
                    <br />
                    <button onclick=self.link.callback(|_| Msg::Connect)>{ "Connect" }</button>
                </div>
                <p>{ self.state.clone() }</p>
                <div class="temperature">
                    <canvas id="temperature" height = "400px" width="1024px" />
                </div>
            </div>
        }
    }
}

const COLORS: [RGBColor; 7] = [BLACK, BLUE, CYAN, GREEN, MAGENTA, RED, YELLOW];

fn pick_random_color() -> usize {
    use rand::RngCore;
    let mut rng = rand::rngs::OsRng;
    let mut b: [u8; 1] = [0];
    rng.fill_bytes(&mut b);
    let num: usize = b[0] as usize % COLORS.len();
    num
}

fn main() {
    yew::start_app::<Model>();
}
