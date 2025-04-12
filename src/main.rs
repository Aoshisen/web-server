use std::thread;
use std::time::Duration;
use std::{
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};
fn main() {
    // 注意这个addr : 前面的是本机地址,而后面是需要绑定的端口号;
    // 注意 这里非管理员 只能创建大于1024 的端口号,且如果两个程序绑定了同一个端口,那么就会报错
    let addr = "127.0.0.1:7878";
    // 建立localhost: 也是一样的
    // let addr = "localhost:7878";
    // 这里的bind 函数其实就是 创建一个TcpListener 的实例,但是在网络领域连接到一个端口叫作 绑定到一个端口(bind to a port)
    //由于这里的绑定可能会绑定失败,所以这个TcpListener::bind(addr) 的返回值是一个Result<T,E> 我们通过unwrap()在出现这种情况的时候直接终止程序;
    let listener = TcpListener::bind(addr).unwrap();

    //监听到port 上的 tcp 连接的输入流,当获取到输入流,打印已经建立连接了
    //TcpListener.incoming() 返回一个迭代器 他提供了一系列的流(tcpStream) stream 表示了客户端和服务端打开的连接;
    //连接(connection) 代表了客户端连接服务端 服务端响应客户端 以及服务端关闭请求的全部 请求/响应过程
    //TcpStream 允许读取其来查看客户端发送了什么,然后处理这个流的信息并返回对应的信息
    //这个for 循环会依次处理每个连接并产生一系列的tcpStream 并供我们处理;
    for stream in listener.incoming() {
        //实际上我们并没有尝试遍历连接,而是遍历了连接尝试(connection attempts),因为连接本身是可能失败的,
        // 连接可能会因为很多不同的原因失败,但是大多是系统的原因,比如系统限制同时打开的连接数量;

        //新连接尝试返回一些错误,直到一些打开的连接关闭为止
        // 1. 为什么新连接会尝试返回错误
        // listener.incoming() 本身是一个阻塞的迭代器,当每次调用next() 的时候会等待一个新连接,当有新连接进来的时候stream.unwrap() 会尝试接受他,
        // 错误的原因大多是操作系统限制,比如操作系统维护了一个未接受的连接队列(backlog queue),当新连接到达时,会先进入这个队列;
        // 但是如果队列满了,那么操作系统就会阻止新的stream 进入该队列,新连接就会被拒绝
        // 在现在的代码中没有显示的处理流信息, 且没有实际处理(读写数据和关闭连接);
        // 连接会一直保持打开的状态,直到流被drop ,导致主线程的连接队列被逐渐占满,报错;

        // 2. 为什么这个错误会直到一些打开的连接关闭为止
        // 当所有未被处理的连接最终被关闭,操作系统中的连接队列才会有空位,此时连接才会被继续接受
        // 如果连接队列被占满,stream.unwrap() 会返回ConnectionAborted 或者 TooMany open files 的错误;

        let stream = stream.unwrap();
        // 现在建立的连接中可能会重复打印下面的字段,这是因为浏览器会请求一些其他的信息 比如出现在浏览器 tab 标签中的 favicon.ico。
        // 还有就是浏览器的重试机制,如果连接成功,但是请求到的东西没有,就会重试

        //

        // println!("建立连接了");
        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    // 1. 为什么这个stream 要是可变的,
    // 这是因为 TcpStream 实例在内部记录了所返回的数据。它可能读取了多于我们请求的数据并保存它们以备下一次请求数据。因此它需要是 mut 的因为其内部状态可能会改变；
    // 2. 这里的buffer 的作用是什么;
    // 申明一块具有实际大小的空间用来存储,然后将这个buffer 传递给stream.read() 那么我们就可以得到stream 上的信息了;
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    // 将buffer 上的数据通过String::from_utf8_lossy 方法打印出来,函数获取一个&u8 并返回一个string,lossy 当遇到无效的utf8 编码时,返回 � (U+FFFD REPLACEMENT CHARACTER)
    // --------------------------------- 请求----------------------------------------
    // 简析以下返回的值分别代表什么
    // 	Method Request-URI HTTP-Version CRLF
    //  headers CRLF
    //  message-body

    //  GET / HTTP/1.1
    // Host: 127.0.0.1:7878
    // User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:137.0) Gecko/20100101 Firefox/137.0
    // Accept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8

    // Method =>GET Request-URI => / (URL[统一资源标识符号] 和URI[统一资源定位符符] 有细微区别但是在这可以理解成相同的 ) HTTP-Version=>HTTP/1.1 行尾序列(CRLF 回车和换行[carriage return line feed] 可以写成\r\n  当时看不到 因为这是换行的意思 )
    // headers => Host: 127.0.0.1:7878;
    // 然后下面就是请求的 message-body 信息(get信息没有body);

    // ----------------------------------- 响应 -----------------------------------------
    // 对应的响应的格式
    // HTTP-Version Status-Code Reason-Phrase CRLF  // 状态行（例如 `HTTP/1.1 200 OK`）
    // headers CRLF                                 // 头部（例如 `Content-Length: 123`）
    // CRLF                                        // 空行（分隔头部和正文）
    // message-body                                // 响应正文（例如 HTML 内容）
    //我们来编写响应

    //header 为空,但是行尾的序列还是要写(没有header 没有body)
    // let response = "HTTP/1.1 200 OK\r\n\r\n";
    // 返回真正的html;
    //判断是否是 根路径的get请求
    // 字节字符串语法将其转换为字节字符串
    let get = b"GET / HTTP/1.1\r\n";
    // 模拟慢请求
	// 现在单线程如果请求了慢请求那么就会阻塞后续请求其他请求,效率很低;
    let sleep = b"GET /sleep HTTP/1.1\r\n";
    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK", "index.html")
    } else if buffer.starts_with(sleep) {
		//等5s 返回;
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK", "index.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };
    let contents = fs::read_to_string(filename).unwrap();
    let response = format!(
        // 注意这里的行尾序列 Content-Length 换行后 还要再换一行 来接受message-body
        // 这里的Content-Length 的作用是这样客户端知道需要读取多少字节的正文。 如果没有他,客户端可能会一直等待更多数据
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );

    //返回成功的response,stream.write 接受一个&[u8] 并将这些字节返回给客户端
    // stream.write() 不会保证数据被立即发送,只会把写好的数据放在 操作系统的发送缓冲区;
    stream.write(response.as_bytes()).unwrap();
    // stream.flush() 会立即清空发送缓冲区;发送所有的缓冲区的数据
    // 如果不调用这个函数,数据可能会存在缓冲区一段时间,直到缓冲区满了,或者关闭了连接,导致客户端延迟收到信息;
    stream.flush().unwrap();

    // println!("Request: {}", String::from_utf8_lossy(&buffer[..]));
}
