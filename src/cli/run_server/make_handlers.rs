use crate::common::{AuthUsersCollection, BillConfig, BillingHandler, LoggerSender};
use crate::handlers::{
    CloseHandler, ConnectHandler, ConvertPointHandler, CostLogHandler, EnterGameHandler,
    KeepHandler, KickHandler, LoginHandler, LogoutHandler, PingHandler, QueryPointHandler,
    RegisterHandler,
};
use mysql_async::Pool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::sync::RwLock;

/// 添加handler的宏
macro_rules! add_handler {
    ($handler_map:ident,$($handler:expr ),*) => {
        $(
            let tmp_handler = Box::new($handler);
            //println!("op_type={:#04X}", $handler.get_type());
            $handler_map.insert($handler.get_type(), tmp_handler);
        )*
    };
}

pub(super) fn make_handlers(
    server_config: &BillConfig,
    tx: &Sender<u8>,
    db_pool: &Pool,
    stopped_flag: &Arc<RwLock<bool>>,
    logger_sender: &LoggerSender,
) -> HashMap<u8, Box<dyn BillingHandler>> {
    let mut handlers: HashMap<u8, Box<dyn BillingHandler>> = HashMap::new();
    let auto_reg = server_config.auto_reg();
    let convert_number = server_config.transfer_number();
    //在线的用户 Map: user_name => AuthUser
    let auth_users_collection = &AuthUsersCollection::new(RwLock::new(HashMap::new()));
    //向handlers Map中添加handler
    add_handler!(
        handlers,
        CloseHandler::new(tx.clone(), stopped_flag.clone(), logger_sender.clone()),
        ConnectHandler,
        PingHandler,
        LoginHandler::new(
            db_pool.clone(),
            auto_reg,
            auth_users_collection.clone(),
            logger_sender.clone()
        ),
        EnterGameHandler::new(auth_users_collection.clone(), logger_sender.clone()),
        LogoutHandler::new(auth_users_collection.clone(), logger_sender.clone()),
        KeepHandler::new(auth_users_collection.clone(), logger_sender.clone()),
        KickHandler,
        CostLogHandler,
        ConvertPointHandler::new(
            db_pool.clone(),
            convert_number,
            auth_users_collection.clone(),
            logger_sender.clone()
        ),
        QueryPointHandler::new(
            db_pool.clone(),
            auth_users_collection.clone(),
            logger_sender.clone()
        ),
        RegisterHandler::new(db_pool.clone(), logger_sender.clone())
    );
    handlers
}
