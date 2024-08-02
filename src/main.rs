use bytes::BytesMut;
use mini_redis::{Frame, Result};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::{self, TcpListener},
};

#[tokio::main]
async fn main() -> Result<()> {
    /*
     *
     * ## 实战：mini-redis - client - IO & Frame
     *
     * 在 mini-redis 中，以帧 frame 作为命令和数据的结合作为一次指令操作，要构建帧 frame 需要先认识 tokio 的 io 操作。
     * Tokio 中的 I/O 操作和 std 在使用方式上几乎没有区别，只是 Tokio 是异步的，std 是同步的，例如 Tokio 的读写特征分别是 AsyncRead 和 AsyncWrite。
     *
     * ### AsyncRead 和 AsyncWrite
     *
     * AsyncRead 和 AsyncWrite 是非常基础的特征，很多类型和数据结构都实现了它们：
     * - 部分类型如 TcpStream，File，Stdout 实现了它们，支持异步读写
     * - 部分数据结构如 `Vec<u8>、&[u8]` 也实现了它们：，支持直接将这些**数据结构作为读写器( reader / writer)**，一些常见的 buff 容器其实就可以视为读取器和写入器。
     *
     * 这两个特征为字节流的异步读写提供了便利，通常会使用 `AsyncReadExt` 和 `AsyncWriteExt` 提供的工具方法，这些方法都是 async 声明，需要 .await 调用。
     *
     * buffer 作为读取器还是写入器是根据实际场景决定的：
     * - 如果从 buffer 中读取内容复制到写入器（writer）中，那么 buffer 就是读取器（reader），如 `&[u8]`
     * - 如果从读取器（reader）中读取内容并写入到 buffer 中，那么 buffer 就是写入器（writer），如 `&[u8]`
     *
     * 注意，是切片 `&[u8]` 而不是字节数组引用 `&[u8; length]`
     *
     * #### read read_to_end
     * AsyncReadExt::read 是一个异步方法可以将数据读入缓冲区( buffer )中，然后返回读取的字节数。
     * 需要注意的是：当 read 返回 Ok(0) 时，意味着字节流( stream )已经关闭，在这之后继续调用 read 会立刻完成，依然获取到返回值 Ok(0)。 例如，字节流如果是 TcpStream 类型，那 Ok(0) 说明该连接的读取端已经被关闭(写入端关闭，会报其它的错误)。
     * ```rust
     * use tokio::{self, AsyncReadExt, AsyncWriteExt};
     *
     * // 部分数据结构如 `Vec<u8>、&[u8]` 也实现了它们：，支持直接将这些**数据结构作为读写器( reader / writer)**，一些常见的 buffer 容器其实就可以视为读取器和写入器。
     * let mut file = File::open(r"Cargo.toml").await.unwrap();
     * let mut buffer = [0; 1024]; // 写入器
     *
     * // 由于 buffer 的长度限制，当次的 `read` 调用最多可以从文件中读取 1024 个字节的数据
     * let n = file.write(&mut buffer).await.unwrap();
     * println!("The bytes: {:?}", buffer);
     * ```
     *
     * AsyncReadExt::read_to_end 方法会从字节流中读取所有的字节，直到遇到 EOF。
     * ```rust
     * let mut file = File::open("Cargo.toml").await.unwrap();
     * // 写入器
     * let mut buffer = Vec::new();
     * let n = file.read_to_end(&mut buffer).await.unwrap();
     * println!("The bytes: {:?}", buffer);
     * ```
     *
     * 因为 `&[u8]` 实现了 AsyncRead 特征，所以可以直接将 `&[u8]` 作为读取器。
     *
     * #### write write_all
     *
     * AsyncWriteExt::write 异步方法会尝试将缓冲区的内容写入到写入器( writer )中，同时返回写入的字节数。
     * ```rust
     * let mut file = File::create("public/foo.txt").await?;
     * // 读取器
     * // let buffer = "Hello World".as_bytes();
     * let buffer = b"Hello World";
     * let n = file.write(buffer).await.unwrap();
     * println!("Wrote the first {} bytes of 'some bytes'.", n);
     * ```
     *
     * `b"some bytes"` 写法可以将一个 &str 字符串转变成一个字节数组：&[u8;10]，然后 write 方法又会将这个 &[u8;10] 的数组类型隐式强转为数组切片: &[u8]。
     * `"some bytes".to_bytes()` 函数则可以直接将字符串转变为字节切片。
     *
     * AsyncWriteExt::write_all 将缓冲区的内容全部写入到写入器中，因为全部写入，所以不再返回字节数。
     * ```rust
     * let mut file = File::create(r"public/foo.txt").await?;
     * // 读取器
     * // let buffer = "Hello World".as_bytes();
     * let buffer = b"Hello World";
     * file.write_all(buffer).await.unwrap();
     * ```
     * 因为 `&[u8]` 实现了 AsyncWrite 特征，所以可以直接将 `&[u8]` 作为写入器。
     *
     *
     * 更多函数阅读：https://docs.rs/tokio/latest/tokio/io/index.html
     *
     * ### 实用函数
     * read 和 write 是最基础的操作，和标准库一样，tokio::io 模块包含了多个实用的封装好的函数或 API，可以用于处理标准输入/输出/错误等。
     * 例如，tokio::io::copy 异步的将读取器( reader )中的内容拷贝到写入器( writer )中。
     *
     * ```rust
     * let mut file = File::create(r"public/foo.txt").await?;
     * // 读取器
     * let mut buffer = "Hello World".as_bytes();
     *
     * io::copy(&mut buffer, &mut file).await.unwrap();
     * ```
     *
     * ### 回声服务 （Echo）
     * 如同写代码必写 hello, world，实现 web 服务器，往往会选择实现一个回声服务。该服务会将用户的输入内容直接返回给用户，就像回声壁一样。
     * 具体来说，就是从用户建立的 TCP 连接的 socket 中读取到数据，然后立刻将同样的数据写回到该 socket 中。因此客户端会收到和自己发送的数据一模一样的回复。
     *
     * 和 async Web 服务器实现类似，基本的服务器框架：通过 loop 循环接收 TCP 连接，然后为每一条连接创建一个单独的任务去处理。
     *
     * ```rust
     * let listener = TcpListener::bind("127.0.0.1:6330").await?;
     *
     * async fn process(stream: net::TcpStream) {}
     *
     * loop {
     *     let (stream, addr) = listener.accept().await?;
     *     tokio::spawn(async move { process(stream).await });
     * }
     * ```
     *
     * 然后使用 `io::copy` 函数完成回声服务。copy 函数有两个参数：读取器的可变引用，写入器的可变引用，现在需要将读取器中的数据直接拷贝到写入器中。
     *
     * 在当前服务中，读取器和写入器都是 stream，根据借用规则，copy 不能同时操作两个 stream 的可变引用：
     * ```rust
     * io::copy(&mut stream, &mut stream).await
     * ```
     *
     * 借用规则限制只能操作一个变量的一个可变引用，这里 stream 不能既做读取器又做写入器。
     * 任何一个读写器( reader + writer )都可以使用 io::split 方法进行分离，最终返回一个读取器和写入器，这两者可以单独使用。
     * 实际上，io::split 可以用于任何同时实现了 AsyncRead 和 AsyncWrite 的值，它的内部使用了 Arc 和 Mutex 来实现相应的功能。
     *
     * ```rust
     * let listener = TcpListener::bind("127.0.0.1:6330").await?;
     *
     * async fn process(mut stream: net::TcpStream) {
     *     let (mut reader, mut writer) = io::split(stream);
     *
     *     if io::copy(&mut reader, &mut writer).await.is_err() {
     *         eprintln!("failed to copy");
     *     };
     * }
     *
     * loop {
     *     let (stream, addr) = listener.accept().await?;
     *     tokio::spawn(async move { process(stream).await });
     * }
     * ```
     *
     * `io::split` 利用 Mutex 会有一定的性能损耗，还有两种方式可以分离读写器：
     * - TcpStream::split会获取字节流的引用，然后将其分离成一个读取器和写入器。但由于使用了引用的方式，它们俩必须和 split 在同一个任务中。 优点就是，这种实现没有性能开销，因为无需 Arc 和 Mutex。
     * - TcpStream::into_split还提供了一种分离实现，分离出来的结果可以在任务间移动，内部是通过 Arc 实现。
     *
     * ```rust
     * let listener = TcpListener::bind("127.0.0.1:6330").await?;
     *
     * async fn process(mut stream: net::TcpStream) {
     *     let (mut reader, mut writer) = stream.split();
     *
     *     if io::copy(&mut reader, &mut writer).await.is_err() {
     *         eprintln!("failed to copy");
     *     };
     * }
     *
     * loop {
     *     let (stream, addr) = listener.accept().await?;
     *     tokio::spawn(async move { process(stream).await });
     * }
     * ```
     *
     * #### 手动拷贝
     * 如果不适用 io::copy，也可以手动实现复制过程，read 和 write 的过程都是间隔非一次性完成的，所以需要 loop。
     * 当然并不需要担心 loop 会导致性能问题，因为当 read 和 write 切换任务时，loop 会被暂停，使用 read 和 write 等方法需要导入 AsyncRead 和 AsyncWrite 特征：
     * ```rust
     * let listener = TcpListener::bind("127.0.0.1:6330").await?;
     *
     * async fn process(mut stream: net::TcpStream) {
     *     let mut buffer = [0; 1024];
     *     loop {
     *         match stream.read(&mut buffer).await {
     *             Ok(0) => {
     *                 return;
     *             }
     *             Ok(n) => {
     *                 if stream.write_all(&buffer[..n]).await.is_err() {
     *                     return;
     *                 }
     *             }
     *             Err(_) => return,
     *         }
     *     }
     * }
     *
     * loop {
     *     let (stream, addr) = listener.accept().await?;
     *     tokio::spawn(async move { process(stream).await });
     * }
     * ```
     *
     * ### 堆分配缓冲区
     * 在 .await 中使用缓冲区时，通常需要**把缓冲区分配在堆上**：
     * ```rust
     * let mut buf = vec![0; 1024]; // 分配在堆上
     *
     * let mut buf = [0; 1024]; // 分配在栈上
     * ```
     *
     * 这是因为 .await 时刻，当前任务需要保存所有作用域跨域 .await 的变量，以支持下一次 task 被恢复运行。
     *
     * ```rust
     * struct Task {
     *     task: enum {
     *         AwaitingRead {
     *             socket: TcpStream,
     *             buf: [BufferType],
     *         },
     *         AwaitingWriteAll {
     *             socket: TcpStream,
     *             buf: [BufferType],
     *         }
     *
     *     }
     * }
     * ```
     *
     * 栈数组要被使用，就必须存储在相应的结构体内，其中两个结构体分别持有了不同的栈数组 [BufferType]，这种方式会导致任务结构变得很大。特别地，我们选择缓冲区长度往往会使用分页长度(page size)，因此使用栈数组会导致任务的内存大小变得很奇怪甚至糟糕：$page-size + 一些额外的字节。
     * 当然，编译器会帮助我们做一些优化。例如，会进一步优化 async 语句块的布局，而不是像上面一样简单的使用 enum。在实践中，变量也不会在枚举成员间移动。
     * 但是再怎么优化，任务的结构体至少也会跟其中的栈数组一样大，因此通常情况下，使用堆上的缓冲区会高效实用的多。
     *
     * 当任务因为调度在线程间移动时，存储在栈上的数据需要进行保存和恢复，过大的栈上变量会带来不小的数据拷贝开销，因此，存储大量数据的变量最好放到堆上。
     *
     * 阅读：https://course.rs/advance-practice/io.html#在堆上分配缓冲区
     *
     *
     * ### 常见问题
     *
     * #### 处理 EOF
     * 在使用 read 读取数据流时，每次只是读取一部分，所以需要 loop 来不断的使用 read 进行读取，当 read 返回 Ok(0) 时就代表 TCP 连接的读取端关闭。
     * 此时需要打破循环，否则 loop 使用 read 会一直返回 Ok(0)，这是没有阻塞任务的，会导致 CPU 立刻跑满 100%。
     *
     * 忘记在 EOF 时退出读取循环是一个网络编程中常见的 bug。
     *
     * ```rust
     * loop {
     *     match socket.read(&mut buf).await {
     *         Ok(0) => return,
     *         // ... 其余错误处理
     *     }
     * }
     * ```
     *
     * ### Frame
     * 在认识 tokio::io 的基础操作后，就可以开始实现 mini-redis 的数据帧。
     *
     * 在 redis 各种指令操作中，命令和数据都是字节流数据，在操作上处于比较底层的位置，所以会比较麻烦，比如缓冲区分配需要手动实现等。
     * 帧相比字节流，封装了一定的结构，支持在更高的视角上操作数据。
     *
     * 帧除了数据之外，并不具备任何语义，每个帧就是一个数据单元，通过帧操作可以将字节流转换成帧组成的流。
     * 所以帧解析层并不包含 redis 的命令解析和实现，它仅是对字节流的一层封装，redis 的命令解析和实现会在更高的层次进行。
     *
     * HTTP 帧结构
     * ```rust
     * enum HttpFrame {
     *     RequestHead {
     *         method: Method,
     *         uri: Uri,
     *         version: Version,
     *         headers: HeaderMap,
     *     },
     *     ResponseHead {
     *         status: StatusCode,
     *         version: Version,
     *         headers: HeaderMap,
     *     },
     *     BodyChunk {
     *         chunk: Bytes,
     *     },
     * }
     * ```
     *
     * 为了实现 mini-redis 的帧，这里先借助 mini-redis 的 Frame 实现缓冲读取、帧解析、缓冲写入功能。
     * 缓冲读取、帧解析、缓冲写入是 Connection 结构体实现的，里面包含了一个 TcpStream 以及对帧进行读写的方法:
     * ```rust
     * use tokio::net::TcpStream;
     * use mini_redis::{Frame, Result};
     *
     * struct Connection {
     *     stream: TcpStream,
     *     // ... 这里定义其它字段
     * }
     *
     * impl Connection {
     *     /// 从连接读取一个帧
     *     ///
     *     /// 如果遇到EOF，则返回 None
     *     pub async fn read_frame(&mut self)
     *         -> Result<Option<Frame>>
     *     {
     *       // 具体实现
     *     }
     *
     *     /// 将帧写入到连接中
     *     pub async fn write_frame(&mut self, frame: &Frame)
     *         -> Result<()>
     *     {
     *         // 具体实现
     *     }
     * }
     * ```
     *
     * 组成帧的基本单元是字节，使用 TcpStream::read 读取字节流时会返回任意多的数据(填满传入的缓冲区 buffer)，
     * 这些数据对于帧结构来说是不确定的，它可能是帧的一部分、一个帧、多个帧。
     *
     * 而 read_frame 方法会等到一个完整的帧都读取完毕后才返回，所以这里需要 read_frame 底层调用 TcpStream::read 读取到数据时，需要做一些缓冲操作：
     * - 当数据不满足一个帧结构要求时，将数据先缓冲起来，继续等待并读取数据，直到读取的数据满足一个帧结构体的要求
     * - 当 TcpStream::read 读取的数据大于一个帧结构时，如读到多个帧，此时第一个帧会被返回，然后剩下的数据会被缓冲起来，等待下一次 read_frame 被调用。
     *
     * 总的来说，Connection 拥有一个读取缓冲区，数据首先从 socket 中读取到缓冲区中，接着当外部调用 Connection::read_frame 进行读取时，这些数据会被解析为帧，当一个帧被解析后，该帧对应的数据会从缓冲区被移除。
     *
     *
     * **具体实现**
     *
     * Connection::read_frame 读取成功后返回一个帧数据，读取到最后需要返回结束标识，并且读取的过程中可能发生错误，所以先借助 mini-redis 的 Frame 结构体，将 read_frame 的返回类型定义为 `mini_redis::Result<Option<Frame>>`。
     *
     * 在实现 read_frame 过程中，需要手动实现缓冲区读取与移除，这里需要考虑避免覆盖之前读取的数据，在缓冲区满了后扩容缓冲区，增加缓冲区长度。
     * 这里需要用到一个属性：游标 (cursor)。事实上在网络编程中，通过字节数组与游标的组合来实现读取数据的方式非常常见。
     * 通过游标( cursor )跟踪已经读取的数据，将下次读取的数据写入到游标之后的缓冲区，这样就不会让新读取的数据将之前读取的数据覆盖掉。
     * 此外一旦缓冲区满了，还需要增加缓冲区的长度，这样才能继续写入数据。
     *
     * 使用 `TcpStream::read` 方法和缓冲区 `&[u8]` 实现读取逻辑：
     *
     * ```rust
     * pub struct Connection {
     *     stream: net::TcpStream,
     *     buffer: Vec<u8>,
     *     cursor: usize,
     * }
     *
     * impl Connection {
     *     pub fn new(stream: net::TcpStream) -> Connection {
     *         Connection {
     *             stream,
     *             // 分配一个缓冲区，具有 4kb 的缓冲长度
     *             buffer: Vec::with_capacity(1024 * 4),
     *             cursor: 0,
     *         }
     *     }
     * }
     * ```
     * 以上代码定义了 Connection 结构体，并提供生成 Connection 实例的 `new` 函数。接下来定义 read_frame 方法。
     * read_frame 内部使用循环的方式读取数据，直到一个完整的帧被读取到时才会返回。当然，当远程的对端关闭了连接后，也会结束并返回。
     *
     * ```rust
     * use tokio::net::TcpStream;
     *
     * impl Connection {
     *     pub async fn read_frame(&mut self) -> mini_redis::Result<Option<Frame>> {
     *         loop {
     *             // 第一步：
     *             // 尝试从缓冲区的数据中解析出一个数据帧，只有当数据足够被解析时，才会返回对应的帧数据，否则返回 None
     *             if let Some(frame) = self.parse_frame()? {
     *                 return Ok(Some(frame));
     *             }
     *
     *             // 第二步：
     *             // 如果缓冲区中的数据还不足以被解析为一个数据帧，需要从 socket 中读取更多的数据
     *             // 使用 read 读取，将读取写入到写入器（缓冲区）中，并返回读取到的字节数
     *             // 这里需要考虑避免覆盖之前读取的数据，在缓冲区满了后扩容缓冲区，增加缓冲区长度
     *             // 通常缓冲区的写入和移除是通过游标 (cursor) 来实现的。
     *             //
     *             // 当返回的字节数为 0 时，代表着读到了数据流的末尾，说明了对端关闭了连接。
     *             // 此时需要检查缓冲区是否还有数据，若没有数据，说明所有数据成功被处理，
     *             // 若还有数据，说明对端在发送字节流的过程中断开了连接，导致只发送了部分数据，需要抛出错误
     *
     *             // 先检查缓冲区长度，确保缓冲区长度足够
     *             if self.cursor == self.buffer.len() {
     *                 self.buffer.resize(self.cursor * 2, 0);
     *             }
     *
     *             // 从缓冲区的游标位置开始写入新数据，避免旧数据被覆盖
     *             // read 方法读取的数据不会超出剩下的buffer长度，当 buffer 没有剩余空间时，read 方法就会结束读取
     *             let n = self.stream.read(&mut self.buffer[self.cursor..]).await?;
     *
     *             // 如果读取数据为空，需要通过缓冲区是否还有数据来判断是否正常关闭
     *             if 0 == n {
     *                 if self.buffer.is_empty() {
     *                     return Ok(None);
     *                 } else {
     *                     return Err("connection reset by peer".into());
     *                 }
     *             }
     *
     *             // 如果读取的数据不为空，则更新游标位置，继续下一轮循环
     *             self.cursor += n;
     *         }
     *     }
     * }
     * ```
     *
     * **BufMut 特征**
     *
     * 每次需要手动判断游标边界显得非常麻烦，bytes 提供了 BytesMut，方便解决此类问题，使用 BytesMut 作为缓冲区类型，它是 Bytes 的可变版本：
     * ```rust
     * use bytes::BytesMut;
     * use tokio::net::TcpStream;
     *
     * pub struct Connection {
     *     stream: TcpStream,
     *     buffer: BytesMut,
     * }
     *
     * impl Connection {
     *     pub fn new(stream: TcpStream) -> Connection {
     *         Connection {
     *             stream,
     *             // 分配一个缓冲区，具有4kb的缓冲长度
     *             buffer: BytesMut::with_capacity(4096),
     *         }
     *     }
     * }
     * ```
     *
     * // TODO
     */

    // {
    //     // 部分数据结构如 `Vec<u8>、&[u8]` 也实现了它们：，支持直接将这些**数据结构作为读写器( reader / writer)**，一些常见的 buffer 容器其实就可以视为读取器和写入器。
    //     let mut file = File::open(r"Cargo.toml").await.unwrap();
    //     // 写入器
    //     let mut buffer = [0; 1024];
    //     // 由于 buffer 的长度限制，当次的 `read` 调用最多可以从文件中读取 1024 个字节的数据
    //     let n = file.write(&mut buffer).await.unwrap();
    //     println!("The bytes: {:?}", buffer);

    //     let mut file = File::open("Cargo.toml").await.unwrap();
    //     // 写入器
    //     let mut buffer = Vec::new();
    //     let n = file.read_to_end(&mut buffer).await.unwrap();
    //     println!("The bytes: {:?}", buffer);
    // }

    // {
    //     // 部分数据结构如 `Vec<u8>、&[u8]` 也实现了它们：，支持直接将这些**数据结构作为读写器( reader / writer)**，一些常见的 buffer 容器其实就可以视为读取器和写入器。
    //     let mut file = File::create("public/foo.txt").await?;
    //     // 读取器
    //     // let buffer = "Hello World".as_bytes();
    //     let buffer = b"Hello World";
    //     let n = file.write(buffer).await.unwrap();
    //     println!("Wrote the first {} bytes of 'some bytes'.", n);

    //     let mut file = File::create(r"public/foo.txt").await?;
    //     // 读取器
    //     // let buffer = "Hello World".as_bytes();
    //     let buffer = b"Hello World";
    //     file.write_all(buffer).await.unwrap();
    // }

    // {
    //     let mut file = File::create(r"public/foo.txt").await?;
    //     // 读取器
    //     let mut buffer = "Hello World".as_bytes();

    //     io::copy(&mut buffer, &mut file).await.unwrap();
    // }

    // {
    //     let listener = TcpListener::bind("127.0.0.1:6330").await?;

    //     async fn process(mut stream: net::TcpStream) {
    //         let (mut reader, mut writer) = io::split(stream);

    //         if io::copy(&mut reader, &mut writer).await.is_err() {
    //             eprintln!("failed to copy");
    //         };
    //     }

    //     loop {
    //         let (stream, addr) = listener.accept().await?;
    //         tokio::spawn(async move { process(stream).await });
    //     }
    // }

    // {
    //     let listener = TcpListener::bind("127.0.0.1:6330").await?;

    //     async fn process(mut stream: net::TcpStream) {
    //         let (mut reader, mut writer) = stream.split();

    //         if io::copy(&mut reader, &mut writer).await.is_err() {
    //             eprintln!("failed to copy");
    //         };
    //     }

    //     loop {
    //         let (stream, addr) = listener.accept().await?;
    //         tokio::spawn(async move { process(stream).await });
    //     }
    // }

    // {
    //     let listener = TcpListener::bind("127.0.0.1:6330").await?;

    //     async fn process(mut stream: net::TcpStream) {
    //         let mut buffer = [0; 1024];
    //         loop {
    //             match stream.read(&mut buffer).await {
    //                 Ok(0) => {
    //                     return;
    //                 }
    //                 Ok(n) => {
    //                     if stream.write_all(&buffer[..n]).await.is_err() {
    //                         return;
    //                     }
    //                 }
    //                 Err(_) => return,
    //             }
    //         }
    //     }

    //     loop {
    //         let (stream, addr) = listener.accept().await?;
    //         tokio::spawn(async move { process(stream).await });
    //     }
    // }

    {
        pub struct Connection {
            stream: net::TcpStream,
            buffer: Vec<u8>,
            cursor: usize,
        }
        impl Connection {
            pub fn new(stream: net::TcpStream) -> Connection {
                Connection {
                    stream,
                    // 分配一个缓冲区，具有 4kb 的缓冲长度
                    buffer: Vec::with_capacity(1024 * 4),
                    cursor: 0,
                }
            }

            pub async fn read_frame(&mut self) -> mini_redis::Result<Option<Frame>> {
                loop {
                    // 第一步：
                    // 尝试从缓冲区的数据中解析出一个数据帧，只有当数据足够被解析时，才会返回对应的帧数据，否则返回 None
                    if let Some(frame) = self.parse_frame()? {
                        return Ok(Some(frame));
                    }

                    // 第二步：
                    // 如果缓冲区中的数据还不足以被解析为一个数据帧，需要从 socket 中读取更多的数据
                    // 使用 read 读取，将读取写入到写入器（缓冲区）中，并返回读取到的字节数
                    // 这里需要考虑避免覆盖之前读取的数据，在缓冲区满了后扩容缓冲区，增加缓冲区长度
                    // 通常缓冲区的写入和移除是通过游标 (cursor) 来实现的。
                    //
                    // 当返回的字节数为 0 时，代表着读到了数据流的末尾，说明了对端关闭了连接。
                    // 此时需要检查缓冲区是否还有数据，若没有数据，说明所有数据成功被处理，
                    // 若还有数据，说明对端在发送字节流的过程中断开了连接，导致只发送了部分数据，需要抛出错误

                    // 先检查缓冲区长度，确保缓冲区长度足够
                    if self.cursor == self.buffer.len() {
                        self.buffer.resize(self.cursor * 2, 0);
                    }

                    // 从缓冲区的游标位置开始写入新数据，避免旧数据被覆盖
                    // read 方法读取的数据不会超出剩下的buffer长度，当 buffer 没有剩余空间时，read 方法就会结束读取
                    let n = self.stream.read(&mut self.buffer[self.cursor..]).await?;

                    // 如果读取数据为空，需要通过缓冲区是否还有数据来判断是否正常关闭
                    if 0 == n {
                        if self.buffer.is_empty() {
                            return Ok(None);
                        } else {
                            return Err("connection reset by peer".into());
                        }
                    }

                    // 如果读取的数据不为空，则更新游标位置，继续下一轮循环
                    self.cursor += n;
                }
            }
        }
    }

    {
        pub struct Connection {
            stream: net::TcpStream,
            buffer: BytesMut,
            cursor: usize,
        }
        impl Connection {
            pub fn new(stream: net::TcpStream) -> Connection {
                Connection {
                    stream,
                    // 分配一个缓冲区，具有 4kb 的缓冲长度
                    buffer: BytesMut::with_capacity(1024 * 4),
                    cursor: 0,
                }
            }

            pub async fn read_frame(&mut self) -> mini_redis::Result<Option<Frame>> {
                loop {
                    // 第一步：
                    // 尝试从缓冲区的数据中解析出一个数据帧，只有当数据足够被解析时，才会返回对应的帧数据，否则返回 None
                    if let Some(frame) = self.parse_frame()? {
                        return Ok(Some(frame));
                    }

                    // 第二步：
                    // 如果缓冲区中的数据还不足以被解析为一个数据帧，需要从 socket 中读取更多的数据
                    // 使用 read 读取，将读取写入到写入器（缓冲区）中，并返回读取到的字节数
                    // 这里需要考虑避免覆盖之前读取的数据，在缓冲区满了后扩容缓冲区，增加缓冲区长度
                    // 通常缓冲区的写入和移除都是通过游标 (cursor) 来实现的。
                    //
                    // 当返回的字节数为 0 时，代表着读到了数据流的末尾，说明了对端关闭了连接。
                    // 此时需要检查缓冲区是否还有数据，若没有数据，说明所有数据成功被处理，
                    // 若还有数据，说明对端在发送字节流的过程中断开了连接，导致只发送了部分数据，需要抛出错误

                    if 0 = self.stream.read(&mut self.buffer).await? {}
                }
            }
        }
    }

    Ok(())
}
