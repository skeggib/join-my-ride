use crate::{app::Context, atoms::button, orders::perform_cmd};
use seed::{prelude::*, *};

pub fn init(username: Option<String>) -> Model {
    Model {
        state: match username {
            Some(username) => State::SignedIn(SignedIn {
                username: username,
                logout_button: button::init("logout".into()),
            }),
            None => State::SignedOut,
        },
    }
}

pub struct Model {
    state: State,
}

enum State {
    SignedIn(SignedIn),
    SignedOut,
}

struct SignedIn {
    username: String,
    logout_button: button::Model,
}

pub enum Msg {
    Public(PublicMsg),
    Private(PrivateMsg),
}

pub enum PublicMsg {
    SignedOut,
}

pub enum PrivateMsg {
    LogoutButton(button::Msg),
    SignedOut,
}

pub fn update(
    msg: PrivateMsg,
    model: &mut Model,
    context: &mut Context,
    orders: &mut impl Orders<Msg>,
) {
    match msg {
        PrivateMsg::LogoutButton(button::Msg::Click) => logout(orders),
        PrivateMsg::SignedOut => {
            context.username = None;
            model.state = State::SignedOut;
            notify_logout(orders)
        }
    }
}

pub fn view(model: &Model) -> Node<Msg> {
    match &model.state {
        State::SignedIn(model) => div![
            model.username.clone(),
            button::view(&model.logout_button, true)
                .map_msg(PrivateMsg::LogoutButton)
                .map_msg(Msg::Private)
        ],
        State::SignedOut => div![a![attrs![At::Href => "/login"], "login"]],
    }
}

fn logout(orders: &mut impl Orders<Msg>) {
    perform_cmd(orders, async {
        // TODO: logout
        Msg::Private(PrivateMsg::SignedOut)
    });
}

fn notify_logout(orders: &mut impl Orders<Msg>) {
    perform_cmd(orders, async { Msg::Public(PublicMsg::SignedOut) });
}
