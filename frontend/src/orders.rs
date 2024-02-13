use std::{
    any::Any,
    convert::identity,
    rc::Rc,
};
use debug_cell::RefCell;
use futures::{executor::block_on, Future};
use seed::{
    app::{OrdersContainer, OrdersProxy},
    prelude::Orders,
    virtual_dom::Node,
};

use crate::app::Model;

pub struct OrdersMock<Ms, Model, Node> {
    messages: Vec<Ms>,
    _model: Vec<Model>,
    _node: Vec<Node>,
}

// methods called in MyOrders
impl<Ms: 'static, Model, Node> OrdersMock<Ms, Model, Node> {
    pub fn new() -> Self {
        OrdersMock {
            messages: vec![],
            _model: vec![],
            _node: vec![],
        }
    }

    fn proxy<ChildMs: 'static>(
        &mut self,
        _f: impl FnOnce(ChildMs) -> Ms + 'static + Clone,
    ) -> OrdersMock<ChildMs, Model, Node> {
        OrdersMock::new() // TODO: return a child orders mock that wraps messages and stores them in its parent
    }

    fn perform_cmd<MsU: 'static>(self: &mut Self, cmd: impl Future<Output = MsU> + 'static) {
        let t_type = std::any::TypeId::of::<MsU>();
        let handler: Box<dyn Fn(MsU) -> Option<Ms>> = if t_type == std::any::TypeId::of::<Ms>() {
            Box::new(move |value| {
                (&mut Some(identity(value)) as &mut dyn std::any::Any)
                    .downcast_mut::<Option<Ms>>()
                    .and_then(Option::take)
            })
        } else if t_type == std::any::TypeId::of::<Option<Ms>>() {
            Box::new(move |value| {
                (&mut identity(value) as &mut dyn std::any::Any)
                    .downcast_mut::<Option<Ms>>()
                    .and_then(Option::take)
            })
        } else if t_type == std::any::TypeId::of::<()>() {
            Box::new(move |value| {
                identity(value);
                None
            }) as Box<dyn Fn(MsU) -> Option<Ms>>
        } else {
            panic!("TODO");
        };

        match handler(block_on(cmd)) {
            Some(msg) => self.messages.push(msg),
            None => { /* do noting */ }
        }
    }

    fn subscribe<MsU: 'static, SubMs: 'static + Clone>(
        &mut self,
        _handler: impl FnOnce(SubMs) -> MsU + Clone + 'static,
    ) {
        // TODO: implement when needed
    }

    fn notify(&mut self, _message: impl Any + Clone) {
        todo!()
    }
}

// methods used for testing purposes
impl<Ms, Model, Node> OrdersMock<Ms, Model, Node> {
    pub fn messages(self: &Self) -> &Vec<Ms> {
        &self.messages
    }
}

pub struct OrdersStub<Ms, Model, Node> {
    pub update: Rc<RefCell<dyn FnMut(Ms)>>,
    _messages: Vec<Ms>,
    _model: Vec<Model>,
    _node: Vec<Node>,
}

// methods called in MyOrders
impl<Ms: 'static, Model, Node> OrdersStub<Ms, Model, Node> {
    pub fn new(update: Rc<RefCell<dyn FnMut(Ms)>>) -> Self {
        OrdersStub {
            update: update,
            _messages: vec![],
            _model: vec![],
            _node: vec![],
        }
    }

    fn proxy<ChildMs: 'static>(
        &mut self,
        f: impl FnOnce(ChildMs) -> Ms + 'static + Clone,
    ) -> OrdersStubProxy<ChildMs, Model, Node> {
        let f_clone = f.clone();

        let update_clone = self.update.clone();
        OrdersStubProxy::<ChildMs, Model, Node>::new(Rc::new(RefCell::new(move |msg| {
            (update_clone.borrow_mut())((f_clone.clone())(msg))
        })))
    }

    fn perform_cmd<MsU: 'static>(self: &mut Self, cmd: impl Future<Output = MsU> + 'static) {
        let t_type = std::any::TypeId::of::<MsU>();
        let handler: Box<dyn Fn(MsU) -> Option<Ms>> = if t_type == std::any::TypeId::of::<Ms>() {
            Box::new(move |value| {
                (&mut Some(identity(value)) as &mut dyn std::any::Any)
                    .downcast_mut::<Option<Ms>>()
                    .and_then(Option::take)
            })
        } else if t_type == std::any::TypeId::of::<Option<Ms>>() {
            Box::new(move |value| {
                (&mut identity(value) as &mut dyn std::any::Any)
                    .downcast_mut::<Option<Ms>>()
                    .and_then(Option::take)
            })
        } else if t_type == std::any::TypeId::of::<()>() {
            Box::new(move |value| {
                identity(value);
                None
            }) as Box<dyn Fn(MsU) -> Option<Ms>>
        } else {
            panic!("TODO");
        };

        match handler(block_on(cmd)) {
            Some(msg) => (self.update.borrow_mut())(msg),
            None => { /* do noting */ }
        }
    }

    fn subscribe<MsU: 'static, SubMs: 'static + Clone>(
        &mut self,
        _handler: impl FnOnce(SubMs) -> MsU + Clone + 'static,
    ) {
        // TODO: implement when needed
    }

    fn notify(&mut self, notification: impl Any + Clone) {
        let t_type = std::any::TypeId::of::<Ms>();
        let handler: Box<dyn Fn(Tmp) -> Option<Ms>> = if t_type == std::any::TypeId::of::<Ms>() {
            Box::new(move |value| {
                (&mut Some(identity(value)) as &mut dyn std::any::Any)
                    .downcast_mut::<Option<Ms>>()
                    .and_then(Option::take)
            })
        } else if t_type == std::any::TypeId::of::<Option<Ms>>() {
            Box::new(move |value| {
                (&mut identity(value) as &mut dyn std::any::Any)
                    .downcast_mut::<Option<Ms>>()
                    .and_then(Option::take)
            })
        } else if t_type == std::any::TypeId::of::<()>() {
            Box::new(move |value| {
                identity(value);
                None
            }) as Box<dyn Fn(Ms) -> Option<Ms>>
        } else {
            panic!("TODO");
        };

        match handler(notification) {
            Some(msg) => (self.update.borrow_mut())(msg),
            None => { /* do noting */ }
        }
    }
}

pub struct OrdersStubProxy<Ms, Model, Node> {
    pub update: Rc<RefCell<dyn FnMut(Ms)>>,
    _model: Vec<Model>,
    _node: Vec<Node>,
}

// methods called in MyOrders
impl<Ms: 'static, Model, Node> OrdersStubProxy<Ms, Model, Node> {
    pub fn new(update: Rc<RefCell<dyn FnMut(Ms)>>) -> Self {
        OrdersStubProxy {
            update: update,
            _model: vec![],
            _node: vec![],
        }
    }

    fn proxy<ChildMs: 'static>(
        &mut self,
        f: impl FnOnce(ChildMs) -> Ms + 'static + Clone,
    ) -> OrdersStubProxy<ChildMs, Model, Node> {
        let f_clone = f.clone();

        let update_clone = self.update.clone();
        OrdersStubProxy::<ChildMs, Model, Node>::new(Rc::new(RefCell::new(move |msg| {
            (update_clone.borrow_mut())((f_clone.clone())(msg));
        })))
    }

    fn perform_cmd<MsU: 'static>(self: &mut Self, cmd: impl Future<Output = MsU> + 'static) {
        let t_type = std::any::TypeId::of::<MsU>();
        let handler: Box<dyn Fn(MsU) -> Option<Ms>> = if t_type == std::any::TypeId::of::<Ms>() {
            Box::new(move |value| {
                (&mut Some(identity(value)) as &mut dyn std::any::Any)
                    .downcast_mut::<Option<Ms>>()
                    .and_then(Option::take)
            })
        } else if t_type == std::any::TypeId::of::<Option<Ms>>() {
            Box::new(move |value| {
                (&mut identity(value) as &mut dyn std::any::Any)
                    .downcast_mut::<Option<Ms>>()
                    .and_then(Option::take)
            })
        } else if t_type == std::any::TypeId::of::<()>() {
            Box::new(move |value| {
                identity(value);
                None
            }) as Box<dyn Fn(MsU) -> Option<Ms>>
        } else {
            panic!("TODO");
        };

        match handler(block_on(cmd)) {
            Some(msg) => (self.update.borrow_mut())(msg),
            None => { /* do noting */ }
        }
    }

    fn subscribe<MsU: 'static, SubMs: 'static + Clone>(
        &mut self,
        _handler: impl FnOnce(SubMs) -> MsU + Clone + 'static,
    ) {
        // TODO: implement when needed
    }

    fn notify(&mut self, _message: impl Any + Clone) {
        todo!()
    }
}

pub enum OrdersImplementation<'a, Ms: 'static, AppMs: 'static> {
    Container(OrdersContainer<AppMs, Model, Node<AppMs>>),
    Proxy(OrdersProxy<'a, Ms, AppMs, Model, Node<AppMs>>),
    Mock(OrdersMock<Ms, Model, Node<AppMs>>),
    Stub(OrdersStub<Ms, Model, Node<AppMs>>),
    StubProxy(OrdersStubProxy<Ms, Model, Node<AppMs>>),
}

pub trait IMyOrders<Ms> {
    type AppMs: 'static;

    fn proxy<ChildMs: 'static>(
        &mut self,
        f: impl FnOnce(ChildMs) -> Ms + 'static + Clone,
    ) -> MyOrders<ChildMs, Self::AppMs>;

    fn perform_cmd<MsU: 'static>(&mut self, cmd: impl Future<Output = MsU> + 'static) -> &mut Self;

    fn subscribe<MsU: 'static, SubMs: 'static + Clone>(
        &mut self,
        handler: impl FnOnce(SubMs) -> MsU + Clone + 'static,
    ) -> &mut Self;

    fn notify(&mut self, message: impl Any + Clone) -> &mut Self;
}

pub struct MyOrders<'a, Ms: 'static, AppMs: 'static> {
    pub implementation: OrdersImplementation<'a, Ms, AppMs>,
}

impl<'a, Ms: 'static, AppMs: 'static> MyOrders<'a, Ms, AppMs> {
    pub fn new(implementation: OrdersImplementation<Ms, AppMs>) -> MyOrders<Ms, AppMs> {
        MyOrders {
            implementation: implementation,
        }
    }

    pub fn mock(self: &'a Self) -> Option<&'a OrdersMock<Ms, Model, Node<AppMs>>> {
        match self.implementation {
            OrdersImplementation::Container(_) => None,
            OrdersImplementation::Proxy(_) => None,
            OrdersImplementation::Mock(ref mock) => Some(mock),
            OrdersImplementation::Stub(_) => None,
            OrdersImplementation::StubProxy(_) => None,
        }
    }

    pub fn stub(self: &'a mut Self) -> Option<&'a mut OrdersStub<Ms, Model, Node<AppMs>>> {
        match self.implementation {
            OrdersImplementation::Container(_) => None,
            OrdersImplementation::Proxy(_) => None,
            OrdersImplementation::Mock(_) => None,
            OrdersImplementation::Stub(ref mut stub) => Some(stub),
            OrdersImplementation::StubProxy(_) => None,
        }
    }
}

impl<'a, Ms, AppMs> IMyOrders<Ms> for MyOrders<'a, Ms, AppMs> {
    type AppMs = AppMs;

    fn proxy<ChildMs: 'static>(
        &mut self,
        f: impl FnOnce(ChildMs) -> Ms + 'static + Clone,
    ) -> MyOrders<ChildMs, AppMs> {
        match self.implementation {
            OrdersImplementation::Container(ref mut _orders) => {
                todo!()
                // let proxy = orders.proxy(f);
                // MyOrders::<ChildMs, AppMs>::new(OrdersImplementation::Proxy(proxy))
            }
            OrdersImplementation::Proxy(ref mut orders) => {
                let proxy = orders.proxy(f);
                MyOrders::<ChildMs, AppMs>::new(OrdersImplementation::Proxy(proxy))
            }
            OrdersImplementation::Mock(ref mut orders) => {
                let proxy = orders.proxy(f);
                MyOrders::<ChildMs, AppMs>::new(OrdersImplementation::Mock(proxy))
            }
            OrdersImplementation::Stub(ref mut orders) => {
                let proxy = orders.proxy(f);
                MyOrders::<ChildMs, AppMs>::new(OrdersImplementation::StubProxy(proxy))
            }
            OrdersImplementation::StubProxy(ref mut orders) => {
                let proxy = orders.proxy(f);
                MyOrders::<ChildMs, AppMs>::new(OrdersImplementation::StubProxy(proxy))
            }
        }
    }

    fn perform_cmd<MsU: 'static>(&mut self, cmd: impl Future<Output = MsU> + 'static) -> &mut Self {
        match self.implementation {
            OrdersImplementation::Container(ref mut orders) => {
                orders.perform_cmd(cmd);
            }
            OrdersImplementation::Proxy(ref mut orders) => {
                orders.perform_cmd(cmd);
            }
            OrdersImplementation::Mock(ref mut orders) => {
                orders.perform_cmd(cmd);
            }
            OrdersImplementation::Stub(ref mut orders) => {
                orders.perform_cmd(cmd);
            }
            OrdersImplementation::StubProxy(ref mut orders) => {
                orders.perform_cmd(cmd);
            }
        }
        self
    }

    fn subscribe<MsU: 'static, SubMs: 'static + Clone>(
        &mut self,
        handler: impl FnOnce(SubMs) -> MsU + Clone + 'static,
    ) -> &mut Self {
        match self.implementation {
            OrdersImplementation::Container(ref mut orders) => {
                orders.subscribe(handler);
            }
            OrdersImplementation::Proxy(ref mut orders) => {
                orders.subscribe(handler);
            }
            OrdersImplementation::Mock(ref mut orders) => {
                orders.subscribe(handler);
            }
            OrdersImplementation::Stub(ref mut orders) => {
                orders.subscribe(handler);
            }
            OrdersImplementation::StubProxy(ref mut orders) => {
                orders.subscribe(handler);
            }
        }
        self
    }

    fn notify(&mut self, message: impl Any + Clone) -> &mut Self {
        match self.implementation {
            OrdersImplementation::Container(ref mut orders) => {
                orders.notify(message);
            }
            OrdersImplementation::Proxy(ref mut orders) => {
                orders.notify(message);
            }
            OrdersImplementation::Mock(ref mut orders) => {
                orders.notify(message);
            }
            OrdersImplementation::Stub(ref mut orders) => {
                orders.notify(message);
            }
            OrdersImplementation::StubProxy(ref mut orders) => {
                orders.notify(message);
            }
        }
        self
    }
}

pub fn perform_cmd<Ms: 'static>(
    orders: &mut impl IMyOrders<Ms>,
    cmd: impl Future<Output = Ms> + 'static,
) {
    orders.perform_cmd(cmd);
}
