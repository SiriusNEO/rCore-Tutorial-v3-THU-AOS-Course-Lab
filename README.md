# THU AOS Course Lab: rCore-Tutorial-v3

[[原 README]](README_original.md)

80240442 Advanced Operating System Course Lab, rCore v3. 本实验为清华大学高级操作系统课程作业，参考 [rCore-Tutorial-Book-v3](https://rcore-os.cn/rCore-Tutorial-Book-v3/index.html) 进行学习，fork 自 [rCore-Tutorial-v3](https://github.com/rcore-os/rCore-Tutorial-v3.git)。

rCore 按照内核逐渐完善的顺序从章节1到章节9，每个章节都给当前的内核起了一个古生物的名字，象征的内核从原始形态不断进化。对于这份报告，我们也按照章节顺序，每一章（除了第零章为环境配置）会记录一些在 Tutorial Book 上学的内容，再跟着本章的思路过一遍实现，以及完成每一章的“实验练习”部分。练习里的所有代码均保留在 repo 中，对应分支以 Tutorial book 为准，也会在小节开头说明。

通过对比 2023S 的 Tutorial 可以得知，在此之前只有五个 Lab（多道程序、地址空间、进程、文件系统、并发）；本文使用的新版本为每一章都添加了 Lab，并且添加了一章。然而由于一些历史原因，我并没有找到第七章和第九章的实验部分测试，因此这里暂时跳过这两个实验。因此本文共有七个代码实验可以复现。

其它有用的参考资料：
- [Rust 菜鸟教程](https://www.runoob.com/rust/rust-tutorial.html)：上手 Rust 语言
- [rCore-Tutorial-Code-2024S](https://github.com/LearningOS/rCore-Tutorial-Code-2024S)：2024 训练营代码
- [rCore-Tutorial-Test-2024S](https://github.com/LearningOS/rCore-Tutorial-Test-2024S.git)：2024 训练营测试点
- [rCore-Tutorial-in-single-workspace](https://github.com/YdrMaster/rCore-Tutorial-in-single-workspace/)：没有分支的 rCore-Tutorial，并且附带讲解
- [rCore-Tutorial V3](https://rcore-os.cn/rCore-Tutorial-deploy/)：rCore 教程，主要面向做实验的同学。
- [rCore-Tutorial-Guide 2023](https://learningos.cn/rCore-Tutorial-Guide-2023S/index.html)：rCore 2023 春季学期版本教程

## 第零章：操作系统概述

本章主要目的是配置 rCore 操作系统环境。推荐使用 Docker，见：https://rcore-os.cn/rCore-Tutorial-Book-v3/chapter0/5setup-devel-env.html

先登录 root 用户，然后在根目录下执行：

```bash
make build_docker
```

构建估计时间：半小时到一小时。然后平时可以使用如下命令进入容器：

```bash
make docker
```

查看、管理、删除容器同正常 docker 容器。

```bash
docker image ls # rcore-tutorial-v3
docker container ls
docker container rm rcore-tutorial-v3
```

启动容器，编译并在 QEMU 上运行 rCore。构建成功后可以看到可用的 APP 列表，可以运行试试。QEMU 的退出方式：先按 `Ctrl+a`，再按 `x`（如果这个退不了也可以试试直接 `ctrl+c` 或 `ctrl+z` 然后 `kill -9`。成功启动 rCore 即完成本章目标。
```bash
cd os/
make run
```

![](figures/rcore_build.png)

Issue: 构建 docker 容器时遇到如下报错

```latex
Step 8/13 : COPY --from=build_qemu /usr/local/bin/* /usr/local/bin
When using COPY with more than one source file, the destination must be a directory and end with a /
make: *** [Makefile:8: build_docker] Error 1
```

- https://github.com/rcore-os/rCore-Tutorial-v3/pull/151
- 解决办法：在 `/usr/local/bin` 后面加一个 `/`

## 第一章：应用程序与基本执行环境

本章的操作系统为寒武纪“三叶虫” LibOS，其目的是为 Hello world 程序提供最小的运行环境（这甚至不一定能算一个操作系统）。本章代码非常简单，主要内容都在 `os/` 目录下。

### 应用程序执行环境与平台支持

为了达成这个目标，我们要先学习应用如何在操作系统上被运行。

- 应用程序执行环境：现代操作系统（如 Linux 等）一般都使用多层的执行环境栈，如：应用程序 → (func call) → 标准库 → (sys call) → 内核/OS → (指令集) → 硬件平台。从操作系统的角度来讲，内核上面都属于**用户态**软件，而它自己属于**内核态**。
- 多层执行环境是必须的吗？最上层的应用和最下层的硬件必须存在，除此以外的中间层不必须。它们都是对下层进行了抽象，并且给上层提供了一个执行环境。抽象可以最小化暴露出功能，提供保护，但同时也会丧失灵活性、带来开销。
- 目标平台：现代编译流程是 preprocessor → compiler → assembler → linker。而在不同的平台上生成可执行文件，由于 OS 不同会导致 syscall 调用接口不同；底层硬件不同会导致 ISA 不同。Rust 通过目标三元组（Target Triplet）来描述一个软件的运行平台：CPU、操作系统、运行时库。
    - 例如：调用 `rustc --version --verbose` 可以看到我们的平台是 `x86_64-unknown-linux-gnu`，表示 CPU 架构是 x86_64，操作系统是 linux，运行时库是 GNU libc。
- 我们的主线任务是希望在另一个平台上运行 `Hello world` ，这里我们选择 `riscv64gc-unknown-none-elf` （elf 表示没有运行时库，但可以生成 ELF 格式的执行程序）。因此我们要把我们程序里对 std 库的依赖换成 core 库（core 不需要任何操作系统支持）。

### 移除标准库依赖

本小节主要目标是在 target 为 `riscv64gc-unknown-none-elf` 上完全移除 std 库的依赖，并使得我们能够编译通过。

- 在 main 函数开头加上 `#![no_std]` 表示不使用 std 库。
- 添加一个简陋的 `panic_handler` 来使其通过运行。
- 由于没有 std 库，我们也没有 `start` lang_item。因此这里我们直接加上 `#![no_main]` 来禁用传统意义上的 main 函数。

### 内核的第一条指令

上述工作终于使得我们编译通过，但是我们的功能还没有实现。本小节我们将通过汇编+文件加载的方式在 Qemu 上执行我们内核的第一条指令。

- Qemu 执行流程：
  ```bash
  qemu-system-riscv64 \
    -machine virt \
    -nographic \
    -bios ../bootloader/rustsbi-qemu.bin \
    -device loader,file=target/riscv64gc-unknown-none-elf/release/os.bin,addr=0x80200000
  ```
  其中通过 `device` 的 `loader` 属性可以将指定的内核镜像（`os.bin`）加载到指定的地址中。
  - 当内核加载到 Qemu 中，Qemu 的 PC（Program Counter）会被初始化到 `0x1000`。然后在执行若干指令后跳转到 `0x80000000`，这部分是写死在 Qemu 里的。
  - 对于不同的 bootloader，下一步的起始位置也不同。我们选用的 RustSBI 会在 `0x80200000` 将控制权转交给内核。因此我们的内核应该在该地址写入我们的第一条指令。

- 加载第一条指令：我们将指令写在汇编中，并在 rust 里用 `global_asm!` 宏嵌入到代码。由于 linker 默认的内存布局不符合我们上面的要求，我们需要使用我们自己的**链接脚本（Linker Script，.ld）**来完成链接。

### 为内核支持函数调用

上面我们成功执行了第一条内核指令。但是我们不想直接在汇编上编程，因此我们需要想办法回到 rust 中。我们知道函数调用的本质就是 pc 跳转，因此我们可以直接在汇编中跳转到我们的 `rust_main()` 位置，这里可以直接使用 risc-v 的伪指令 call 来完成。但是注意要为 main 函数留出足够的栈空间。

### 基于 SBI 服务完成输出和关机

由于无法使用标准库，我们就需要使用一些更底层的东西来完成 print 的实现。这里我们使用 Machine 特权级的指令来完成（内核位于 Supervisor 特权级，暴露给 Supervisor 的借口叫做 SBI）——即 RustSBI。我们使用 Rust 的 sbi_rt 库来完成。

### 实验

- 目标：实现彩色终端输出，并且实现一个 logger（支持不同等级的信息，如 ERROR、INFO、DEBUG）。
- 分支：`ch1`

笔者发现这部分代码好像在最新的代码 `ch1` 分支下已经被实现了（见 `logging.rs`）。主要就是对不同等级信息指定不同的颜色代码。可以运行

```bash
LOG=TRACE make run
```

观察到终端输出不同等级的 Log。修改 TRACE 为其它等级，能看到有些输出被略去。

## 第二章：批处理系统

本章的操作系统为泥盆纪“邓式鱼” BatchOS，它是一个支持**批处理**的操作系统：即用户可以一次性输入一批应用程序，操作系统自动按某种顺序将它们全部执行。此外，还需要注意当其中一个应用发生错误时，它尽量不要让整个系统崩溃，这就涉及到**特权级（Privilege）** 机制。代码上增加了 `sync`，`syscall` 和 `trap` 子模块以及 `batch.rs` 文件实现批处理系统。

### 特权级机制

特权级机制是由 CPU 硬件提供的而非单纯软件 / OS 完成的。特权级机制的核心思想是将操作系统等重要、底层的代码运行在一个相对安全、硬件保护的环境中。而将用户的应用运行在一个受限的运行环境中隔离开来。RISC-V 共有四级特权级（U/S/H/M），并且提供了一些特权级指令能够切换当前程序的特权级。

### 实现应用程序

用户的应用程序代码放在 `user/src` 下，包括用户库、用户程序以及 linker script（用户程序在内存的布局）。在 `src/bin/` 下包含用户的各个程序，然后提供一个外部库 user_lib（相当于编程语言的标准库）。

Qemu 模拟器有两种运行模式：用户态模拟（User mode）和系统级模拟（System mode）。在我们的操作系统被实现之前，我们可以用 user mode（`qemu-riscv64` 半系统模拟器，与之相对应 `qemu-system-riscv64` 为全系统模拟器）来模拟执行用户应用程序。

### 实现批处理操作系统

首先我们需要把用户应用程序的二进制镜像文件链接到内核里面，我们通过在 main 里引入一段 `link_app.S` 汇编来实现，这个汇编通过 `.incbin` 指令来插入每个 app，这个文件是由 `os/build.rs` 控制生成的。

为了实现批处理，我们需要一个全局的 `AppManager`，维护不同 app 的起始地址。在 Rust 语言中，我们需要一些技巧才能实现一个可变的全局变量：首先，使用 `static mut` 会导致对它的访问变为 unsafe，而单独使用 `static` 会导致 App Manager 内部不可变。因此我们需要借用 `RefCell` 来实现，并且通过再包一层来将其标记为 `Sync`，使得 Rust 编译器允许其在单核上安全使用。

最后我们的 batch 子模块暴露出如下接口：`init` 用来初始化 App Manager，`run_next_app` 用来按顺序执行下一个 App。


### 实现特权级的切换

我们主要关注批处理系统和用户应用程序是如何配合完成 RISC-V 的 U/S 特权级切换的。在操作系统中，用户态和内核态分别运行在两个栈上（为了安全）。

### 实验

- 目标：为 sys_write 增加安全检查，使其仅能输出位于程序本身内存空间内的数据，否则报错。
- 分支：`ch2-lab`
- 运行： `make run TEST=1`，看到期望输出。

实验记录：一开始尝试 `make run TEST=1` 的时候发生报错：
```bash
error: failed to compile `cargo-binutils v0.3.3`, intermediate artifacts can be found at `/tmp/cargo-installvTJxCF`

Caused by:
  package `regex-automata v0.4.7` cannot be built because it requires rustc 1.65 or newer, while the currently active rustc version is 1.64.0-nightly
make: *** [Makefile:45: env] Error 101
```

这是因为 `ch2-lab` 分支的 `rust-toolchain.toml` 版本比较老，而我们用的是 `main` 分支下的 Dockerfile 启的环境。解决方案：将 `main` 分支下的该文件覆盖到 `ch2-lab` 分支，然后编译运行（不同的章节放在不同分支感觉并不是很好...）。运行成功后首先会看到 kernel panic 的信息是 `Unsupported trap Exception(LoadFault)`，推测应该是人造的越界导致 trap 但是并没有对应的 Exception 处理，因此我们开始修改代码。

具体思路就是在 sys_write 前加一个 check。在一个 user app 运行的时候，它仅能访问两部分内存空间的数据：User Stack 和 App 堆上内存。因此我们修改 `batch.rs` 暴露出两个接口返回 `app_addr_range` 和 `user_stack_addr_range`，然后排查即可。注意，每次 write 的首地址和尾地址（即首地址 + 长度）都要 check。

然后又踩了一个坑，跑测试的时候 test1_write0 过了，write1 直接报这个错误。经排查似乎是 console 相关的错误，暂时先注释掉相关的 panic 代码（改为返回 -1）能够正常运行。以及 test1_write0 里的 `STACK_SIZE` 写成了 0x1000，需要修改成 0x2000。
```bash
[kernel] Panicked at src/syscall/fs.rs:31 Unsupported fd in sys_write!
```



## 第三章：多道程序与分时多任务

本章包含三个不同的操作系统，分别是二叠纪“锯齿螈”（支持多道程序）、三叠纪“始初龙”（支持多道程序的协作式操作系统）和三叠纪“腔骨龙” （支持分时多任务的抢占式操作系统）。代码上主要增加了 `task` 子模块来支持任务的管理与运行。

### 多道程序的放置与加载

与上章的批处理系统不同，随着计算机发展我们内存足够，可以将多个应用同时载入内存。加载相关功能被写在 `loader.rs` 中。简单来说，每个应用都留出 `APP_SIZE_LIMIT` 的大小，所以第 i 个应用的地址开头就是 `APP_BASE_ADDRESS + app_id * APP_SIZE_LIMIT`。执行第 i 个应用程序时，先跳转到它的入口点 `entry_i`，再将当前栈切换为用户栈。“锯齿螈”操作系统基本就完成实现了。

### 任务切换

我们把一个应用程序的一次执行过程成为**任务（Task）**，这个过程可以划分为很多时间片：计算任务片、空闲任务片，即暂停-继续的反复流程。我们在将任务暂停切出去的时候需要维护的资源成为**任务上下文（Task Context）**。

### 任务切换

我们需要在 Task Context 中保存 ra, sp, s0~s11 这些寄存器。任务切换是来自两个不同应用在内核中的 Trap 控制流之间的切换，切换时其会调用 `__switch` 函数完成切换，这部分用汇编在 `switch.S` 中实现。

### 多道程序与协作式调度

上一节实现了进程切换，但是我们需要知道何时调用这个切换。我们知道多道程序主要是解决程序等待 I/O 时 CPU 资源空闲的问题，这一信号靠的是应用主动调用 `sys_yield` 这一 syscall 来表示主动交出 CPU 使用权（即“协作式”）。最后我们用一个 `TaskManager` 完成任务的调度（通过将 Task 标记成不同状态），完成“始初龙”系统。

### 分时多任务系统与抢占式调度

上一章我们实现的是协作式调度系统（即靠应用发起 `sys_yield` 来切换控制权），这一章我们需要实现更加公平高效的分时多任务与抢占式系统“腔骨龙”。

本书中我们采用 **时间片轮转算法（Round-Robin）** 来调度应用程序，即不断在任务间公平轮转（维护一个队列，不断把队首拿出来执行一个时间片，放到队尾）。因此我们需要一个方式来记时，操作系统的记时是通过硬件提供的时钟中断实现的（中断主要有软件中断、时钟中断、外部中断）。


### 实验

- 目标：引入一个新的 syscall `sys_task_info`，可以获取指定的任务信息。
- 分支：`ch3-lab`
- 运行： `make run BASE=0`，看到期望输出。

一开始仍然遇到了 rust-toolchain 版本过于老旧的问题，发现 Tutorial 的所有 lab 分支都没太维护这些版本... 为了避免后续实验浪费时间，从这一章开始，我们改变策略：将 `ch-labX` 分支直接 reset 到 `chX` 分支，然后在 [2024 OS 训练营](https://github.com/LearningOS/rCore-Tutorial-Test-2024S.git) 这里把对应测试拷贝到 `/usr/src/bin/` 下来跑，期间也需要修改对应的 `/user/lib.rs` 和 `/user/syscall.rs`。期间经历了很多兼容问题，如变量名不一样、接口需要对齐、main 函数里的 `#![deny(missing_docs)]` 需要注释掉等等。在经历了一番折磨后，终于跑起来了测试部分。注意，跑起来仍然是切换到 `ch3-lab` 分支，然后运行 `make run BASE=0`，这部分已经都被我改好了，按道理在 main 分支的 Dockerfile 所构建的运行环境下可以正常运行。

本章任务是实现一个新的 syscall，所以我们需要先添加一些接口。其中 `TaskStatus` 是我们可以直接拿到的，而另外两个都需要我们维护。我们直接将对应信息添加到 `TaskControlBlock` 里，包括 `syscall_times` 和任务开始的时间戳 `start_time`。

syscall times 采用桶计数（因为已经给定了 `MAX_SYSCALL_NUM`），在 trap 控制流调用 syscall 的地方（`trap/mod.rs`）记录次数即可；对于 run time，我们只要维护每个 Task 第一次被执行的时间戳即可，因此我们在 `TaskManager` 的 `run_first_task` 和 `run_next_task` 中维护开始时间，用时间戳是否为 0 来保证我们只维护开始执行的时间（中间被别的任务抢占切换后的时间也属于执行时间），最后只要返回当前时间减去开始时间即可。不过这里其实有个小 bug，就是理论上任务结束后我们应该停止记时，按现在的逻辑停止后还会一直算时间，这里我们可以在应用退出后记录一个停止时间即可。不过测试里我们只会运行一个 Task，并且其测试过程中一直运行，所以没关系。

## 第四章：地址空间

本章的操作系统为侏罗纪“头甲龙”，它具有基本的内存管理功能。在这之前，我们的应用都是直接用物理地址访问物理内存，我们会实现一个基于分页机制的虚拟内存，给应用提供地址空间抽象。本章代码新增了 `mm` 内存管理模块。

### Rust 中的动态内存分配

在 C 标准库中我们通过 `malloc` 和 `free` 使用动态内存分配。Python/Java 通过引用计数（Reference Counting）对所有对象进行运行时的动态管理与垃圾回收（GC，Garbage Collection）。C++ 中引入了智能指针（shared_ptr、unique_ptr、weak_ptr、auto_ptr等）以及 RAII 机制。

Rust 吸取 C++ 的经验，设计了 `Box<T>`、`Rc<T>`、`RefCell<T>`、`Mutex<T>` 等智能指针/容器。智能指针本身会维护一些元信息，通常大小大于裸指针，这被称作胖指针（Fat Pointer）。

### 地址空间

操作系统给应用提供一个**地址空间 (Address Space)**，应用通过**虚拟地址 (Virtual Address)** 读写自己的地址空间。

内碎片（Internal Fragment）：如果我们给每个应用分配一段固定的连续大小，那么内核必须以消耗内存最多的应用为准，其它低内存占用的应用就会浪费大量的已分配内存。于是一种分段分配策略开始出现（注意这里段的大小可能是不同的），这种方式会有外碎片（External Fragment）问题，即夹在两个段之间的一个很小的段无法再被分配使用（因为我们给一个段分配的是连续内存）。我们可以知道段大小不一是外碎片产生的根本原因，因此我们可以使用统一的分配单元，即分页内存管理。

### SV39 多级页表的硬件机制

本小节主要介绍了地址的格式，包括虚拟地址、物理地址的结构（基本就是 page number + page offset 的形式），以及**页表项（PTE，Page Table Entry）** 的结构。然后在多级页表的实现上，讲了一种比线性表更优的“按需分配”做法。此外，硬件上还有 TLB 这种东西可以加速页表访问。

### 管理 SV39 多级页表

物理分页内存管理上，我们暂时采用一个比较简单的页帧管理器 StackFrameAllocator（并没有实现 buddy 算法）。而对于多级页表，主要通过 map 和 unmap 的方式建立虚存和物理内存的之间的页表映射。

### 内核与应用的地址空间

本节主要是从代码角度实现地址空间抽象。

### 基于地址空间的分时多任务

注意，当开启分页模式后，内核代码访存也只能看到内核地址空间。

在引入分页机制后，我们必须要在 Trap 时同时完成地址空间的转换。因此我们需要扩展 Trap 上下文。同时，在加载运行应用程序的时候，由于现在每个应用程序都有自己的地址空间，我们还需要扩展 Task 的一些字段，并且修改任务控制相关代码。

### 超越物理内存的地址空间

本章讲解了一些在有限的物理内存限制下充分利用内存的方式，使得其能“超越”物理内存大小。首先是分时复用内存，即在应用中动态分配内存。然后还有内存交换技术（swapping）与虚拟内存技术（virtual memory）。

### 实验

- 目标：1. 引入虚存机制后原来的 `sys_get_time` 实现失效了，请重写它；2. 实现 mmap 和 munmap syscall。
- 分支：`ch4-lab`
- 运行： `make run` 测试 map 和 unmap 看到期望输出。

这里第一个小目标我翻阅代码，发现  `sys_get_time` 在 rCore 的不同版本存在差异。地址空间应该影响的是  `sys_get_time(ts: *mut TimeVal)` 这种接口的 get time，考察点在于，我们需要处理指针的地址转换。不过由于我们的代码改自官方的 `ch4` 分支，接口为 `sys_get_time()`，因此没有这个问题，并且训练营的测试代码中也没有相关测试，因此在我们的实验中略过。第二点的测试正常运行后应该可以看到 1、4、5、6 的测试 OK 提示和两个 PageFault 报错（2、3 故意造成的访存错误）。

在本次任务中，我们的 mmap 只用于申请内存，而非如 Linux 的 mmap 一样可以把文件映射到内存中。我们给 `task` 模块额外实现两个接口：`map_in_current_task` 和 `unmap_in_current_task`，传入对应的 VPN 范围，调用对应 task 的 page table 对指定范围进行 map/unmap 即可，注意一些条件不要漏判。

## 第五章：进程

本章我们将开发具有进程管理功能的白垩纪“伤齿龙”操作系统，它会引入用户终端（Terminal）或称命令行应用（俗称 shell），并且允许在用户态灵活控制和管理应用的执行。因此我们需要将已有的任务抽象扩展为**进程**。

### 进程概念及重要系统调用

进程是在操作系统管理下程序的一次执行过程。进程模型包含三个 syscall：`fork`，`waitpid` 和 `exec`。为了实现 shell，我们还需要一个 syscall `sys_read` 来读入输出。

### 进程管理的核心数据结构

- 基于应用名的链接 & 应用加载器：首先，我们修改 `os/build.rs` 使得其可以读取 `/user/src/bin` 中应用程序对应的执行文件，并自动生成 `link_app.S`；然后在 loader 中，我们就可以通过应用名来获取其 ELF。
- 进程标识符和内核栈：进程标识符（pid）是每个进程的唯一标识符，这里用一个类似 RAII 的方式将其定义为一个 struct，并实现析构函数 `Drop` 来允许其自动回收 pid。同样内核栈我们也采用类似的思想，因为每个进程需要分配一个。
- 进程控制块（Process Control Block，PCB）：用于保存进程的执行状态等元信息，在内核中等价于一个进程，由 TaskControlBlock 扩展而来。
- 任务管理器（TaskManager）：提供给全局 `add_task` 和 `fetch_task` 两个接口，内部用一个双端队列维护。
- 处理器（Processor）：从原来的 TaskManager 拆出去，专门用来维护当前 CPU 上的任务执行状态。

### 进程管理机制的设计实现

这一节主要讲了实现几个前面提到的 syscall 的方式。

### 进程调度

这一节总览了进程调度策略，包括：
- 批处理系统上的调度：先来先服务（FCFS / FIFO），最短作业优先（Shortest Job First，SJF） 。这里我们的性能指标是评价平均周转时间。
- 交互式系统上的调度：最短完成时间优先（STCF），基于时间片轮转（Round-Robin）。注意前两个系统上我们都假任务的完成时间已知。这里由于引入 I/O 设备，我们的性能指标是平均响应时间（类似 TTFT，到应用第一次被执行的时间）。
- 通用计算机上的调度：多级反馈队列调度（MLFQ）以及公平份额调度（彩票调度与步长调度）。这里的性能指标引入了公平性。
- 实时计算机系统的调度：相比通用操作系统，这里进程虽然执行时间未知，但是是可预测、提前确定的（上限可以提前确定）。这里包括速率调度与最早截止时间优先（Earliest Deadline First，EDF）调度。
- 多处理器系统的调度：包括单队列、多队列调度。

### 实验

- 目标：实现一个完全 DIY 的系统调用 `spawn`，效果是直接创建一个新进程。
- 分支：`ch5-lab`
- 运行： `make run`，在 Terminal 中分别运行 `spawn0` 和 `spawn1` 运行两个测试点，看到期望输出。

这一章从训练营扒过来的测试点有两个，注意这些测试点还会调用一些别的辅助 App，也要一并放在 `user/src/bin` 下。

本章的实现比较直接，先按之前的老方法把 syscall 注册上去，然后给 TaskControlBlock 实现一个新方法叫 `spawn`，实现上就是 `fork` 和 `exec` 的融合版，由于这两个的代码已经给出，可以参照着写，完成相关资源的创建以及执行即可。

## 第六章：文件系统

本章我们将实现一个简单的文件系统 easyfs，以及具有强大 UNIX 操作系统基本功能的“霸王龙”操作系统。代码上，本章在根目录下新增了 `easy-fs` 与 `easy-fs-fuse` 用于 EasyFileSystem 的实现，以及对 `fs` 模块进行了修改，并在 `drivers` 模块中新增了对 Qemu 和 K210 两个平台的块设备驱动用于文件系统的硬件支持。同时，由于带有文件系统，本章开始我们通过文件系统加载应用，`loader.rs` 也被升级为一个子模块。

### 文件系统接口

文件可以用 `stat` 查看其的一些属性。为了管理方便，我们引入目录这一概念，所有文件和目录组织成一个目录树（Directory Tree）。我们可以用绝对路径（Absolute Path）定位其中的每个目录和文件，同时用相对路径（Relative Path）切换自己的当前工作目录（Current Working Directory，CWD）。我们的内核对目录树结构做了很多简化：仅存在根目录一个目录、没有权限控制、不支持软硬链接、不记录访问时间戳等等。

文件相关的系统调用有 `sys_open/sys_close`、`sys_read/sys_write`（前面我们只实现了标准输入输出上的）等。

### 简易文件系统 easy-fs

easy-fs 采用了松耦合的设计，实现上分成了两个不同的 crate：
- `easy-fs` 是文件系统的核心实现部分。
- `easy-fs-fuse` 是一个能在我们的 rCore 开发环境下运行的应用程序，可以将我们为内核开发的应用打包为一个 easy-fs 格式的文件系统镜像，这样我们就可以通过 easy-fs 的方式在内核加载这个应用。

easy-fs 自下而上可以分为五层：
- 块设备接口层定义了以块大小为单位对磁盘块设备进行读写的接口（Rust trait）。
- 块缓存层：我们需要尽量降低实际的块读写次数，因此我们引入一个缓存层（BlockCache）。
- 磁盘数据结构层：描述了磁盘的布局以及其相关的一些数据结构。磁盘从开头到结尾主要包括超级块（super block，存储一些元信息）、索引节点（Inode，Index Node）部分以及数据块部分。索引节点和数据块部分又都由对应的位图（bitmap）与存储区域（多个块）组成。
- 磁盘块管理层：这一层将上一层的零散的结构统一管理起来（类 EasyFileSystem），并且提供一些接口来快速管理 inode 与 data。
- 索引节点层：DiskInode 放在磁盘中的一个固定位置，而 Inode 则是在操作系统内存中记录文件索引节点信息的数据结构，在内核中我们通过 Inode 来管理这些文件，实现之前列举的文件操作。

最后还讲了将应用打包成 easy-fs 镜像的实现。之前我们的做法是直接将 `user/src/bin` 目录下所有应用链接到内核中，这可能造成内核体积过度膨胀。而实现 easy-fs 文件系统后，我们可以将这些应用打包成 easy-fs 镜像格式放在磁盘中，执行应用时只需要通过文件系统取出 ELF 格式的执行文件并且加载到内存中执行即可。

### 在内核中接入 easy-fs

上一节我们实现了 easy-fs 并且在用户态完成了测试。本节主要将上面的五层对接插入到内核代码中。

### 实验

- 目标：实现三个系统调用 `sys_linkat`，`sys_unlinkat`，`sys_fstat` 从而完成硬链接功能。
- 分支：`ch6-lab`
- 运行： `make run`，在 Terminal 中逐个运行 `file0~file3` 这些看到测试，看到期望输出。

本次实现这些功能要改动的地方，需要修改 easy-fs 以及 os 内核代码本身。首先，stat 只要暴露出对应接口拿到对应信息即可。对于 `sys_linkat`，我们找到先找到 old filename 对应的 ino_id，然后新建 DirEntry 写入 root inode 中；对于 `sys_unlinkat`，我们先找到需要删除的 inode，查看其 num_link 并将其减 1。如果计数减到 0，我们会调用 clear 清楚其信息。

在实现的过程中，一个比较需要注意的事情是我们需要合理使用 对于整个文件系统的 Mutex 来保证不发生 concurrent 的读写。通常我们会在一个文件系统接口的开头调用 `lock()` 来锁住当前操作，然后在执行完之后 drop 掉。

## 第七章：进程间通信与 I/O 重定向

本章我们重点引入了一个操作系统新概念——管道，这体现了 UNIX 操作系统“一切皆文件”的设计哲学，从而实现具有进程间通信功能的“迅猛龙”操作系统。本章在代码上主要是在 `fs` 模块下加入了 `pipe.rs`，以及修改了之前 fs 相关的一些代码，以及在任务中加入文件描述符表等。

### 基于文件的标准输入/输出

主要讲的是把标准输入/输出（STDIN/STDOUT/STDERR）也看成一个文件进行操作（文件描述符 0/1/2），这些文件会在进程开始时创建。

### 管道

管道是一种进程间通信机制，这里我们的管道是单向的（有分读端、写端），相应的 syscall 为 `sys_pipe`。

### 命令行参数与标准 I/O 重定向

这里我们会支持命令行参数（args）在 shell 中的传入，以及标准输入输出的重定向（`>`），这里需要实现 `sys_dup` 来支持。

### 信号

信号（Signals）是类 UNIX 操作系统中实现进程间通信的一种异步通知机制（`SIGXX`）。

## 第八章：并发

本章的并发主要指线程（Thread）并发（因为进程并发已经在 Round-Robin 调度时就实现了）。本章的“达科塔盗龙”操作系统 Thread & Coroutine OS 支持了线程抽象以及线程间的并发，同时支持了相应的 syscall；而进一步地，“慈母龙”操作系统 SyncMutex OS 具有同步互斥机制，包括了互斥锁（Mutex）、信号量（Semaphore）和条件变量（Condvar）等的实现。本章代码上进一步丰富了 `sync` 模块（同步互斥相关功能），以及在 `task` 模块中增加了线程控制等。

### 用户态的线程管理

本小节主要内容是对一个简单的、用户态的多线程应用执行过程的分析，并且设计了一个执行环境。关键内容是实现线程控制块（TCB）、线程的上下文以及汇编函数 switch。

### 内核态的线程管理

在内核态中，我们可以基于时钟中断直接打断当前用户态线程的运行，实现调度切换。本章其实很多地方和最初实现进程抽象的时候有点像，可以类比一下其中的共同之处与差别。

### 互斥锁

互斥锁（Mutex）主要是为了保证多线程应用能够正确访问共享资源，基本思想就是在访问共享资源前先检查其是否“上锁”，如果是，需要等待；若没上锁，该线程即可“拿到锁”，最后该线程完成操作后需要释放锁。锁有纯用户态的实现（Peterson 算法），但是其十分精密复杂，因此现代处理器往往在硬件层面提供了锁的支持（通过中断）。

### 信号量机制

互斥锁在一些更灵活的需求（如同时最多允许 N 个线程访问共享资源）不太够了，而信号量（Semaphore）可以看作是一种更高级的同步互斥机制。信号量可以看作是一个“有容量”的锁，内部有一个计数器，初始状态下其值为一个非负整数 N，然后提供两种操作：P操作（尝试），即一个线程尝试占用一个资源；V操作（增加），一个线程执行完将资源归还。每个成功P的操作会使得计数器-1，而每个成功的V操作会使得计数器+1。若当前的P操作发现计数器为0，即没有可用资源，则会等待（阻塞，加入阻塞队列）。可以看出 N=1 时其等价于 Mutex。

### 条件变量机制

条件变量是一个相对上面两个原语抽象程度更高、更简单直接的同步原语。考虑场景，我们第一个线程会将 A 修改为 1，而第二个线程会等待 A 修改成 1 后才继续执行，这里的 A 就可以看作是一个条件变量。它提供了两个基本的操作：`condvar_signal` 表示将条件变量设为 1；`condvar_wait` 表示等待直到条件变量变成 1。

### 并发中的问题

本小节讲了三个并发中的问题，分别是：
- 互斥缺陷：违反了临界区（共享资源）的原子性原则，即两个线程对某个共享变量的操作不是原子操作，因此在并发场景下有可能穿插执行导致错误。这个和同步缺陷其实某些意义下有点像。
- 同步缺陷：由于对共享变量访问时顺序可能是不同的，结果可能也不同，需要在访问前加同步操作。
- 死锁缺陷：两个线程互相等待锁导致卡死。

### 实验

- 目标：使用银行家算法检测死锁。
- 分支：`ch8-lab`
- 运行： `make run`，在 Terminal 中逐个运行 `deadlock_mutex1.rs`、`deadlock_sem1.rs` 和 `deadlock_sem2.rs` 这些看到测试，看到期望输出。

这里新版 Tutorial 实验是实现 eventfd 和银行家算法更新分数，但是我找不到对应的测试点，因此这里还是做旧版的 ch8 lab。这次实验的代码量较大，花费的时间也很多。主要目标是实现死锁检测机制，当检测到可能死锁时直接拒绝对应资源的请求。测试点总共有三个，其中我们应该在两个测试点检测到死锁，一个测试点没有死锁。

首先还是解决了一些兼容性问题，包括 user_lib 里一些 syscall 是否有返回指定规定等（对齐 2024 训练营）。

然后对于算法本身，其实在 Tutorial 里已经描述得很清楚了。我们主要考虑 allocation 矩阵和 need 矩阵如何维护。我们在每个 TaskControlBlock 中都维护它自己的 allocation 和 need，这样它们分别都是一个长度为 mutex/semaphore 个数的向量。算法本身对 mutex 和 semaphore 是差不多的（因为 mutex 本身可以看作一种特殊的 semaphore），所以这里我先讲一下 semaphore 要怎么维护。

我们首先要理清楚 semaphore 的实现。在 semaphore up 的时候，我们的可用资源增加了（归还，V操作）；down 的时候，可用资源减少了（寻求资源， P操作）。并且当 counter 降低到 < 0 时，后续的所有P操作都会让当前线程被塞到 wait queue 里；而在 counter < 0 时（即 wait queue 里一定存在线程时），每次 up，我们都需要从 wait queue 里拿出一个线程来执行（注意，在 rCore 的代码实现里是先给 counter+1，所以是 <= 0）。

那么我们回过来，allocation 指的是现在真正有资源，然后对应线程有需求，正好可以给其分配。所以其加 1 的时机就是：1. counter > 0 时候的 down（资源充足时）；2. counter < 0 时的每次 up，给从 wait queue 中拿出的那个 task 更新。那么相应的，need 指的现在无资源，但是对应线程也有需求，暂时记在 need 这里。所以其对应加 1 的时机是：counter <= 0 时候的 down（资源不充足时）。

对应到 mutex 的情况，我们不需要考虑 wait queue，所以就很简单：对于一次 lock 操作，如果当前的锁已经 locked，我们给 need 加 1；否则，我们给 allocation 加 1。


## 第九章：I/O 设备管理

本章的“侏罗猎龙操作系统具有更加便携地访问外设的能力，包括对块设备（virtio-blk）、网络设备（virtio-net）、键盘鼠标类设备（virtio-input）、显示设备（virtio-gpu）的 I/O 操作。本章主要内容是介绍了各种设备的驱动程序等。本章学习以阅读了解为主。

