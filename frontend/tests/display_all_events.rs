use common::api::MockBackendApi;
use common::Event;
use frontend::app::{self, Msg, Page};
use frontend::orders::{MyOrders, OrdersImplementation, OrdersMock};
use frontend::pages::main::{self, State};
use seed::virtual_dom::{At, El, Node};
use seed::Url;
use std::fmt::Display;
use std::rc::Rc;

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
        app::Msg::Main(main::Msg::OnGetEventsResponse(vec![event_1, event_2])),
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
    assert!(matches!(
        get_element_by_contents(&view, "event 1 name"),
        Some(..)
    ));
    assert!(matches!(
        get_element_by_contents(&view, "event 2 name"),
        Some(..)
    ));
}

/// Get the first element of type seed::virtual_dom::El containing a Text node which text contains `contents`
fn get_element_by_contents<'a>(node: &'a Node<Msg>, contents: &str) -> Option<&'a El<Msg>> {
    // search children only if the current node is an element
    if let Node::Element(current_el) = node {
        // the current element has a text child containing `contents` -> return it
        if current_el
            .children
            .iter()
            .filter(|child| child.is_text() && child.text().unwrap().text.contains(contents))
            .count()
            > 0
        {
            Some(current_el)
        }
        // check if any child element has a text child containing `contents`
        else {
            current_el
                .children
                .iter()
                .filter(|child| child.is_el())
                .map(|child| get_element_by_contents(child, contents))
                .find(|child| child.is_some())?
        }
    } else {
        None
    }
}
