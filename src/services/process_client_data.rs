use crate::common::{
    BillDebugType, BillingData, BillingHandler, LoggerSender, ParsePackError, ResponseError,
};
use crate::log_message;
use std::collections::HashMap;

use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

/// 当读取到TCP数据后的处理
pub async fn process_client_data<S: std::hash::BuildHasher>(
    socket: &mut TcpStream,
    client_data: &mut Vec<u8>,
    handlers: &mut HashMap<u8, Box<dyn BillingHandler>, S>,
    logger_sender: &mut LoggerSender,
    debug_type: BillDebugType,
) -> Result<(), ResponseError> {
    //循环读取
    loop {
        let (billing_data, full_pack_size) =
            match BillingData::read_from_client(client_data.as_slice()) {
                //成功读取到一个BillingData,将进行后续处理
                Ok(value) => value,
                Err(err) => match err {
                    // 数据长度不足,跳出loop循环
                    ParsePackError::BillingDataNotFull => break,
                    //数据结构错误
                    ParsePackError::BillingDataError => return Err(ResponseError::PackError),
                },
            };
        if client_data.len() == full_pack_size {
            //已读完,清理client_data
            client_data.clear();
        } else {
            let end_pos = client_data.len();
            //copy
            client_data.copy_within(full_pack_size..end_pos, 0);
            client_data.resize(end_pos - full_pack_size, 0);
        }
        //调试模式: 显示请求的数据包
        if debug_type != BillDebugType::NoDebug {
            //full或者不为Ping类信息时,打印数据包
            if debug_type == BillDebugType::Full || billing_data.op_type != 0xA1 {
                log_message!(logger_sender, Debug, "request = {:?}", &billing_data);
            }
        }
        //查找对应类型的handler
        if let Some(bill_handler) = handlers.get_mut(&billing_data.op_type) {
            // 使用handler从request得到response
            let response = bill_handler.get_response(&billing_data).await?;
            //dbg!(&response);
            // 打包为字节序列
            let response_bytes = response.pack_data();
            //dbg!(&response_bytes);
            // 发送到Client
            socket.write_all(&response_bytes).await?;
        } else {
            // 记录不能处理的类型
            log_message!(
                logger_sender,
                Error,
                "unknown billing data (op_type={:#04X}) :{:?}",
                billing_data.op_type,
                &billing_data
            );
        }
    }
    Ok(())
}
