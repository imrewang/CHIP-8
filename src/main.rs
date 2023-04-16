struct CPU {
    registers: [u8; 16],
    position_in_memory: usize, // program counter ("PC")
    memory: [u8; 4096],
    stack: [u16; 16], 
    stack_pointer: usize,
}

impl CPU {
    fn run(&mut self) {
        loop {          
            let op_byte1 = self.memory[self.position_in_memory] as u16;
            let op_byte2 = self.memory[self.position_in_memory + 1] as u16;
            let opcode: u16 = op_byte1 << 8 | op_byte2;

            let x  =       ((opcode & 0x0F00) >>  8) as u8; 
            let y  =       ((opcode & 0x00F0) >>  4) as u8;
            let kk =        (opcode & 0x00FF) as u8;
            let op_minor =  (opcode & 0x000F) as u8;
            let addr =      opcode & 0x0FFF;

            self.position_in_memory += 2;

            match opcode {
                0x0000 => { return; },
                0x00E0 => { /* CLEAR SCREEN */ },//清空屏幕
                0x00EE => { self.ret(); },//从子程序中返回
                0x1000..=0x1FFF => { self.jmp(addr); },//跳转到地址NNN
                0x2000..=0x2FFF => { self.call(addr); },//从NNN跳转到子程序
                0x3000..=0x3FFF => { self.se(x, kk); },//如果VX == NN，则跳过下一条指令
                0x4000..=0x4FFF => { self.sne(x, kk); },//如果VX != NN，则跳过下一条指令
                0x5000..=0x5FFF => { self.se(x, y); },//如果VX == VY，则跳过下一条指令
                0x6000..=0x6FFF => { self.ld(x, kk); },//VX = NN
                0x7000..=0x7FFF => { self.add(x, kk); },//VX += NN
                0x8000..=0x8FFF => {
                    match op_minor {
                        0 => { self.ld(x, self.registers[y as usize]) },//VX = VY
                        1 => { self.or_xy(x, y) },//VX = VX | VY
                        2 => { self.and_xy(x, y) },//VX = VX & VY
                        3 => { self.xor_xy(x, y) },//VX = VX ^ VY
                        4 => { self.add_xy(x, y); },//VX += VY，有进位时VF = 1
                        _ => { todo!("opcode: {:04x}", opcode); },
                    }
                },
                _ => todo!("opcode {:04x}", opcode),
            }
        }
    }

    ///(6xkk) LD 将值 `kk` 设置到寄存器 `vx`
    fn ld(&mut self, vx: u8, kk: u8) {//VX = NN
        self.registers[vx as usize] = kk; 
    }

    /// (7xkk) Add 将值 `kk` 设置到寄存器 `vx`
    fn add(&mut self, vx: u8, kk: u8) {//VX += NN
        self.registers[vx as usize] += kk; 
    }

    fn se(&mut self, vx: u8, kk: u8) {
        if vx == kk {//如果VX == NN，则跳过下一条指令
            self.position_in_memory += 2;
        }
    }

    
    fn sne(&mut self, vx: u8, kk: u8) {
        if vx != kk {//如果VX != NN，则跳过下一条指令
            self.position_in_memory += 2;
        }
    }

    ///(1nnn) 跳转到 `addr`
    fn jmp(&mut self, addr: u16) { //跳转到地址NNN
        self.position_in_memory = addr as usize;
    }

    /// (2nnn) 在 `addr` 调用子例程
    fn call(&mut self, addr: u16) {//从NNN跳转到子程序
        let sp = self.stack_pointer;
        let stack = &mut self.stack;
        
        if sp >= stack.len() {
            panic!("Stack overflow!")
        }

        stack[sp] = self.position_in_memory as u16;
        self.stack_pointer += 1;
        self.position_in_memory = addr as usize;
    }

    /// (00ee) RET 从当前子程序返回
    fn ret(&mut self) {//从子程序中返回
        if self.stack_pointer == 0 {
            panic!("Stack underflow");
        }

        self.stack_pointer -= 1;
        self.position_in_memory = self.stack[self.stack_pointer] as usize;
    }

    fn add_xy(&mut self, x: u8, y: u8) {//VX += VY，有进位时VF = 1
        let arg1 = self.registers[x as usize];
        let arg2 = self.registers[y as usize];
  
        let (val, overflow_detected) = arg1.overflowing_add(arg2);
        self.registers[x as usize] = val;
  
        if overflow_detected {
          self.registers[0xF] = 1;
        } else {
          self.registers[0xF] = 0;
        }
    }

    fn and_xy(&mut self, x: u8, y: u8) {//VX = VX & VY
        let x_ = self.registers[x as usize];
        let y_ = self.registers[y as usize];

        self.registers[x as usize] = x_ & y_;
    }

    fn or_xy(&mut self, x: u8, y: u8) {//VX = VX | VY
        let x_ = self.registers[x as usize];
        let y_ = self.registers[y as usize];

        self.registers[x as usize] = x_ | y_;
    }

    fn xor_xy(&mut self, x: u8, y: u8) {//VX = VX ^ VY
        let x_ = self.registers[x as usize];
        let y_ = self.registers[y as usize];

        self.registers[x as usize] = x_ ^ y_;
    }
}

fn main() {
    let mut cpu = CPU {
        registers: [0; 16],
        memory: [0; 4096],
        position_in_memory: 0,
        stack: [0; 16],
        stack_pointer: 0,
    };

    cpu.registers[0] = 5;
    cpu.registers[1] = 10;

    // 
    cpu.memory[0x000] = 0x21; cpu.memory[0x001] = 0x00; 
    cpu.memory[0x002] = 0x21; cpu.memory[0x003] = 0x00;

    cpu.memory[0x100] = 0x80; cpu.memory[0x101] = 0x14; 
    cpu.memory[0x102] = 0x80; cpu.memory[0x103] = 0x14;
    cpu.memory[0x104] = 0x00; cpu.memory[0x105] = 0xEE;

    cpu.run();

    assert_eq!(cpu.registers[0], 45);

    println!("5 + (10 * 2) + (10 * 2) = {}", cpu.registers[0]);
}