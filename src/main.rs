use anyhow::Error;
use chrono::{DateTime, FixedOffset, TimeZone, Utc};
use patternfly_yew::*;
use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use serde_json::Value;
use std::collections::{HashMap, VecDeque};
use yew::prelude::*;
use yew::services::{
    websocket::{WebSocketService, WebSocketStatus, WebSocketTask},
    ConsoleService,
};
use yew_router::prelude::*;

mod chart;
use chart::*;
mod index;
use index::*;

struct Model {}

#[derive(Switch, Debug, Clone, PartialEq)]
pub enum AppRoute {
    #[to = "/temperature"]
    Temperature,
    #[to = "/"]
    Index,
}

impl Component for Model {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        true
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
            <BackdropViewer/>
            <ToastViewer/>

            <Router<AppRoute, ()>
                redirect = Router::redirect(|_|AppRoute::Index)
                render = Router::render(|switch: AppRoute| {
                    match switch {
                        AppRoute::Temperature  => Self::page(html!{<Chart/>}),
                        AppRoute::Index => Self::page(html!{<Index/>}),
                    }
                })
             />
            </>
        }
    }
}

impl Model {
    fn page(html: Html) -> Html {
        let sidebar = html_nested! {
            <PageSidebar>
                <Nav>
                    <NavRouterExpandable<AppRoute> title="Websocket Integration">
                        <NavRouterItem<AppRoute> to=AppRoute::Index>{"Index"}</NavRouterItem<AppRoute>>
                        <NavRouterItem<AppRoute> to=AppRoute::Temperature>{"Temperature"}</NavRouterItem<AppRoute>>
                        <NavItem external=true to="https://github.com/ctron/patternfly-yew">{"PatternFly Yew"}</NavItem>
                    </NavRouterExpandable<AppRoute>>
                </Nav>
            </PageSidebar>
        };

        html! {
            <Page
                logo={html_nested!{
                    <Logo src="https://www.patternfly.org/assets/images/PF-Masthead-Logo.svg" alt="Patternfly Logo" />
                }}
                sidebar=sidebar
                >
                { html }
            </Page>
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}
