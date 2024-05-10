use std::rc::Rc;

use ilearn::{run, Config};

fn main() {
    /*
     * ## Rc 与 Arc 引用计数，多个不可变引用的释放管理
     * Rust 所有权机制要求**一个值只能有一个所有者**，在大多数情况下这个设定都没有问题，但是考虑以下情况：
     * - 在图数据结构中，多条边可能会指向（拥有）同一个节点，该节点直到没有边指向它时，才应该被释放清理（多个不可变引用，怎么正确释放）
     * - 在多线程中，多个线程可能会持有同一个数据，但是受限于 Rust 的安全机制，无法同时获取该数据的可变引用（只能存在一个可变引用）
     *
     * 以上场景不是很常见，但一旦遇到就非常棘手，为了解决此类问题，Rust 在所有权机制外引入了额外的措施**引用计数 (`reference counting`)**来简化相应的实现（只是简化实现，并不违背所有权的要求）。
     *
     * 这一章先用 Rc 与 Arc 智能指针解决由于所有权机制导致维护共享不可变数据（不可变引用）方式非常复杂的问题，维护共享可变数据方式非常复杂的问题由下一章的 Cell 与 RefCell 解决。
     *
     * **为什么维护共享不可变数据方式非常复杂？**
     * 维护共享不可变数据方式非常复杂，这个复杂在于一个值存在**多个不可变引用**时，很难确定哪个不可变引用是最后一个使用者。
     * 而为了内存安全，rust 又需要找到最后一个使用者（不可变引用），以便在**最后一个使用者销毁时将此时已没有所有者的值一并销毁**。
     *
     * 除此寻找最后一个使用者外，将不可变引用传递给其他函数，生命周期的标注也是需要注意的。
     *
     * > 编译器采用三条规则来判断引用何时不需要明确的标注。第一条规则适用于输入生命周期，第二、三条规则适用于输出生命周期。
     * > 如果编译器检查完这三条规则后仍然存在没有计算出生命周期的引用，编译器将会停止并生成错误。
     * > 1. **每一个引用参数都有独自的生命周期。**
     * >      例如一个引用参数的函数就有一个生命周期标注: fn foo<'a>(x: &'a i32)，两个引用参数的有两个生命周期标注:fn foo<'a, 'b>(x: &'a i32, y: &'b i32), 依此类推。
     * > 2. **若只有一个输入生命周期(函数参数中只有一个引用类型)，那么该生命周期会被赋给所有的输出生命周期**。也就是所有返回值的生命周期都等于该输入生命周期。
     * >      例如函数 fn foo(x: &i32) -> &i32，x 参数的生命周期会被自动赋给返回值 &i32，因此该函数等同于 fn foo<'a>(x: &'a i32) -> &'a i32。
     * > 3. **若存在多个输入生命周期，且其中一个是 &self 或 &mut self，则 &self 的生命周期被赋给所有的输出生命周期**。
     * >      拥有 &self 形式的参数，说明该函数是一个 方法，该规则让方法的使用便利度大幅提升。
     *
     * ```rust
     * // 存在多个不可变引用时，很难确定哪个不可变引用是最后一个使用者
     * let s = String::from("Hello World");
     * let s1 = &s;
     * let s2 = &s;
     *
     * // 根据消除的三条规则，生命周期不能消除隐藏，需要显式标注
     * // 1. 为每个参数标注生命周期
     * // 2. 不符合第二条规则
     * // 3. 不符合第三条规则
     * // fn display(s1: &String, s2: &String) -> &String {
     * //     println!("{s1}, {s2}");
     * //     s1
     * // }
     * fn display<'a>(s1: &'a String, s2: &'a String) -> &'a String {
     *     println!("{s1}, {s2}");
     *     s1
     * }
     * display(&s1, &s2);
     * drop(s2);
     * drop(s); // 销毁s变量和对应的值
     * drop(s1); 报错，因为引用的值已经被销毁，现在 s1 指向的是一个空
     * ```
     *
     * 很明显，除了复杂的生命周期标注外，很难确定最后一个使用者，只有当最后一个使用者释放时才能将对应的值释放，如果使用的销毁顺序不正确就会导致内存错误。
     *
     * 针对此类共享不可变数据（不可变引用）问题，引用计数 (`reference counting`) 通过**记录一个数据被引用的次数**来确定该数据是否正在被使用来解决。
     * 当引用次数归零时，就代表该数据不再被使用，可以被清理释放。
     *
     * rust 内置的不可变引用的引用计数的实现有 `Rc（reference counting）` 和 `Arc（atomic reference counting）` 两种，Rc 适用于单线程，Arc 适用于多线程，在大部分情况下二者的功能都是相同的。
     *
     * ### Rc<T>
     *
     * 结构体（智能指针）Rc 的名称正是引用计数的英文缩写，当**需要在堆上分配一个对象供程序的多个部分使用，并且无法确定哪个部分是最后一个结束时（释放）**，
     * 就可以使用 Rc 成为**数据值的所有者（具有数据的所有权）**，实现多个不可变引用使用值，并且无需关心最后一个使用者释放问题。Rc 解决的是引用生命周期的复杂性。
     * 
     * TODO
     *
     *
     * 因此在不可变引用中这句话是错误的：~~通过引用计数的方式，允许一个数据资源在同一时刻拥有多个所有者~~，并不是指数据有多个所有者，而是指多个不可变引用。
     *
     *
     *
     * 一个值只能有一个所有者：
     * ```rust
     * // 一个值只能有一个所有者
     * let s1 = String::from("Hello World");
     * let s2 = s1; // s1 变量失去值的所有权
     * let s3 = s2; // s2 变量失去值的所有权
     * // let s3 = s1; 报错，s1 已经失去值的所有权，不能再使用没有值所有权的 s1 变量
     * ```
     *
     * 使用 Rc 实现一个值有多个所有者：
     * ```rust
     * let r1 = Rc::new(String::from("Hello World"));
     * let c = Rc::strong_count(&r1);
     * ```
     * 使用 `Rc::new` 创建一个 `Rc<String>` 智能指针并赋给变量 r1，该指针指向底层的字符串数据。
     *
     * 智能指针 `Rc<T>` 在创建时，会将引用计数加 1，引用计数可以通过关联函数 `Rc::strong_count` 获取，这里关联函数 `Rc::strong_count(&r1)` 返回 1。
     *
     * 在不定长类型 DST 和定长类型章节中提到过：
     * 不能简单的将变量与类型视为只是一块栈内存或一块堆内存数据，比如 Vec 类型，rust将其分成两部分数据：存储在堆中的实际类型数据与存储在栈上的管理信息数据。
     * 其中存储在栈上的管理信息数据是引用类型，包含实际类型数据的地址、元素的数量，分配的空间等信息，**rust 通过栈上的管理信息数据掌控实际类型数据的信息**。
     *
     * Rc 智能指针就是一种在堆栈均有存储数据的实现，它的原理是**利用结构体存储底层数据的地址和引用次数**，底层数据（实际类型数据）存放在堆上，结构体（胖指针，智能指针）存储在栈上作为管理信息数据管理实际类型数据。
     *
     * #### Rc::clone
     * 通过 `Rc::clone` 可以复制栈上的智能指针数据，让多个变量都能访问堆上的实际数据，
     * ```rust
     * let r1 = Rc::new(String::from("Hello World"));
     * let r2 = r1.clone();
     * let r3 = Rc::clone(&r1);
     * let r4 = r1; // 如果直接转移所有权，r1变量将不能再使用
     * // let r5 = r1; 报错，r1不具有值（智能指针）的所有权，所有权已经被转移到r4变量
     * ```
     *
     */

    // 一个值只能有一个所有者
    let s1 = String::from("Hello World");
    let s2 = s1; // s1 变量失去值的所有权
    let s3 = s2;
    // let s3 = s1; s1 已经失去值的所有权，不能再使用没有值所有权的 s1 变量

    // 存在多个不可变引用时，很难确定哪个不可变引用是最后一个使用者。
    let s = String::from("Hello World");
    let s1 = &s;
    let s2 = &s;

    // 根据消除的三条规则，生命周期不能消除隐藏，需要显式标注
    // 1. 为每个参数标注生命周期
    // 2. 不符合第二条规则
    // 3. 不符合第三条规则
    // fn display(s1: &String, s2: &String) -> &String {
    //     println!("{s1}, {s2}");
    //     s1
    // }
    fn display_str_ref<'a>(s1: &'a String, s2: &'a String) -> &'a String {
        println!("{s1}, {s2}");
        s1
    }
    display_str_ref(&s1, &s2);
    drop(s2);
    drop(s); // 销毁s变量和对应的值
             // drop(s1); 报错，因为引用的值已经被销毁，现在 s1 指向的是一个空

    // 引用计数
    let r1 = Rc::new(String::from("Hello World"));
    let r2 = r1.clone();
    let r3 = Rc::clone(&r1);
    // let s4 = s1; 如果直接转移所有权，s1变量将不能再使用

    fn display_rc(r1: Rc<String>, r2: Rc<String>) -> Rc<String> {
        println!("{r1}, {r2}");
        r1
    }
    // TODO 判定
    // 传递的指针都是被复制了一次，两者性能是等价的。直接传递让原有变量失去所有权
    // display_rc(r1, r2);
    display_rc(r1.clone(), r2.clone());
    println!("{:?}, {}, {}", r1, r2, r3);
    println!("{}", Rc::strong_count(&r1));
    drop(r1); // 释放s1，由于s1是Rc智能指针，有自定义的Drop::drop，因此底层的数据不会改变，只是引用计数减一
    println!("{}", Rc::strong_count(&r2));
}
