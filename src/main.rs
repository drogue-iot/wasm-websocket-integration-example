#![recursion_limit = "512"]
use patternfly_yew::*;
use yew::prelude::*;
use yew_router::prelude::*;

mod chart;
use chart::*;

struct Model {}

#[derive(Switch, Debug, Clone, PartialEq)]
pub enum AppRoute {
    #[to = "/temperature"]
    Temperature,
}

impl Component for Model {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
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
                redirect = Router::redirect(|_|AppRoute::Temperature)
                render = Router::render(|switch: AppRoute| {
                    match switch {
                        AppRoute::Temperature  => Self::page(html!{<Chart url = "wss://ws-integration.sandbox.drogue.cloud/drogue-public-temperature" />}),
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
                        <NavRouterItem<AppRoute> to=AppRoute::Temperature>{"Temperature"}</NavRouterItem<AppRoute>>
                    </NavRouterExpandable<AppRoute>>
                </Nav>
            </PageSidebar>
        };

        html! {
            <Page
                logo={html_nested!{
                    <Logo src="/logo.png" alt="Drogue IoT Logo" />
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
