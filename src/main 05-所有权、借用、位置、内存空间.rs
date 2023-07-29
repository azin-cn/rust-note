fn main() {
    /*
     * ## 认识Rust的内存
     * 胖指针：除了指针信息外，还保存有额外的元信息。
     *
     * Rust内存中，需要着重理解位置表达式和值这两个概念。
     *
     * ### 位置表达式
     * 其中，变量名是易于读取的指针表达，是内存地址的别名。变量就是一个位置（位置表达式，内存位置）！有自己的地址，自己的空间，自己保存的值。可能位于栈，可能位于堆，还可能位于全局内存区。
     *
     * 变量名是内存地址的别名，供人阅读；变量（位置）是一块内存，有自己的地址，自己的空间，自己保存的值。
     *
     * https://rust-book.junmajinlong.com/ch5/03_rust_place_value.html
     *
     * ### 值
     * 值就是存储到位置中的数据(即保存在内存中的数据)。值的类型有多种，如数值类型的值、字符类型的值、指针类型的值(包括裸指针和胖指针)，等等。
     *
     * ### 注意
     * - 每个位置（变量）就是它存储的值的所有者，因为每个值都只能存在一个位置，所以只能有一个所有者。
     * - 有时候，会将声明变量时的位置看作是变量(注意不是变量名)，或者将变量看作是位置。无论如何看待两者，我们内心需要明确的是，在能够将位置视为变量的情况下，变量或这种位置，是栈中的一块内存。
     * - 有时候，不能将位置视作变量。例如如果赋值给变量的是保存在堆中的数据(例如Vec类型)，那么变量中保存的是该数据的胖指针而不是实际数据。
     *
     * ### 总结
     * ```rs
     * let n = 1;
     * let v = vec![1, 2, 3, 4];
     * ```
     * 这两个变量产生的位置（内存）不一样的。
     * - 首先是n变量的声明，因为值1是原始数据类型，所以在n变量声明时，栈中开辟的一个位置（内存）可以直接存放值1。也就是说n这个变量（位置，内存）自己的地址别名叫n，代表的是这个位置（内存），存储的值是1。
     * - 其次是v变量的声明，因为Vec不是原始数据类型，所以在声明v变量时，栈中开辟的一个位置（内存）不是存放实际数据的，存放的是一个胖指针。这是因为在声明v变量时，显式开辟的栈内存（位置）就代表v变量，隐式开辟的堆中位置（内存）存放实际数据，栈位置（内存）存放的数据是堆地址，所以堆位置不代表任何变量。
     * - **变量v的值是那个胖指针，而不是堆中的那串实际数据。** 更严格地说，Vec类型的值，指的是那个胖指针数据即指针+其他元信息，而不是实际数据，变量v的值是那个胖指针而不是实际数据，变量v是胖指针这个值的所有者，而不是实际数据的所有者！
     *
     * ## 所有权
     * 所有权和借用是Rust控制内存的核心所在，参考一下文章。
     *
     * https://course.rs/basic/ownership/ownership.html
     *
     * https://github.com/sunface/rust-course/discussions/690#discussioncomment-3164352
     *
     * ### 注意
     * 基本数据都是存储在栈上，实现Copy特征的，也就是在赋值给其他变量时，不会转移所有权！
     *
     * 复杂数据的值存储在堆上，堆的指针存储在栈上，实现Move特征，在赋值时会转移所有权，可以调用clone()方法实现深度克隆。
     *
     * Rust是怎么保证内存安全呢？除了上述的操作之外，Rust会在每一个块（函数）结束后，自动调用drop函数释放这个块（函数）内的**值**，这个值类型是复杂类型。
     *
     * 这是因为基本数据类型存储在栈中，当这个块（函数）执行完成后，栈会弹出这些变量即相应的值就会被销毁，而在堆中的复杂数据需要drop函数进行释放。
     *
     * 转移所有权的过程是将原有变量（存储在栈）的信息复制到新变量，然后将原变量设置为未初始化。
     *
     * ## 借用
     * Rust 变量因为所有权的存在，所以在赋值/传递时会很麻烦，借用（borrowing）就是处理这种情况的概念。
     *
     * 借用与所有权相对，是一种操作/概念/意图/解释，而引用是类型，为了实现借用这个操作的具体实现。即概念上这种操作是借用，具体代码中的数据类型称为引用，
     *
     * borrowing分为两种，不可变借用（不可变引用数据类型）和可变借用（可变引用数据类型），以下简称为不可变引用和可变引用。
     *
     * 可变引用需要可变变量/值，即mut修饰的变量/值，不可变引用没有限制。可变引用和可变变量是不同的概念。
     *
     * ```rs
     * & 引用
     * mut 可变
     * &mut 可变引用
     * let a 不可变变量
     * let mut a 可变变量
     *
     * let a: String = String::from("hello world");
     * let b: &String = &String::from("hello world");
     * let c: &mut String = &mut String::from("Hello World");
     * let mut c: &mut String = &mut String::from("Hello World");
     * ```
     *
     * 引用作用域定义为一个引用使用开始到使用结束的范围，变量作用域是变量创建开始到某一个`}`为止的范围。
     *
     * 总结规则
     * - 不可变引用作用域中不能出现可变引用
     * - 可变引用作用域中不能出现可变引用、不可能引用
     *
     * ### 注意
     * - 引用是一种原始数据类型，存储在栈中，保存的值是地址值（指针），这个地址指向它引用的目标。引用在栈的位置（内存空间）代表某一个变量，这个变量是指针的所有者，而不是指针所指向数据的所有者！
     * - 引用的值，即引用的指向目标是指指向位置，是指它指向的那个变量（指向内存），值就是所指向变量的地址。这一点和复杂数据的内存分布不是同一个东西，需要分开理解。
     * - 其他语言的引用的意义就是指引用复杂数据（堆）的地址，但Rust中的引用是对一个变量的引用，引用是一种基本数据类型，用变量（栈空间）存储引用。
     * - Rust不允许存在对堆中同一个内存的多个指向，因为这可能会导致重复释放同一块堆内存的危险。但允许对栈中同一个数据的多个指向，这是因为栈内存由编译器负责维护，编译器知道栈中的某个内存是否安全(比如判断变量是否离开作用域被销毁、判断生命周期)，而堆内存是由程序员负责维护，程序员的行为是不安全的。这也是原始数据类型存放在栈的原因之一。
     * - 尽可能地让涉及到内存安全的概念实现在栈上，尽可能让程序员远离对堆内存的操作。
     *
     * ## 何时产生位置（内存空间）
     * 1. 产生变量时会产生位置，例如声明变量，或者形参和规则3类似
     * 2. 产生新值时会产生位置（ 引用(&) 产生新值 [地址]，解引用(\*) 产生新值 ）
     * 3. 保存值时，如函数传参、返回值，会产生位置（临时变量）
     * 4. 使用值时，就会产生位置。例如在模式匹配，使用枚举的某一个值
     *
     * 位置一旦初始化赋值，就会有一个永远不变的地址，直到销毁。换句话说，变量一旦初始化，无论它之后保存的数据发生了什么变化，它的地址都是固定不变的。也说明了，编译器在编译期间就已经安排好了所有位置的分配。这个位置指的是栈中的位置（内存空间）！
     * ```rs
     * fn main() {
     *      let mut n = "hello".to_string();  // n是一个栈中的位置，保存了一个胖指针指向堆中数据
     *      println!("n: {:p}", &n);  // &n产生一个位置，该位置中保存指向位置n的地址值
     *      let m = n;     // 将n中的胖指针移给了m，m保存胖指针指向堆中数据，n变回未初始化状态
     *      println!("m: {:p}", &m);  // &m产生一个位置，该位置中保存指向位置m的地址值
     *      n = "world".to_string();  // 重新为n赋值，位置n保存另一个胖指针，但位置n还是那个位置
     *      println!("n: {:p}", &n);  // &n产生一个位置，该位置中保存指向位置n的地址值
     * }
     * ```
     *
     * ## 变量声明
     * ```rs
     * let n = 1;
     * ```
     * Rust的变量声明分为两个步骤，第一步骤是声明变量，即初始化但未赋值，第二步骤是真正赋值，赋值是一个移动操作。这个和JavaScript的变量提升概念类似。只初始化的变量不能使用，这个状态也是变量的值的所有权被转移后的状态。
     *
     * 将变量赋值给其它变量，就更容易理解了，要么将源变量位置中的值(注意是位置中的值，不是实际数据)移动到目标位置，要么将位置中的值拷贝到目标位置。如果是移动原变量位置中的值，在移动完成后将原变量设置为初始化未赋值的状态（不允许使用）。
     *
     * ## 位置状态的标记
     * 位置不仅仅只是一个简单的内存位置，它还有各种属性和状态，**这些属性和状态都是编译期间由编译器维护的，不会保留到运行期间。即内存分布还是上述分布，只是在编译期间编译器维护了更多的属性，但是不会带到运行期间**
     * 包括且可能不限于如下几种行为：
     * - 位置的类型（Rust中有变量类型、值类型）
     * - 位置保存的值是否正在被引用以及它是共享引用还是独占引用的标记。(borrow operators: The memory location is also placed into a borrowed state for the duration of the reference)。
     * - 根据位置的类型是否实现Copy Trait 来决定该位置的值是移动还是拷贝。
     *
     *
     * 总结：
     * - 其他语言的引用的意义就是指引用复杂数据（堆）的地址，但Rust中的引用是对一个变量的引用，引用是一种基本数据类型，用变量（栈空间）存储引用。
     * - 位置一旦初始化赋值，就会有一个永远不变的地址，直到销毁。换句话说，变量一旦初始化，无论它之后保存的数据发生了什么变化，它的地址都是固定不变的。也说明了，编译器在编译期间就已经安排好了所有位置的分配。这个位置指的是栈中的位置（内存空间）！
     * - 引用的值，即引用的指向目标是指指向位置，是指它指向的那个变量（指向内存），值就是所指向变量的地址。这一点和复杂数据的内存分布不是同一个东西，需要分开理解。
     * -
     * 这里有三个概念，1.引用 2.引用的值 3.引用是原始数据类型。以下的引用都是指名词，而不是借用这个动词概念的实现。
     * - 引用是一个原始数据类型，是一个似整数类型的数据类型，存储在栈上。
     * - 引用类型的变量的值是指针也就是位置的地址，整数类型的变量的值就是整数。
     *
     */

    println!("unit4");

    let a: String = String::from("Hello world");
    let b: &String = &String::from("Hello World");
    let c: &mut String = &mut String::from("Hello World");

    let mut d = c;

    let s = String::from("Hello");

    let mut s1 = s;
    s1.push_str(", world");

    // 可变变量与可变引用
    let mut s = String::from("hello, ");

    push_str(&mut s);

    fn push_str(s: &mut String) {
        s.push_str("world");
        println!("{}", s)
    }

    let a = 1;
    let b = 1;
    println!("{:p}, {:p}", &a, &b);

    let c = &a;
    let d = &c;

    println!("{}, {}", c, d);
}
