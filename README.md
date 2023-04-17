## 1. 什么是CHIP-8？

chip-8本质上是一种编程语言，是在1970被发明用于游戏开发的解释型语言，因为chip-8虚拟机运行的时候是直接将每一句代码转为了机器语言去运行的，我们这次要实现的是就是一个chip-8的虚拟机。

chip-8使用两个字节的十六进制编码来进程编程的，每条编码会对应cpu指令集中的一个指令，虚拟机会翻译并对cpu进行操作。

## 2. 为什么要实现CHIP-8模拟器?

第一，因为最近在学习计算机组成原理和操作系统，所以作为一个练习，用来加深对CPU的工作流程的了解；

第二，它本身也是很有趣的，我想了解其中的原理；

最后，这个项目比较简单，我想通过它来更深入的了解模拟器的实现，以便于以后实现nes、GBA模拟器（相信很多人都有这样的想法）。

## 3. CHIP-8模拟器的组成

接下来我们来看一下CHIP-8的硬件组成

- 图形：chip-8的分辨率为64x32，颜色为单色
- 输入：chip-8使用16个键的十六进制键盘进行输入

- 内存：

- - 4K的内存（4096个位置，每个位置8bit）
  - 最开始时，chip-8的解释器本身占用了前512个字节，所以大部分程序都是从第512个位置开始（0x200），我们实现时，解释器本身运行在4K内存之外，所以一般用它来存字体数据
  - 最后面的256个字节（0xF00~0xFFF）用于刷新显示
  - 显示之前的96个字节（0xEA0~0xEFF）用于栈

- 寄存器：16个8bit的数据寄存器（用V0~VF表示）；1个地址寄存器（称为I），可以与涉及内存地址操作的操作码配合使用

- 栈：用于开始子程序时存储返回的地址

- 定时器：两个定时器

- - 延时计时器：一般用于处理游戏中的计时事件
  - 声音计时器：用于播放声音，如果其值不为0，就会发出声音

## 4. CHIP-8指令集

看完硬件后，我们要了解一下CHIP-8的指令集，很容易找到参考：

[mattmikolay/chip-8](https://link.zhihu.com/?target=https%3A//github.com/mattmikolay/chip-8/wiki/CHIP%E2%80%908-Instruction-Set)

[wiki-Opcode table](https://link.zhihu.com/?target=https%3A//en.wikipedia.org/wiki/CHIP-8%23Virtual_machine_description)

HIP-8有35个指令，都为两字节长，以大端方式存储。指令表的指令格式规定如下：

| 符号  | 含义             |
| ----- | ---------------- |
| NNN   | 12bit地址        |
| NN    | 8bit常量         |
| N     | 4bit常量         |
| X / Y | 4bit的寄存器标识 |
| PC    | Program Counter  |
| I     | 16bit地址寄存器  |

35个指令代码说明如下:

|      |          |                                                              |
| ---- | -------- | ------------------------------------------------------------ |
| 0NNN | 调用     | 执行地址NNN的子程序                                          |
| 00E0 | 显示     | 清空屏幕                                                     |
| 00EE | 流程控制 | 从子程序中返回                                               |
| 1NNN | 流程控制 | 跳转到地址NNN                                                |
| 2NNN | 流程控制 | 从NNN跳转到子程序                                            |
| 3XNN | 条件     | 如果VX == NN，则跳过下一条指令                               |
| 4XNN | 条件     | 如果VX != NN，则跳过下一条指令                               |
| 5XY0 | 条件     | 如果VX == VY，则跳过下一条指令                               |
| 6XNN | 常量     | VX = NN                                                      |
| 7XNN | 常量     | VX += NN                                                     |
| 8XY0 | 赋值     | VX = VY                                                      |
| 8XY1 | 位运算   | VX = VX \| VY                                                |
| 8XY2 | 位运算   | VX = VX & VY                                                 |
| 8XY3 | 位运算   | VX = VX ^ VY                                                 |
| 8XY4 | 数学     | VX += VY，有进位时VF = 1                                     |
| 8XY5 | 数学     | VX -= VY，有借位时VF = 0                                     |
| 8XY6 | 位运算   | VX >> 1                                                      |
| 8XY7 | 位运算   | VX = VY - VX                                                 |
| 8XYE | 位运算   | VX << 1                                                      |
| 9XY0 | 条件     | 如果VX != VY，则跳过下一条指令                               |
| ANNN | 内存地址 | 寄存器 I = NNN                                               |
| BNNN | 流程控制 | PC = V0 + NNN                                                |
| CXNN | 随机     | VX = rand() & NN，生成一个随机数并与NN按位与运算             |
| DXYN | 显示     | 在(VX, VY)位置绘制一个图像，其宽为8bit，高为N+1bit           |
| EX9E | 按键操作 | 等待按键，如果key() == VX，跳过下一条指令                    |
| EXA1 | 按键操作 | 等待按键，如果key() != VX，跳过下一条指令                    |
| FX07 | 计时器   | 将VX设置为延迟计时器                                         |
| FX15 | 计时器   | 将延迟计时器设置为VX                                         |
| FX18 | 声音     | 将声音计时器设置为VX                                         |
| FX0A | 按键     | 等待按键，然后VX = get_key()                                 |
| FX1E | 内存地址 | I += VX (VF不受影响)                                         |
| FX29 | 内存地址 | 将I设置为VX的字符地址，字符0-F由4X5字体表示                  |
| FX33 | BCD      | 将VX中值的BCD码存入I中的地址内，百位在I，十位在I+1，个位在I+2 |
| FX55 | 内存地址 | 将V0到VX的值存入I中地址为起始的内存空间                      |
| FX65 | 内存地址 | 将I中地址为起始的内容依次存入V0-VX                           |




















