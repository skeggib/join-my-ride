use std::rc::Rc;

use common::{Event, Id};
use common::api::BackendApi;
use frontend::app::{self, Msg};
use frontend::orders::{MyOrders, OrdersMock, OrdersImplementation};
use seed::Url;
use async_trait::async_trait;

struct MockBackend {}

#[async_trait(?Send)]
impl BackendApi for MockBackend {
    async fn get_events(self: &Self) -> Result<Vec<Event>, String>{
        Err("not implemented".into())
    }
    async fn get_event(self: &Self, id: Id) -> Result<Event, String>{
        Err("not implemented".into())
    }
    async fn publish_event(self: &Self, name: String) -> Result<(), String>{
        Err("not implemented".into())
    }
    async fn join_event(self: &Self, id: Id) -> Result<(), String>{
        Err("not implemented".into())
    }
}

#[test]
fn test() {
    let url = Url::new();
    let mut orders = MyOrders::new(OrdersImplementation::<Msg, Msg>::Mock(OrdersMock::new()));
    let app_ = app::testable_init(url, &mut orders, Rc::new(MockBackend{}));
    assert!(orders.mock().unwrap().messages().is_empty())
}
