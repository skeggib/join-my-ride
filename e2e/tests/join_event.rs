use crate::app::AppMsg;
use async_trait::async_trait;
use backend::Backend;
use common::api::BackendApi;
use common::{Event, Id};
use debug_cell::RefCell;
use frontend::app::AppMsg as Msg;
use frontend::url::Url;
use frontend::{
    app::{self},
    orders::{MyOrders, OrdersStub},
};
use seed::virtual_dom::{At, AtValue};
use seed::virtual_dom::{Node, Tag};
use std::fmt::Display;
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
    // since the frontend and orders stub can only be borrowed once at a time,
    // we need to finish processing an update before starting another. this
    // means that a produced message during an update needs to be stored and
    // processed after the update

    // list of produced messages that need to be handled after an update
    let messages: Rc<RefCell<Vec<Msg>>> = Rc::new(RefCell::new(vec![]));

    // orders stub that stores produced messages in a list
    let messages_clone = messages.clone();
    let orders = Rc::new(RefCell::new(MyOrders::new(
        frontend::orders::OrdersImplementation::<Msg, Msg>::Stub(OrdersStub::new(Rc::new(
            RefCell::new(move |msg| messages_clone.as_ref().borrow_mut().push(msg)),
        ))),
    )));

    let backend = SyncBackend::new();

    // instantiate a new frontend
    let frontend = Rc::new(RefCell::new(app::testable_init(
        Url::new(),
        &mut *orders.as_ref().borrow_mut(),
        Rc::new(backend),
    )));

    // processes all produced messages, should be called after each call to init or update
    let process_messages = || {
        while !messages.borrow().is_empty() {
            // clone messages and empty the messages list to allow storing newly produced messages
            let messages_clone = messages.borrow().clone();
            messages.as_ref().borrow_mut().clear();
            for message in messages_clone {
                app::testable_update(
                    message.clone(),
                    &mut frontend.as_ref().borrow_mut(),
                    &mut *orders.as_ref().borrow_mut(),
                );
            }
        }
    };

    process_messages();

    // given a logged-in user
    {
        let view = app::testable_view(&frontend.borrow());
        let login_url = get_login_url(&view).unwrap();
        app::testable_update(
            Msg::UrlChanged(Url::from_str(login_url).unwrap()),
            &mut frontend.as_ref().borrow_mut(),
            &mut *orders.as_ref().borrow_mut(),
        );
        process_messages();
        app::testable_update(
            Msg::Login(frontend::pages::login::Msg::Private(
                frontend::pages::login::PrivateMsg::UsernameInput(
                    frontend::atoms::input::Msg::ValueChange("user".into()),
                ),
            )),
            &mut frontend.as_ref().borrow_mut(),
            &mut *orders.as_ref().borrow_mut(),
        );
        process_messages();
        app::testable_update(
            Msg::Login(frontend::pages::login::Msg::Private(
                frontend::pages::login::PrivateMsg::LoginButton(
                    frontend::atoms::button::Msg::Click,
                ),
            )),
            &mut frontend.as_ref().borrow_mut(),
            &mut *orders.as_ref().borrow_mut(),
        );
        process_messages();
        // TODO: check that the user is logged in
    }

    // and given the displayed page is an event
    {
        let view = app::testable_view(&frontend.borrow());
        let event_link = find_node(&view, &|node| {
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
        assert!(
            event_link.is_some(),
            "the view does not contain a node with the text 'event_1':\n{}",
            indent(&view)
        );
        let event_url = if let AtValue::Some(url) = event_link
            .unwrap()
            .el()
            .unwrap()
            .attrs
            .vals
            .get(&At::Href)
            .unwrap()
        {
            Some(url)
        } else {
            None
        };
        app::testable_update(
            Msg::UrlChanged(Url::from_str(&event_url.unwrap()).unwrap()),
            &mut frontend.as_ref().borrow_mut(),
            &mut *orders.as_ref().borrow_mut(),
        );
        process_messages();
        // TODO: check that the current page is the event's page
    }

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
        self.backend
            .join_event(&id.to_string(), "user")
            .map_err(|err| err.0)
    }
}

fn indent(node: &Node<AppMsg>) -> String {
    IndentedHtml { node: node }.to_string()
}

struct IndentedHtml<'a> {
    node: &'a Node<AppMsg>,
}

impl<'a> IndentedHtml<'a> {
    fn write_node(
        node: &'a Node<AppMsg>,
        f: &mut std::fmt::Formatter<'_>,
        indentation: usize,
    ) -> std::fmt::Result {
        match node {
            Node::Element(el) => {
                let tag = el.tag.to_string();
                let mut open_tag = format!("<{}", &tag);
                let mut attrs = el.attrs.clone();
                let style = el.style.to_string();
                if !style.is_empty() {
                    attrs.add(At::Style, style);
                }
                if let Some(namespace) = el.namespace.as_ref() {
                    attrs.add(At::Xmlns, namespace.as_str());
                }
                let attributes = attrs.to_string();
                if !attributes.is_empty() {
                    open_tag += &format!(" {}", attributes);
                }
                open_tag += ">";
                if el.children.len() > 1 {
                    open_tag += "\n";
                }
                write!(f, "{}{}", "  ".repeat(indentation), open_tag)?;

                if el.children.len() > 1 {
                    for child in &el.children {
                        IndentedHtml::write_node(child, f, indentation + 1)?;
                        write!(f, "\n")?;
                    }
                    write!(f, "{}</{}>", "  ".repeat(indentation), tag)?;
                } else {
                    for child in &el.children {
                        IndentedHtml::write_node(child, f, 0)?;
                    }
                    write!(f, "</{}>", tag)?;
                }

                Ok(())
            }
            Node::Text(text) => write!(f, "{}{}", "  ".repeat(indentation), text),
            Node::Empty => write!(f, ""),
            Node::NoChange => write!(f, ""),
        }
    }
}

impl<'a> Display for IndentedHtml<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        IndentedHtml::write_node(self.node, f, 0)
    }
}
