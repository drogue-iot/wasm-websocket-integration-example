use anyhow::Error;
use chrono::{DateTime, FixedOffset, Timelike};
use patternfly_yew::{
    Bullseye, Button, Content, EmptyState, Icon, PageSection, PageSectionVariant, Size, TextInput,
    Title, Toolbar, ToolbarElementModifier, ToolbarGroup, ToolbarItem, Variant, WithBreakpointExt,
};
use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use serde_json::Value;
use std::collections::{HashMap, VecDeque};
use yew::prelude::*;
use yew::services::{
    websocket::{WebSocketService, WebSocketStatus, WebSocketTask},
    ConsoleService,
};

pub enum Msg {
    Connect,
    Disconnect,
    UpdateUrl(String),
    UpdateGraph,
    Data(Result<String, Error>),
    Ignore,
}

#[derive(Clone, Debug, Properties, PartialEq)]
pub struct Props {
    pub url: String,
}

pub struct Chart {
    // It can be used to send messages to the component
    link: ComponentLink<Self>,
    socket: Option<WebSocketTask>,
    temperature_dataset: HashMap<String, (usize, VecDeque<(DateTime<FixedOffset>, f64)>)>,
    state: State,
    props: Props,
    total_received: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
    Disconnected,
    Connecting,
    Connected,
}

const SCHEMA: &str = "urn:drogue:iot:temperature";
const CAPACITY: usize = 100;

impl Component for Chart {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            socket: None,
            temperature_dataset: HashMap::new(),
            state: State::Disconnected,
            props,
            total_received: 0,
            //url: "wss://ws-integration.sandbox.drogue.cloud/drogue-public-temperature".to_string(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Ignore => false,
            Msg::UpdateUrl(url) => {
                self.props.url = url;
                true
            }
            Msg::Data(data) => {
                match data {
                    Ok(s) => {
                        ConsoleService::log("Data updated");
                        let json: Value = serde_json::from_str(s.as_str()).unwrap();

                        ConsoleService::log(&format!("Received: {:?}", &json));
                        if let Some(SCHEMA) = json["dataschema"].as_str() {
                            self.total_received += 1;
                            json["time"].as_str().map(|time| {
                                ConsoleService::log(&format!("Time: {:?}", time));
                                let _ = DateTime::parse_from_rfc3339(&time).map(|r| {
                                    ConsoleService::log(&format!(
                                        "Successfully parsed, timestamp: {}",
                                        r.timestamp()
                                    ));
                                    if let Some(device) = json["device"].as_str() {
                                        let (_, dataset) =
                                            if self.temperature_dataset.contains_key(device) {
                                                self.temperature_dataset.get_mut(device).unwrap()
                                            } else {
                                                let color =
                                                    self.temperature_dataset.len() % COLORS.len();
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
                                                    dataset.push_back((r, temp));
                                                });
                                            }
                                            Value::String(s) => {
                                                let s = s.as_str();
                                                if let Ok(temp) = s.parse::<f64>() {
                                                    if dataset.len() >= CAPACITY {
                                                        dataset.pop_front();
                                                    }
                                                    dataset.push_back((r, temp));
                                                }
                                            }
                                            _ => {}
                                        }
                                        self.link.send_message(Msg::UpdateGraph);
                                    }
                                });
                            });
                        }
                    }
                    Err(e) => {
                        ConsoleService::log(format!("Data ERROR: {}", e).as_str());
                    }
                }
                true
            }
            Msg::Disconnect => {
                self.socket.take();
                self.state = State::Disconnected;
                true
            }
            Msg::Connect => {
                self.total_received = 0;
                self.temperature_dataset.clear();
                self.state = State::Connecting;
                let on_data = self.link.callback(|data| Msg::Data(data));
                let on_notify = self.link.callback(move |input| {
                    ConsoleService::log(&format!("Notification: {:?}", input));
                    match input {
                        WebSocketStatus::Closed => Msg::Connect,
                        _ => Msg::Ignore,
                    }
                });
                if self.socket.is_none() {
                    let task = WebSocketService::connect_text(&self.props.url, on_data, on_notify);
                    ConsoleService::log("Task created");
                    self.socket.replace(task.unwrap());
                    self.state = State::Connected;
                }
                true
            }
            Msg::UpdateGraph => {
                let mut first: Option<DateTime<FixedOffset>> = None;
                let mut last: Option<DateTime<FixedOffset>> = None;

                // Find the first and last datapoints in dataset
                for (_, (_, dataset)) in &self.temperature_dataset {
                    if !dataset.is_empty() {
                        if let Some(f) = first {
                            if dataset[0].0 < f {
                                first.replace(dataset[0].0);
                            }
                        } else {
                            first.replace(dataset[0].0);
                        }

                        if let Some(l) = last {
                            if dataset[dataset.len() - 1].0 > l {
                                last.replace(dataset[dataset.len() - 1].0);
                            }
                        } else {
                            last.replace(dataset[dataset.len() - 1].0);
                        }
                    }
                }

                if first.is_some() && last.is_some() {
                    let start_date = first.unwrap();
                    let end_date = last.unwrap();

                    let backend = CanvasBackend::new("temperature").expect("cannot find canvas");
                    let root = backend.into_drawing_area();

                    root.fill(&WHITE).unwrap();

                    let mut chart = ChartBuilder::on(&root)
                        //.caption("Temperature", ("sans-serif", 32))
                        .margin(5)
                        .set_label_area_size(LabelAreaPosition::Left, 60)
                        .set_label_area_size(LabelAreaPosition::Bottom, 40)
                        .build_cartesian_2d(start_date..end_date, -10.0..40.0)
                        .unwrap();

                    chart
                        .configure_mesh()
                        .disable_x_mesh()
                        .x_labels(8)
                        .y_desc("Temperature (℃)")
                        .x_desc("Time")
                        .x_label_formatter(&|x| {
                            format!("{:02}:{:02}:{:02}", x.hour(), x.minute(), x.second())
                        })
                        .draw()
                        .unwrap();

                    for (device, (color, dataset)) in &self.temperature_dataset {
                        if !dataset.is_empty() {
                            let series: LineSeries<_, _> = LineSeries::new(
                                dataset.iter().map(|(date, temp)| {
                                    let date: DateTime<_> = *date;
                                    (date, *temp)
                                }),
                                COLORS[*color],
                            )
                            .point_size(2);
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
                        .position(SeriesLabelPosition::UpperRight)
                        .border_style(&BLACK)
                        .background_style(&WHITE.mix(0.5))
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
            <>
            <PageSection variant=PageSectionVariant::Light limit_width=true>
                <Content>
                    <Title>{"Temperature Monitor"}</Title>
                </Content>
            </PageSection>
            <PageSection>
           <Toolbar>
               <ToolbarGroup>

                   { if self.state == State::Disconnected { html!{
                   <ToolbarItem>
                       <TextInput
                           value=self.props.url.clone(),
                           onchange=self.link.callback(|url|Msg::UpdateUrl(url))
                           required=true,
                           r#type="url".to_string(),
                           placeholder="Websocket URL to consume events from"/>
                   </ToolbarItem>
                   }} else { html!{} }}

                   <ToolbarItem>
                       {if self.state != State::Disconnected {
                           html!{<Button
                                   label="Disconnected"
                                   icon=Icon::Pause
                                   variant=Variant::Secondary
                                   onclick=self.link.callback(|_|Msg::Disconnect)
                           />}
                       } else {
                           html!{<Button
                                   label="Connect"
                                   icon=Icon::Play
                                   variant=Variant::Primary
                                   onclick=self.link.callback(|_|Msg::Connect)
                           />}
                       }}
                   </ToolbarItem>
               </ToolbarGroup>
               <ToolbarItem modifiers=vec![ToolbarElementModifier::Left.all()]>
                   {
                       html!{
                           <strong>{"State: "}{format!("{:?}", self.state)}</strong>
                       }
                   }
               </ToolbarItem>
               <ToolbarItem modifiers=vec![ToolbarElementModifier::Right.all()]>
                   {
                       html!{
                           <strong>{"Events received: "}{self.total_received}</strong>
                       }
                   }
               </ToolbarItem>
           </Toolbar>
            </PageSection>
            <PageSection>
               { if self.temperature_dataset.is_empty() {
                   html! {
                       <div style="width: 100%; height: 100%;">
                           {self.render_empty() }
                           <canvas id="temperature" width="1024px" height="768px" />
                       </div>
                   }
               } else {
                   html! {
                       <div style="width: 100%; height: 100%;">
                           <canvas id="temperature" width="1024px" height="768px" />
                       </div>
                   }
               }
               }
            </PageSection>
            </>
        }
    }
}

impl Chart {
    fn render_empty(&self) -> Html {
        return html! {
            <div style="padding-bottom: 10rem; height: 100%;">
            <Bullseye>
            <EmptyState
                title="No events"
                icon=Icon::Pending
                size=Size::XLarge
                >
                { "The " } <q> {"graph "} </q> { " will only draw when messages are received.
                When the messages arrive, you will see it right here." }
            </EmptyState>
            </Bullseye>
            </div>
        };
    }
}

const COLORS: [RGBColor; 7] = [BLACK, BLUE, CYAN, GREEN, MAGENTA, RED, YELLOW];
