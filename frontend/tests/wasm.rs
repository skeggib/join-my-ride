use frontend::app::{self, Msg};
use frontend::orders::{MyOrders, OrdersMock, OrdersImplementation};
use seed::Url;

#[test]
fn test() {
    let url = Url::new();
    let mut orders = MyOrders::new(OrdersImplementation::<Msg, Msg>::Mock(OrdersMock::new()));
    let app_ = app::testable_init(url, &mut orders);
    assert!(orders.mock().unwrap().messages().is_empty())
}
