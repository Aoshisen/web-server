use std::thread;

// 申明我们的线程的类型;
type Thread = thread::JoinHandle<()>;
pub struct ThreadPool {
    workers: Vec<Worker>,
}

//如何分配空间存储线程
impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        //使用assert 宏在size 为0 的时候panic;
        assert!(size > 0);
        //创建长度为size 的 vec
        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            //
            workers.push(Worker::new(id))
        }
        ThreadPool { workers }
    }
    pub fn execute<F>(&self, f: F)
    //FnOnce 处理线程 只会执行闭包一次:
    // Send 将闭包从主线程发送到ThreadPool 管理的线程
    // 'static 生命周期,因为不知道线程执行多久,所以使用了static 生命周期;
    where
        F: FnOnce() + Send + 'static,
    {
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
    thread: Thread,
    id: usize,
}
impl Worker {
    fn new(id: usize) -> Worker {
        let thread = thread::spawn(|| {});
        Worker { id, thread }
    }
}
