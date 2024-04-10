use ilearn::{run, Config};
use std::{array::IntoIter, env, error::Error, fmt::Display, fs, process};

fn main() {
    /*
     * ## 迭代器
     *
     * rust中，**迭代器的方法**可以细分为消费者适配器（consuming adaptors）和迭代器适配器（iterator adaptors），两者的区别在于是否消费迭代器，即是否调用迭代器的 next 方法。
     *
     * ### 消费者适配器
     * 消费者适配器（consuming adaptors）是迭代器上的方法，它会消费掉迭代器和迭代器中的元素，然后返回其类型的值，因此被称为消费。
     * 这些消费者（方法）都有一个共同的特点：在它们的定义中，都依赖 next 方法来消费元素。这也是为什么迭代器要实现 Iterator 特征时必须要实现 next 方法的原因。
     *
     * 只要迭代器上的某个方法 A 在其内部调用了 next 方法，那么 A 就可以被称为消费性适配器。这是因为 next 方法会消耗掉迭代器上的元素，所以方法 A 的调用也会消耗掉迭代器上的元素。
     *
     * 其中一个例子是 sum 方法，它会拿走迭代器的所有权，然后通过不断调用 next 方法对里面的元素进行求和：
     * ```rust
     * let v = vec![1, 2, 3];
     * let iter = v.iter();
     * let total: i32 = iter.sum(); // 消费者适配器需要标注数据类型
     * // println!("{:#?}", iter); 不能再访问iter，因为sum消费了迭代器和迭代器中的元素
     * println!("{total}");
     * ```
     *
     * 可以看到sum函数的定义 `fn sum(self) {}`，拿走了迭代器的所有权：
     * ```rust
     * fn sum<S>(self) -> S
     * where
     *     Self: Sized,
     *     S: Sum<Self::Item>,
     * {
     *     Sum::sum(self)
     * }
     * ```
     *
     * ### 迭代器适配器
     * 迭代器适配器（iterator adapters）即迭代器方法会返回一个新的迭代器，这是实现链式方法调用的关键：`v.iter().map().filter()...`。
     * 与消费者适配器不同，迭代器适配器是惰性的，意味着需要一个消费者适配器来收尾，最终将迭代器转换成一个具体的值：
     * ```rust
     * let v: Vec<i32> = vec![1, 2, 3];
     * // v.iter().map(|x| x + 1); 仅有迭代器适配器是不行的，需要消费者适配器收尾
     * let newV: Vec<_> = v.iter().map(|x| x + 1).collect(); // 正常
     * ```
     * 
     * > 为什么要区分消费者适配器和迭代器适配器两种方法呢？
     * > 
     * > Rust语言在设计上非常注重内存安全和效率，这种设计哲学体现在它对迭代器模式的处理上。Rust区分消费性适配器（consuming adaptors）和迭代器适配器（iterator adaptors）主要是为了提供更细粒度的控制以及更明确的语义。
     * > 
     * > 消费性适配器（Consuming Adaptors）
     * > 
     * > 消费性适配器是那些会消耗迭代器的方法，它们会遍历迭代器并返回一个最终的结果。这意味着一旦调用了消费性适配器，原来的迭代器就不能再使用了。在Rust中，collect()就是一个消费性适配器的例子，它可以将迭代器中的元素收集到一个集合类型中，比如Vec、HashMap等。
     * > 
     * > 迭代器适配器（Iterator Adaptors）
     * > 
     * > 迭代器适配器则是对迭代器进行转换，但不会立即进行任何遍历操作。它们返回的是一个新的迭代器，这个新迭代器会在每次遍历时应用某种操作。在Rust中，map()就是一个迭代器适配器的例子，它会创建一个新的迭代器，这个迭代器会在每次访问时应用一个函数到原迭代器的每个元素上。
     * > 
     * > 1. 性能优化：Rust的迭代器设计允许编译器在编译时进行更多的优化，比如通过迭代器链的懒惰求值来减少中间集合的创建，这可以显著提高程序的性能。
     * > 2. 内存管理：Rust通过所有权系统来保证内存安全，区分消费性适配器和迭代器适配器有助于明确所有权和借用的规则，避免悬垂指针和数据竞争等问题。
     * > 3. 明确的语义：在Rust中，当你使用collect()时，你明确地表达了你想要从迭代器中消费所有元素并生成一个集合。这种明确性有助于代码的可读性和维护性。
     */

    let v = vec![1, 2, 3];
    let mut iter = v.into_iter();
    let total: i32 = iter.sum();
    // println!("{:#?}", iter); 不能再访问iter，因为sum消费了迭代器和迭代器中的元素
    println!("{total}");

    let v = vec![1, 2, 3];
    let mut iter = v.into_iter();

    let total: i32 = iter.map(|x| x + 1).sum();
    println!("{total}");
}
