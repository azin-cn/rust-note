use std::{
    sync::{
        mpsc::{self, Sender, SyncSender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

fn main() {
    /*
     *
     * ## 线程同步：锁、Condvar 和信号量
     * **同步性指的是通过协调不同线程或任务的执行顺序来安全地共享数据和资源**。
     * 同步性是并发编程中的一个重要概念，涉及到如何保证多个执行单元（如线程或异步任务）之间正确且安全地访问共享资源，而不会导致数据竞争、死锁等问题。
     *
     * 借助 Rust 强大的类型系统和所有权模型，在编写多线程代码，需要使用同步性时，可以通过互斥锁(Mutex)、读写锁(RwLock)、原子类型(Atomic Types)和通道(Channel)等机制，编写高效且安全的并发程序。
     *
     * 在多线程间有多种方式可以共享和传递数据，最常用有两种：
     * - 消息传递
     * - 锁和 Arc 联合使用
     *
     * 对于消息传递，在编程界有一个大名鼎鼎的 **Actor 线程模型**为其背书，典型的有 Erlang 语言、Go 语言。
     *
     * ### 如何选择数据共享方式
     *
     * **共享内存**是同步的灵魂，消息传递的底层也是通过共享内存来实现的：
     * - 消息传递类似一个单所有权的系统，一个值同时只能有一个所有者，如果另一个线程需要该值的所有权，需要将所有权通过消息传递进行转移，可以做到传递引用和传递值
     * - 而共享内存类似于一个多所有权的系统，多个线程可以同时访问同一个值，用锁来控制哪个线程可以在当前时刻访问，可以做到直接访问同一个内存
     *
     * 对比两种方式：
     * - 锁和 Arc 联合使用的共享内存相对消息传递能节省多次内存拷贝的成本
     * - 共享内存的实现简洁的多
     * - 共享内存的锁竞争更多
     *
     * 消息传递适用的场景很多，几个主要的使用场景:
     * - 需要可靠和简单的(简单不等于简洁)实现多线程编程
     * - 需要模拟现实世界，例如用消息去通知某个目标执行相应的操作时（事件触发）
     * - 需要一个任务处理流水线(管道)时，等等
     *
     * 而使用共享内存(并发原语)的场景往往就比较简单粗暴：需要**简洁的实现以及更高的性能**。
     *
     * ### 互斥锁 Mutex
     * > Mutex 在之前章节已经用过，这里的介绍有点繁琐，精简了一下学习过程
     * > https://course.rs/advance/concurrency-with-threads/sync1.html#互斥锁-mutex
     *
     * 在之前章节介绍中提到过，Mutex 是一个并发原语，它能让多个线程并发的访问同一个值变成了排队访问，同一时间只允许一个线程 A 访问该值，其它线程需要等待 A 访问完成后才能访问。
     *
     * 使用 Mutex 时，需要先锁定它访问数据，然后再解锁让其他线程可以访问该数据。
     * 锁定和解锁的过程通常是自动的，通过 Rust 的作用域管理来实现。当 Mutex 的锁超出作用域时，它会自动释放。
     *
     * 不同于线程局部变量的每一个线程都有单独的数据拷贝，**Mutex 用于多线程访问同一个实例**，因为用于多线程，所以常常和 **Arc** 搭配使用：
     * ```rust
     * // Mutex 需要手动上锁，超过作用于后自动解锁
     * let count = 5;
     * let mutex = Arc::new(Mutex::new(String::from("Hello")));
     * let mut handles: Vec<JoinHandle<()>> = Vec::with_capacity(count);
     * for i in 0..count {
     *     let _mutex = Arc::clone(&mutex);
     *     handles.push(thread::spawn(move || {
     *         // lock 方法申请一个锁, 该方法会阻塞当前线程，直到获取到锁，因此当多个线程同时访问该数据时，只有一个线程能获取到锁
     *         // 其它线程只能阻塞着等待，这样就保证了数据能被安全的修改！
     *         let mut s = _mutex.lock().unwrap();
     *         s.push_str(i.to_string().as_str())
     *         // 锁自动被drop
     *     }))
     * }
     *
     * for h in handles {
     *     h.join().unwrap();
     * }
     * println!("{}", mutex.lock().unwrap());
     * ```
     *
     * lock 方法申请一个锁, 该方法会阻塞当前线程，直到获取到锁，因此当多个线程同时访问该数据时，只有一个线程能获取到锁，其它线程只能阻塞着等待，这样就保证了数据能被安全的修改！
     * lock 方法也有可能报错，例如当前正在持有锁的线程 panic 了，在这种情况下，其它线程不可能再获得锁，因此 lock 方法会返回一个错误。
     *
     * `Mutex<T>` 是一个智能指针（结构体），它的方法 lock 返回另外一个智能指针（结构体） `MutexGuard<T>`，`MutexGuard<T>` 实现两个非常便捷的特征，Deref 和 Drop：
     * - Deref 特征，会被自动解引用后获得一个引用类型，该引用指向 Mutex 内部的数据
     * - Drop 特征，在超出作用域后，自动释放锁，以便其它线程能继续获取锁
     *
     * 使用 Mutex 时注意避免形成死锁：
     * ```rust
     * // 使用 mutex 注意避免形成死锁
     * let mutex = Mutex::new(3);
     * let num = mutex.lock().unwrap(); // 上锁
     * {
     *     // 由于在上一行给mutex上锁了，因此这里会一直阻塞，等待获取值的所有权，但是因为 num 没有释放，所以线程一直在阻塞，这就是死锁
     *     let _num = mutex.lock().unwrap();
     * }
     * println!("{}", num);
     * ```
     * #### 小心使用 Mutex
     * - 在使用数据前必须先获取锁
     * - 在数据使用完成后，必须及时的释放锁，例如增加作用域
     * 
     * 例如：当一个操作试图锁住两个资源，然后两个线程各自获取其中一个锁，并试图获取另一个锁时，就会造成死锁（deadlock）。
     *
     * #### 内部可变性
     * 内部可变性是指当前**变量/值的空间存储的内容发生改变**的行为。
     * 
     * Cell 与 RefCell 的可变借用行为并不完全一致，这是由于存储的数据类型不一样决定的：
     * Cell 和 RefCell 都是智能指针，用一个栈上的新空间存储被管理的值，不同的是 Cell 存储 Copy 类型的值，而 RefCell 存储的是非 Copy 类型的栈上指针信息（通过栈上指针信息管理堆上实际数据）。
     *
     * `Rc<T>/RefCell<T>` 用于单线程内部可变性， `Arc<T>/Mutex<T>` 用于多线程内部可变性。
     * 
     * 
     * 
     * 
     *
     *
     *
     *
     */
    let count = 5;
    let mutex = Arc::new(Mutex::new(String::from("Hello")));
    let mut handles: Vec<JoinHandle<()>> = Vec::with_capacity(count);
    for i in 0..count {
        let _mutex = Arc::clone(&mutex);
        handles.push(thread::spawn(move || {
            // lock 方法申请一个锁, 该方法会阻塞当前线程，直到获取到锁，因此当多个线程同时访问该数据时，只有一个线程能获取到锁
            // 其它线程只能阻塞着等待，这样就保证了数据能被安全的修改！
            let mut s: std::sync::MutexGuard<String> = _mutex.lock().unwrap();
            s.push_str(i.to_string().as_str());
            // 锁自动被drop
        }))
    }

    for h in handles {
        h.join().unwrap();
    }
    println!("{}", mutex.lock().unwrap());

    // 使用 mutex 注意避免形成死锁
    let mutex = Mutex::new(3);
    let num = mutex.lock().unwrap(); // 上锁
    {
        // 由于在上一行给mutex上锁了，因此这里会一直阻塞，等待获取值的所有权，但是因为 num 没有释放，所以线程一直在阻塞，这就是死锁
        let _num = mutex.lock().unwrap();
    }
    println!("{}", num);
}
