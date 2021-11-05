mod moving_avg;

use anyhow::Error;
use chrono::{DateTime, TimeZone, Utc};
use plotters::coord::types::RangedCoordf64;
use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use serde_json::Value;
use std::collections::VecDeque;
use yew::prelude::*;
use yew::services::{
    websocket::{WebSocketService, WebSocketTask},
    ConsoleService,
};

use crate::moving_avg::MovingAverage;

enum Msg {
    Connect,
    UpdateUrl(String),
    Data(Result<String, Error>),
    Ignore,
}

struct Model {
    // It can be used to send messages to the component
    link: ComponentLink<Self>,
    socket: Option<WebSocketTask>,
    temperature_values: MovingAverage<f64>,
    temperature_dataset: VecDeque<(i64, f64)>,
    average: f64,
    trend: String,
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
            temperature_dataset: VecDeque::with_capacity(CAPACITY),
            temperature_values: MovingAverage::new(5),
            average: 0.0,
            trend: String::from("No data"),
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
                match data {
                    Ok(s) => {
                        ConsoleService::log("Data updated");
                        let json: Value = serde_json::from_str(s.as_str()).unwrap();

                        json["time"].as_str().map(|time| {
                            ConsoleService::log(&format!("Time: {:?}", time));
                            DateTime::parse_from_rfc3339(&time).map(|r| {
                                ConsoleService::log(&format!(
                                    "Successfully parsed, timestamp: {}",
                                    r.timestamp()
                                ));
                                json["data"]["temp"].as_f64().map(|temp| {
                                    self.temperature_dataset.push_back((r.timestamp(), temp));
                                    if self.temperature_dataset.len() > CAPACITY {
                                        self.temperature_dataset.pop_front();
                                    }

                                    let first = self.temperature_dataset[0].0;
                                    let last = self.temperature_dataset
                                        [self.temperature_dataset.len() - 1]
                                        .0;

                                    let start_date = Utc.timestamp(first, 0);
                                    let end_date = Utc.timestamp(last, 0);

                                    let backend = CanvasBackend::new("temperature")
                                        .expect("cannot find canvas");
                                    let root = backend.into_drawing_area();
                                    let font: FontDesc = ("sans-serif", 20.0).into();

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
                                        .x_labels(3)
                                        .y_labels(3)
                                        .draw()
                                        .unwrap();

                                    let series: LineSeries<_, _> = LineSeries::new(
                                        (0..).zip(self.temperature_dataset.iter()).map(
                                            |(idx, (date, temp))| {
                                                let date: DateTime<Utc> = Utc.timestamp(*date, 0);
                                                (date, *temp)
                                            },
                                        ),
                                        &RED,
                                    );

                                    chart.draw_series(series).unwrap();

                                    root.present().unwrap();
                                });
                            });
                        });
                        let _ = json["data"]["temp"].as_f64().map(|temp| {
                            self.update_data(temp);
                        });
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
                    let task = WebSocketService::connect_text(&self.url, on_data, on_notify);
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
                <div>
                    <input placeholder="Url" value=self.url.clone() size="100" oninput=self.link.callback(|e: InputData| Msg::UpdateUrl(e.value)) />
                    <br />
                    <button onclick=self.link.callback(|_| Msg::Connect)>{ "Connect" }</button>
                </div>
                <p>{"Last received value: "}{ self.temperature_values.last() }</p>
                <p>{"Last 5 values average: "}{ self.average }</p>
                <p>{ self.trend.clone() }</p>
                <div class="temperature">
                    <canvas id="temperature" height = "600px" width="800px" />
                </div>
            </div>
        }
    }
}

impl Model {
    fn update_data(&mut self, new_value: f64) {
        let average = self.temperature_values.add(new_value).clone();

        self.trend = if average > self.average {
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
