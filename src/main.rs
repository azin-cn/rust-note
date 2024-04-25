use ilearn::{run, Config};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::{
    fmt::{Debug, Display},
    ops::{Add, Index},
};

fn main() {
    /*
     * ## 智能指针（二）Box 对象分配
     * `Box<T>` 是 Rust 中最常见的智能指针，功能是将一个值分配到堆上，然后在栈上保留一个**智能指针**指向堆上的数据。
     *
     * 要想用好Box，需要深入了解计算机堆栈概念。
     *
     * **栈**
     *
     * 栈内存从**高位地址向下增长**，且栈内存是连续分配的，一般来说操作系统对栈内存的大小都有限制，因此 C 语言中无法创建任意长度的数组（存储在栈）。
     * 在 Rust 中，main 线程的栈大小是 8MB，普通线程是 2MB，在函数被调用时 Rust 会在线程内存中创建一个**临时栈空间**，调用结束后 Rust 会让这个栈空间里的所有对象自动进入 Drop 流程，最后栈顶指针自动移动到上一个调用栈顶，无需程序员手动干预，因而栈内存申请和释放是非常高效的。
     *
     * **堆**
     *
     * 与栈相反，堆上内存则是从低位地址向上增长，堆内存通常只受物理内存限制，而且通常是不连续的，因此从性能的角度看，栈往往比堆更高。
     *
     * 相比其它语言，Rust 堆上对象还有一个特殊之处，它们都拥有一个所有者，因此**受所有权规则的限制**：当赋值时，发生的是**所有权的转移**（只需浅拷贝栈上的引用或智能指针即可）
     *
     *
     * ```rust
     * fn foo(x: &str) -> String {
     *     let s = "Hello, ".to_string() + x;
     *     s
     * }
     *
     * let
     * println!("{}", foo("World"));
     * ```
     * 在 foo 函数中，s 是一个 String 类型，它是由存储在堆中的实际类型数据和存储在栈中的智能指针结构体（指向堆数据）共同组成的。
     * 当 s 被从 foo 函数转移给 x 变量时，只需要将 s 栈上的智能指针复制一份赋予给 x，而底层数据不发生改变即可完成堆数据的所有权从 foo 函数内部到 x 的转移。
     *
     * ### 栈与堆的性能
     * 很多人可能会觉得栈的性能肯定比堆高，其实未必，这里有一个大概：
     * - 小型数据，在栈上的分配性能和读取性能都要比堆上高
     * - 中型数据，栈上分配性能高，但是读取性能和堆上并无区别，因为无法利用寄存器或 CPU 高速缓存（空间非常小），最终还是要经过一次内存寻址
     * - 大型数据，只建议在堆上分配和使用
     *
     * 总结：栈的**分配速度**比堆快，但是**读取速度**往往取决于数据能不能放入寄存器或 CPU 高速缓存。因此不要因为堆的性能不如栈这个印象，就总是优先选择使用栈，导致代码更复杂的实现。
     *
     * ### Box 的使用场景
     * 由于 Box 是简单的封装，除了将值存储在堆上外，并没有其它性能上的损耗。而性能和功能往往是鱼和熊掌，因此 Box 相比其它智能指针，功能较为单一，可以在以下场景中使用它：
     * - 特意的将数据分配在堆上
     * - 数据较大时，又不想在转移所有权时进行数据拷贝
     * - 类型的大小在编译期无法确定，但是我们又需要固定大小的类型时（递归对象，切片等）
     * - 特征对象，用于说明对象实现了一个特征，而不是某个特定的类型
     *
     *
     * #### 使用 `Box<T>` 将数据存储在堆上
     *
     * 如果一个变量拥有一个数值，即直接声明变量 `let a = 3`，那变量 a 必然是存储在栈上的，如果想要 a 的值存储在堆上就需要使用 `Box<T>`：
     * ```rust
     * let a = Box::new(2);
     * println!("a = {}", a); // a = 3
     * // let b = a + 1; // 代码将报错 cannot add `{integer}` to `Box<{integer}>`
     * ```
     * 这样就可以创建一个智能指针指向了存储在堆上的 3，并且 a 持有了该智能指针，而智能指针往往都实现了 Deref 和 Drop 特征，因此：
     * - println! 可以正常打印出 a 的值，是因为它隐式地调用了 Deref 对智能指针 a 进行了解引用 `*a`，即 `println!("{}", *a);`
     * - 最后一行代码 `let b = a + 1` 报错，是因为**在表达式中不能自动地执行隐式 Deref 解引用操作**，需要手动使用 `*` 操作符来显式的进行解引用 `let b = *a + 1`
     * - a 持有的智能指针将在作用域结束（main 函数结束）时，被释放掉，这是因为 `Box<T>` 实现了 Drop 特征
     *
     * > Rust 会在方法调用和字段访问时自动应用解引用强制多态（deref coercions），这意味着如果类型实现了 Deref trait，Rust 会自动将引用类型转换为目标类型。
     * > 在一些其他情况下，如在标准比较操作或赋值中，Rust 不会自动应用解引用：**在表达式中不能自动地执行隐式 Deref 解引用操作**，需要手动使用 `*` 操作符解引用。
     *
     * #### 避免栈上数据的拷贝
     *
     * 当栈上数据转移所有权时，实际上是把**底层数据拷贝了一份**，最终新旧变量各自拥有不同的数据，因此**所有权未转移**。
     * 而堆上则不然，底层数据并不会被拷贝，转移所有权仅仅是**复制一份栈中的指针**，再将新的指针赋予新的变量，然后让拥有旧指针的变量失效，最终完成了**所有权转移**：
     *
     * ```rust
     * let arr = [0;1000]; // 在栈上创建一个长度为1000的数组
     * let arr1 = arr; // 将arr所有权转移arr1，由于 `arr` 分配在栈上，因此这里实际上是直接重新深拷贝了一份数据
     * ```
     *
     * #### 将动态大小类型变为 Sized 固定大小类型
     * Rust 需要在编译时知道类型占用多少空间，如果一种类型在编译时无法知道具体的大小，那么被称为动态大小类型 DST。
     * 在闭包作为函数返回值（特征对象）和不定长类型（切片）章节中就曾使用 `Box` 将动态大小类型DST转化为定长类型（Sized）。
     *
     * 除了特征对象和切片外，这里还有一种无法在编译时知道大小的类型是**递归类型**：在类型定义中又使用到了自身，或者说该类型的值的一部分可以是相同类型的其它值。
     *
     * 这种值的嵌套理论上可以无限进行下去，所以 Rust 不知道递归类型需要多少空间，以函数式语言中常见的 Cons List 为例，它的每个节点包含一个 i32 值，还包含了一个新的 List，递归类型声明：
     *
     * ```rust
     * enum List {
     *     Cons(i32, List),
     *     Nil,
     * }
     * ```
     *
     * 但是上面这段代码声明是错误的，因为这种嵌套可以无限进行下去，Rust 认为该类型是一个 DST 类型：
     * ```shell
     * recursive type `List` has infinite size //递归类型 `List` 拥有无限长的大小
     * ```
     *
     * 该数据类型可以无限拓展，因此要将 List 改成存储在堆上，可使用 `Box`, `Rc`, `&` 阻断该数据类型在栈上的无限拓展的可能，即变为在栈上存储指针（固定大小），堆存储实际数据：
     * ```rust
     * enum List {
     *     Cons(i32, Box<List>), // 固定大小，因为 i32 和 Box 都是固定大小
     *     Nil,
     * }
     * ```
     *
     * #### 特征对象
     * 特征是一种动态尺寸类型（Dynamically Sized Types，DST），即特征本身不具有固定的大小，因此不能直接实例化为对象。
     * 在Rust中，特征通常通过指针（如 `Box<T>、&T`）来使用，这些指针指向实现了该特征的具体类型的实例。
     * 这些**对动态尺寸类型的一种封装，使其可以通过具体的、已知大小的指针类型（如 `Box<dyn Trait>` 或 `&dyn Trait`）来使用，这种封装类型就是一个特征对象**。因此特征对象可以被视为具体的、已知大小的类型。
     *
     * 在这里需要更新前几章的描述：特征对象是动态尺寸类型，这是有误的。正确的认识是：特征是动态尺寸类型，而特征对象是对特征的一种封装，使特征可以通过具体的，已知大小的指针类型来描述，因此特征对象是一个定长类型（Sized）。
     *
     * ### Box 内存布局
     *
     * 前面提到过：
     * 不能简单的将变量与类型视为只是一块栈内存或一块堆内存数据，比如 Vec 类型，rust将其分成两部分数据：存储在堆中的实际类型数据与存储在栈上的管理信息数据。
     * 其中存储在栈上的管理信息数据是引用类型，包含实际类型数据的地址、元素的数量，分配的空间等信息，**rust 通过栈上的管理信息数据掌控实际类型数据的信息**。
     *
     * 因此来看一下几种常见的类型的内存模型，首先是 `Vec<i32>` 的内存布局：
     * ```txt
     * (stack)    (heap)
     * ┌──────┐   ┌───┐
     * │ vec1 │──→│ 1 │
     * └──────┘   ├───┤
     *            │ 2 │
     *            ├───┤
     *            │ 3 │
     *            ├───┤
     *            │ 4 │
     *            └───┘
     * ```
     * 智能指针存储在栈中，然后指向堆上的数组数据，String类型与Vec类型内存布局是类似的，栈上存储智能指针，堆上存储实际类型数据。
     *
     * 那如果数组中每个元素都是一个 Box 对象呢？来看看 `Vec<Box<i32>>` 的内存布局：
     * ```txt
     *                     (heap)
     * (stack)    (heap)   ┌───┐
     * ┌──────┐   ┌───┐ ┌─→│ 1 │
     * │ vec2 │──→│B1 │─┘  └───┘
     * └──────┘   ├───┤    ┌───┐
     *            │B2 │───→│ 2 │
     *            ├───┤    └───┘
     *            │B3 │─┐  ┌───┐
     *            ├───┤ └─→│ 3 │
     *            │B4 │─┐  └───┘
     *            └───┘ │  ┌───┐
     *                  └─→│ 4 │
     *                     └───┘
     * ```
     * 看出智能指针 vec2 依然是存储在栈上，然后指针指向一个存储在堆上的数组，该数组中每个元素都是一个 Box 智能指针，Box 智能指针又指向了存储在堆上的实际值。
     * 因此当我们从数组中取出某个元素时，取到的是对应的智能指针 Box，需要对该智能指针进行解引用，才能取出最终的值，以B1为例：B1 代表被 Box 分配到堆上的值 1。
     *
     * > Rust 会在方法调用和字段访问时自动应用解引用强制多态（deref coercions），在一些其他情况下，如在标准比较操作或赋值中，Rust 不会自动应用解引用：**在表达式中不能自动地执行隐式 Deref 解引用操作**。
     * > println! 实际上调用的就是Display特征的方法，所以println时存在自动解引用
     * 
     * ```rust
     * let arr = vec![Box::new(1), Box::new(2)];
     * let (first, second) = (&arr[0], &arr[1]);
     * let sum = **first + **second;
     * println!("{}, {}, {}", first, second, sum);
     * ```
     * 以上代码有几个值得注意的点：
     * - 使用 & 借用数组中的元素，否则会报所有权错误
     * - **表达式不能隐式的解引用**，因此必须使用 ** 做两次解引用，第一次将 `&Box<i32>` 类型转成 `Box<i32>`，第二次将 `Box<i32>` 转成 i32
     *
     * 
     *
     *
     *
     *
     *
     *
     *
     *
     */

    let arr1 = [0; 1000];
    let arr2 = arr1; // 由于数组存储在栈上，因此赋值转移时，深拷贝了一份数据
    println!("{:p}, {:p}", &arr1, &arr2);

    fn foo(x: &str) -> String {
        let s = "Hello, ".to_string() + x;
        s
    }
    let x = foo("World");
    println!("{}", x);

    // 智能指针往往都实现了 Deref 和 Drop 特征
    let a = Box::new(3);
    let a_deref = *a;
    println!("{}, {}", *a, a_deref);

    // 避免在栈上存储大型数据，以避免复制成本
    let mut arr1 = [0, 1000];
    let mut arr2 = arr1; // 复制了一份arr1数据
    let sum1: i32 = arr1.into_iter().map(|x| x + 1).sum();
    let sum2 = arr2.into_iter().map(|x| x + 1).sum::<i32>();
    arr1[0] = 1;
    println!("{}, {}", arr1[0], arr2[0]);

    // 将动态大小类型变为 Sized 固定大小类型
    enum List {
        // Cons(i32, List), 错误的，因为这个类型可以无限拓展，因此要存储在堆上，可使用  `Box`, `Rc`, `&`) 打断动态，即将在栈上存储指针，而不存储实际数据
        Cons(i32, Box<List>),
        Nil,
    }

    // BOX内存布局，自动解引用和手动解引用
    // Rust 会在方法调用和字段访问时自动应用解引用强制多态（deref coercions），在一些其他情况下，如在标准比较操作或赋值中，Rust 不会自动应用解引用：**在表达式中不能自动地执行隐式 Deref 解引用操作**。
    // println! 实际上调用的就是Display特征的方法，所以println时存在自动解引用
    let arr = vec![Box::new(0), Box::new(1)];
    let (first, second) = (&arr[0], &arr[1]);
    let sum = **first + **second;
    println!("{}, {}, {}", first, second, sum);
}
