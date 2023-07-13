use crate::{app::Context, orders::perform_cmd};
use seed::{prelude::*, *};

use crate::atoms::{button, input};

pub fn init(
    url: &mut Url,
    target_url: Option<Url>,
    context: &Context,
    orders: &mut impl Orders<Msg>,
) -> Model {
    let stage = match &context.username {
        Some(_) => Stage::SignedIn,
        None => Stage::SignedOut(SignedOut {
            username_input: input::init("username".into()),
            login_button: button::init("login".into()),
        }),
    };
    Model {
        stage: stage,
        url: target_url,
    }
}

pub struct Model {
    stage: Stage,
    url: Option<Url>,
}

pub enum Stage {
    SignedOut(SignedOut),
    LoggingIn,
    SignedIn,
}

struct SignedOut {
    username_input: input::Model,
    login_button: button::Model,
}

pub enum Msg {
    Public(PublicMsg),
    Private(PrivateMsg),
}

pub enum PublicMsg {
    LoggedIn(String, Option<Url>),
}

pub enum PrivateMsg {
    UsernameInput(input::Msg),
    LoginButton(button::Msg),
    LoggedIn(String),
}

pub fn update(
    msg: PrivateMsg,
    model: &mut Model,
    context: &mut Context,
    orders: &mut impl Orders<Msg>,
) {
    match msg {
        PrivateMsg::UsernameInput(msg) => username_input_msg(msg, model, context, orders),
        PrivateMsg::LoginButton(msg) => login_button_msg(msg, model, context, orders),
        PrivateMsg::LoggedIn(username) => logged_in_msg(username, model, context, orders),
    }
}

fn username_input_msg(
    msg: input::Msg,
    model: &mut Model,
    context: &mut Context,
    orders: &mut impl Orders<Msg>,
) {
    match &mut model.stage {
        Stage::SignedOut(model) => match msg {
            input::Msg::ValueChange(value) => model.username_input.value = value,
        },
        Stage::LoggingIn => error!("received username input msg while logging in"),
        Stage::SignedIn => error!("received username input msg while signed in"),
    }
}

fn login_button_msg(
    msg: button::Msg,
    model: &mut Model,
    context: &mut Context,
    orders: &mut impl Orders<Msg>,
) {
    match &mut model.stage {
        Stage::SignedOut(signed_out) => match msg {
            button::Msg::Click => {
                if signed_out.username_input.value.is_empty() {
                    error!("received login button msg but username input is empty")
                } else {
                    login(signed_out.username_input.value.clone(), orders);
                    model.stage = Stage::LoggingIn;
                }
            }
        },
        Stage::LoggingIn => error!("received login button msg while logging in"),
        Stage::SignedIn => error!("received login button msg while signed in"),
    }
}

fn logged_in_msg(
    username: String,
    model: &mut Model,
    context: &mut Context,
    orders: &mut impl Orders<Msg>,
) {
    context.username = Some(username.clone());
    model.stage = Stage::SignedIn;
    notify_login(username, model.url.clone(), orders);
}

pub fn view(model: &Model) -> Node<Msg> {
    div![
        h1!(a![attrs![At::Href => "/"], "join my ride"]),
        h2!("login"),
        match &model.stage {
            Stage::SignedOut(signed_out) => {
                div![
                    input::view(&signed_out.username_input)
                        .map_msg(PrivateMsg::UsernameInput)
                        .map_msg(Msg::Private),
                    button::view(
                        &signed_out.login_button,
                        !signed_out.username_input.value.is_empty()
                    )
                    .map_msg(PrivateMsg::LoginButton)
                    .map_msg(Msg::Private)
                ]
            }
            Stage::LoggingIn => {
                div!["logging in..."]
            }
            Stage::SignedIn => {
                div!["already signed-in"]
            }
        }
    ]
}

fn login(username: String, orders: &mut impl Orders<Msg>) {
    perform_cmd(orders, async {
        // TODO: login
        Msg::Private(PrivateMsg::LoggedIn(username))
    });
}

fn notify_login(username: String, url: Option<Url>, orders: &mut impl Orders<Msg>) {
    perform_cmd(orders, async {
        Msg::Public(PublicMsg::LoggedIn(username, url))
    });
}
