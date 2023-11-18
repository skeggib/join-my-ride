use common::api::MockBackendApi;
use common::Event;
use frontend::app::{self, Msg, Page};
use frontend::orders::{MyOrders, OrdersImplementation, OrdersMock};
use frontend::pages::main::{self, State};
use seed::Url;
use std::rc::Rc;

mod html_query;
use html_query::assert_contains_text;

#[test]
fn main_page_requests_all_events_and_displays_them() {
    let mut orders = MyOrders::new(OrdersImplementation::<Msg, Msg>::Mock(OrdersMock::new()));

    let event_1 = Event::new("event 1 name".into());
    let event_2 = Event::new("event 2 name".into());

    // expect the front-end to request all events
    let mut backend = MockBackendApi::new();
    let events_backend_mock = vec![event_1.clone(), event_2.clone()];
    backend
        .expect_get_events()
        .returning(move || Ok(events_backend_mock.clone())); // TODO: why do I need to move AND copy events_backend_mock?

    // given a new app is being initialized
    let mut app_ = app::testable_init(Url::new(), &mut orders, Rc::new(backend));

    // when the backend responds with events
    // assert!(matches!(orders.mock().unwrap().messages().last(), Some(app::Msg::Main(main::Msg::OnGetEventsResponse(..))))); // TODO: uncomment and fix
    app::testable_update(
        app::Msg::Main(main::Msg::OnGetEventsResponse(vec![
            event_1.clone(),
            event_2.clone(),
        ])),
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
    let view = app::view(&app_);
    assert_contains_text(&view, &event_1.name);
    assert_contains_text(&view, &event_2.name);
}
