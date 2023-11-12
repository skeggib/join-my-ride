use std::pin::Pin;
use std::rc::Rc;

use async_trait::async_trait;
use common::api::{BackendApi, MockBackendApi};
use common::{Event, Id};
use frontend::app::{self, Msg, Page};
use frontend::orders::{MyOrders, OrdersImplementation, OrdersMock};
use frontend::pages::main::{self, State};
use futures::Future;
use seed::Url;

#[test]
fn main_page_requests_for_all_events_and_displays_them() {
    let mut orders = MyOrders::new(OrdersImplementation::<Msg, Msg>::Mock(OrdersMock::new()));

    // expect the front-end to request all events
    let mut backend = MockBackendApi::new();
    backend
        .expect_get_events()
        .returning(|| Ok(vec![Event::new("event".into())]));

    // given a new app is being initialized
    let mut app_ = app::testable_init(Url::new(), &mut orders, Rc::new(backend));

    // when the backend responds with events
    println!("{:?}", orders.mock().unwrap().messages());
    // assert!(matches!(orders.mock().unwrap().messages().last(), Some(app::Msg::Main(main::Msg::OnGetEventsResponse(..))))); // TODO: uncomment and fix
    app::testable_update(
        app::Msg::Main(main::Msg::OnGetEventsResponse(vec![Event::new(
            "event".into(),
        )])),
        &mut app_,
        &mut orders,
    );

    // then the displayed page is main in the loaded state
    assert!(matches!(
        app_.page,
        Page::Main(main::Model {
            state: State::Loaded(..)
        })
    ));

    // and then the page contains the events returned by the backend
    // TODO: query html
}
