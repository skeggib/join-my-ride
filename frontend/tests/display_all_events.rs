use common::api::MockBackendApi;
use common::Event;
use frontend::app::{self, Msg, Page};
use frontend::orders::{MyOrders, OrdersImplementation, OrdersMock};
use frontend::pages::main::{self, State};
use seed::Url;
use std::rc::Rc;

#[test]
fn main_page_requests_all_events_and_displays_them() {
    let mut orders = MyOrders::new(OrdersImplementation::<Msg, Msg>::Mock(OrdersMock::new()));

    // expect the front-end to request all events
    let mut backend = MockBackendApi::new();
    backend
        .expect_get_events()
        .returning(|| Ok(vec![Event::new("event".into())]));

    // given a new app is being initialized
    let mut app_ = app::testable_init(Url::new(), &mut orders, Rc::new(backend));

    // when the backend responds with events
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
