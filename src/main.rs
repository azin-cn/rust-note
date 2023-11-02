use core::panic;
use std::fmt::{Debug, Display};
use std::fs::File;
use std::io::ErrorKind;
use std::io::{self, Read};

fn main() {
    /*
     * ## 包和模块
     * 将大的代码文件拆分成包和模块，有利于实现代码抽象和复用。Rust 也提供了相应概念用于代码的组织管理：
     * - 项目(Packages)：一个 Cargo 提供的 feature，可以用来构建、测试和分享包
     * - 工作空间(WorkSpace)：对于大型项目，可以进一步将多个包联合在一起，组织成工作空间
     * - 包(Crate)：一个由多个模块组成的树形结构，可以作为三方库进行分发，也可以生成可执行文件进行运行
     * - 模块(Module)：可以一个文件多个模块，也可以一个文件一个模块，模块可以被认为是真实项目中的代码组织单元
     *
     * ### 1. 包 Crate
     * 对于 Rust 而言，包（Crate）是一个独立的可编译单元，它编译后会生成一个可执行文件或者一个库。
     * 一个包会将相关联的功能打包在一起，使得该功能可以很方便的在多个项目中分享。例如标准库中没有提供但是在三方库中提供的 rand 包，它提供了随机数生成的功能，我们只需要将该包通过 use rand; 引入到当前项目的作用域中，就可以在项目中使用 rand 的功能：rand::XXX。
     * 同一个包中不能有同名的类型，但是在不同包中就可以。例如，虽然 rand 包中，有一个 Rng 特征，可是我们依然可以在自己的项目中定义一个 Rng，前者通过 rand::Rng 访问，后者通过 Rng 访问，对于编译器而言，这两者的边界非常清晰，不会存在引用歧义。
     *
     *
     *
     */

    println!("项目、包、模块");
}
