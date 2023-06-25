use seed::prelude::Orders;
use std::future::Future;

pub fn perform_cmd<Msg: 'static>(
    orders: &mut impl Orders<Msg>,
    cmd: impl Future<Output = Msg> + 'static,
) {
    orders.perform_cmd(cmd);
}
