use async_trait::async_trait;
use backend::Backend;
use common::api::BackendApi;
use common::{Event, Id};
use debug_cell::RefCell;
use frontend::{
    app::{self, testable_update, Msg},
    orders::{MyOrders, OrdersStub},
};
use seed::app::subs::UrlChanged;
use seed::virtual_dom::At;
use seed::{
    virtual_dom::{Node, Tag},
    Url,
};
use std::borrow::BorrowMut;
use std::rc::Rc;
use std::str::FromStr;

fn get_login_url(root: &Node<Msg>) -> Option<&str> {
    let login_a = find_node(&root, &|node| {
        node.is_el()
            && node.el().unwrap().tag == Tag::A
            && !node.el().unwrap().children.is_empty()
            && node.el().unwrap().children.first().unwrap().is_text()
            && node
                .el()
                .unwrap()
                .children
                .first()
                .unwrap()
                .text()
                .unwrap()
                .text
                == "login"
    })
    .unwrap()
    .el()
    .unwrap();
    let login_href = login_a.attrs.vals.get(&At::Href).unwrap();
    if let seed::virtual_dom::values::AtValue::Some(login_url) = login_href {
        Some(login_url)
    } else {
        None
    }
}

#[test]
fn clicking_join_on_event_adds_user_to_participants() {
    let messages: Rc<RefCell<Vec<Msg>>> = Rc::new(RefCell::new(vec![]));
    let messages_clone = messages.clone();
    let orders = Rc::new(RefCell::new(MyOrders::new(
        frontend::orders::OrdersImplementation::<Msg, Msg>::Stub(OrdersStub::new(Rc::new(
            RefCell::new(move |msg| messages_clone.as_ref().borrow_mut().push(msg)),
        ))),
    )));
    let backend = SyncBackend::new();

    let app_ = Rc::new(RefCell::new(app::testable_init(
        Url::new(),
        &mut *orders.as_ref().borrow_mut(),
        Rc::new(backend),
    )));

    let mut cloned_app_ = app_.clone();
    let mut cloned_orders = orders.clone();
    let update = Rc::new(RefCell::new(move |msg| {
        let app_ = cloned_app_.borrow_mut();
        let orders = cloned_orders.borrow_mut();
        testable_update(
            msg,
            &mut app_.as_ref().borrow_mut(),
            &mut *orders.as_ref().borrow_mut(),
        );
    }));

    for message in messages.borrow().iter() {
        (update.as_ref().borrow_mut())(message.clone());
    }
    messages.as_ref().borrow_mut().clear();
    match orders.as_ref().borrow_mut().implementation {
        frontend::orders::OrdersImplementation::Container(_) => todo!(),
        frontend::orders::OrdersImplementation::Proxy(_) => todo!(),
        frontend::orders::OrdersImplementation::Mock(_) => todo!(),
        frontend::orders::OrdersImplementation::Stub(ref mut stub) => {
            stub.update = update;
        }
        frontend::orders::OrdersImplementation::StubProxy(_) => todo!(),
    };

    // given a logged-in user
    {
        let view = app::view(&app_.borrow());
        let login_url = get_login_url(&view).unwrap();
        app::testable_update(
            app::Msg::UrlChanged(UrlChanged(Url::from_str(login_url).unwrap())),
            &mut app_.as_ref().borrow_mut(),
            &mut *orders.as_ref().borrow_mut(),
        );
    }

    // and given the displayed page is an event
    {
        let view = app::view(&app_.borrow());
        let event_url = find_node(&view, &|node| {
            node.is_el()
                && node.el().unwrap().tag == Tag::A
                && !node.el().unwrap().children.is_empty()
                && node.el().unwrap().children.first().unwrap().is_text()
                && node
                    .el()
                    .unwrap()
                    .children
                    .first()
                    .unwrap()
                    .text()
                    .unwrap()
                    .text
                    == "event_1"
        });
        assert!(event_url.is_some());
        println!("{}", event_url.unwrap().to_string());
    }
    // app::testable_update(app::Msg::UrlChanged(UrlChanged(Url::from_str(format!("/event/{}", evend_id)))), model, orders)

    // when the user clicks on the join button of an event

    // then the user's name is added to the participants of the event

    // and then other users can see that the user participates in the event
}

fn find_node<'a, F>(node: &'a Node<Msg>, predicate: &F) -> Option<&'a Node<Msg>>
where
    F: Fn(&'a Node<Msg>) -> bool,
{
    if predicate(node) {
        Some(node)
    } else if let Node::Element(el) = node {
        for child in &el.children {
            let result = find_node(&child, predicate);
            match result {
                Some(some) => return Some(some),
                None => { /* do nothing */ }
            }
        }
        None
    } else {
        None
    }
}

pub struct SyncBackend {
    backend: Backend,
}

impl SyncBackend {
    pub fn new() -> SyncBackend {
        SyncBackend {
            backend: Backend::new(),
        }
    }
}

#[async_trait(?Send)]
impl BackendApi for SyncBackend {
    async fn get_events(self: &Self) -> Result<Vec<Event>, String> {
        Ok(self.backend.get_events())
    }
    async fn get_event(self: &Self, id: Id) -> Result<Event, String> {
        self.backend
            .get_event(&id.to_string())
            .ok_or(format!("could not find event with id {}", id))
    }
    async fn publish_event(self: &Self, name: String) -> Result<(), String> {
        self.backend.publish_event(Event::new(name));
        Ok(())
    }
    async fn join_event(self: &Self, id: Id) -> Result<(), String> {
        self.backend.join_event(&id.to_string(), "user");
        Ok(())
    }
}
