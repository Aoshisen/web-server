use std::{
    sync::{Arc, Mutex, mpsc},
    thread,
};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

//如何分配空间存储线程
impl ThreadPool {
    // 1. ThreadPool 会创建一个通道并充当发送端。
    //2. 每个 Worker 将会充当通道的接收端。
    // 3. 新建一个 Job 结构体来存放用于向通道中发送的闭包。
    // 4. execute 方法会在通道发送端发出期望执行的任务。
    // 5.在线程中，Worker 会遍历通道的接收端并执行任何接收到的任务。
    pub fn new(size: usize) -> ThreadPool {
        //使用assert 宏在size 为0 的时候panic;
        assert!(size > 0);
        //创建长度为size 的 vec
        let mut workers = Vec::with_capacity(size);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        for id in 0..size {
            //在创建work的时候传递闭包,然后后面才好在execute 里面处理闭包逻辑
            // 将接受端发送到每个worker 内部;
            //需要使用Mutex 和Arc 来进行多线程传递receiver
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        ThreadPool { workers, sender }
    }
    pub fn execute<F>(&self, f: F)
    //FnOnce 处理线程 只会执行闭包一次:
    // Send 将闭包从主线程发送到ThreadPool 管理的线程
    // 'static 生命周期,因为不知道线程执行多久,所以使用了static 生命周期;
    where
        F: FnOnce() + Send + 'static,
    {
        //通过Box<T> 包装闭包发送给worker
        let job = Box::new(f);
        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

//优雅的停机和清理
impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");

        for _ in &mut self.workers {
            //发送停机信息;
            self.sender.send(Message::Terminate).unwrap();
        }

        println!("Shutting down all workers.");

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                //等待停机完成
                thread.join().unwrap();
            }
        }
    }
}

// 经过观察我们需要使用execute 来处理闭包里面的逻辑,就像thread::spawn 差不多
// pub fn spawn<F, T>(f: F) -> JoinHandle<T>
//     where
//         F: FnOnce() -> T + Send + 'static,
//         T: Send + 'static
//然后仿造 spawn 的类型定义来完成我们的execute 函数的f 的类型定义;

//spawn 返回 JoinHandle<T>，其中 T 是闭包返回的类型。尝试使用 JoinHandle 来看看会发生什么。在我们的情况中，传递给线程池的闭包会处理连接并不返回任何值，所以 T 将会是单元类型 ()。

// Worker 负责从ThreadPool 里面把代码传递给线程
// 标准库中的spawn 希望获取一个线程创建就可以立即执行的代码,但是我们希望创建线程并等待稍后传递的代码;
// 使用Worker 来代替之前的Thread Worker 内部维护 thread:JoinHandle<()>, 然后需要在Worker 上实现方法,他会获取需要运行的代码闭包,然后发送代码到当前worker 运行的线程执行; 我们还会创建一个worker 的唯一id 来在日志或者控制台区分每个worker
pub struct Worker {
    thread: Option<thread::JoinHandle<()>>,
    id: usize,
}
impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        // 传递给 thread::spawn 的闭包仍然还只是 引用 了通道的接收端。相反我们需要闭包一直循环，向通道的接收端请求任务，并在得到任务时执行他们。如示例 20-20 对 Worker::new 做出修改：
        let thread = thread::spawn(move || {
            loop {
                //调用了lock() 来获取receiver 并通过 recv 阻塞当前线程直到有新的可用任务;
                let message = receiver.lock().unwrap().recv().unwrap();
                match message {
                    Message::NewJob(job) => {
                        // println!("new job");
                        job()
                    }
                    Message::Terminate => {
                        println!("terminate");
                        break;
                    }
                }
            }
        });
        Worker {
            id,
            thread: Some(thread),
        }
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

//向线程发送信号使其停止任务
enum Message {
    NewJob(Job),
    Terminate,
}
