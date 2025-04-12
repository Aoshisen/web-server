use std::net::TcpListener;
fn main() {
    let port = "127.0.0.1:7878";
    let listener = TcpListener::bind(port).unwrap();
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        println!("建立连接了");
    }
}
