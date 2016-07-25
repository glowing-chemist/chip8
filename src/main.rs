#![allow(dead_code)]

extern crate rand;
extern crate glium;

use std::fs::File;
use std::io::Read;
use std::env;
use rand::Rng;
use glium::glutin;


struct CPU { // main CPU STruct main memory and registers
    memory: [u8; 0x1000],  // all 4k of main memory
    stack: [u16; 0xF], // seperate stack
    screen_mem: [[u8; 64]; 32],// seperate screen memory
    stackp: u16, // pointer to top of the stack
    registers: [u16; 0xF], // all 16 registers
    i_pointer: u16,
    programc: u16, // prgram counter

    delay_timer: u32, // 60Hz timers possible in seperate struct / Thread??
    sound_timer: u32
}


impl CPU {
    fn new() -> CPU { // sets all the CPU constants to starting position i.e initialisation
        CPU{memory: [0; 0x1000],stack: [0; 0xF],screen_mem: [[0; 64]; 32], stackp: 0x0,
         registers: [0; 0xF],i_pointer: 0,programc: 0x200, delay_timer: 0, sound_timer: 0}
    }
}

fn maskx(opcode: u16) -> usize { // isolate specific value from opcode
    ((opcode & 0x0F00) >> 8) as usize // 0xABCD
}                                     //    ^ isolate this value

fn masky(opcode: u16) -> usize { // same as above
    ((opcode & 0x00F0) >> 4) as usize //0xABCD
}                                     //    ^ isolate this number

fn initialize() -> CPU {
    let mut arg = env::args();
    let file_name = arg.nth(1).unwrap();
    let file = File::open(file_name).unwrap();// read program binary from specified file
    let bytes = file.bytes();
    let mut my_cpu = CPU::new();
    for (i,x) in bytes.enumerate() { // reads program in to memory starting at adress 0x200
        my_cpu.memory[0x200 + i] = x.unwrap();
    }
    my_cpu
}

fn perf_op_code(my_cpu: &mut CPU, code: u16, my_window: &mut glutin::Window) { //match full opcode (u16) against chip8 instruction set
    match code & 0xF000 { // reference can be found at http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#8xy2
        0x0000 => {
            match code {
                0x00E0 => for i in 0xF00..0x1000 {
                    my_cpu.memory[i] = 0;
                },
                0x00EE => {
                    my_cpu.programc = my_cpu.stack[my_cpu.stackp as usize];
                    my_cpu.stackp -= 1;
                }
                0x0000 => {
                    //my_cpu.programc += 2;
                    return;
                }
                _ => panic!("incorrect opcode on code = {:X}",code)
            }
        }
        0x1000 => my_cpu.programc = code & 0x0FFF,
        0x2000 => {
                my_cpu.stackp += 1;
                my_cpu.stack[(my_cpu.stackp) as usize] = my_cpu.programc;
                my_cpu.programc = code & 0x0FFF;
        }
        0x3000 => {
            if my_cpu.registers[maskx(code)] as u16 == (code & 0x00FF) {
                    my_cpu.programc += 2;
            }
        }
        0x4000 => {
            if my_cpu.registers[maskx(code)] as u16 != (code & 0x00FF) {
                my_cpu.programc += 2;
            }
        }
        0x5000 => {
            if my_cpu.registers[maskx(code)] == my_cpu.registers[masky(code)] {
                my_cpu.programc += 2;
                }
        }
        0x6000 => my_cpu.registers[maskx(code)] = code & 0x00FF,
        
        0x7000 => { my_cpu.registers[maskx(code)]  
                        = my_cpu.registers[maskx(code)] as u16 + (code & 0x00FF)
        }
        0x8000 => {
            match code & 0x000F {
                0x0 => my_cpu.registers[maskx(code)] = my_cpu.registers[masky(code)],
                
                0x1 => {
                    my_cpu.registers[((code & 0x0F00) >>8) as usize] = 
                        my_cpu.registers[maskx(code)] | my_cpu.registers[masky(code)]
                }
                0x2 => {
                    my_cpu.registers[maskx(code)] = 
                        my_cpu.registers[maskx(code)] & my_cpu.registers[masky(code)]
                }
                0x3 => {
                    my_cpu.registers[maskx(code)] = 
                        my_cpu.registers[maskx(code)] ^ my_cpu.registers[masky(code)]
                }
                0x4 => {
                    my_cpu.registers[maskx(code)] = 
                        my_cpu.registers[maskx(code)] + my_cpu.registers[masky(code)]
                }
                0x5 => {
                    my_cpu.registers[maskx(code)] = 
                        my_cpu.registers[maskx(code)] - my_cpu.registers[masky(code)]
                }
                0x6 => {
                    if code & 0x000F == 0x1 {
                        my_cpu.registers[0xF as usize] = 1;
                    }else {
                        my_cpu.registers[0xF as usize] = 0;
                    }
                    my_cpu.registers[maskx(code)] /= 2;
                }
                0x7 => {
                    if my_cpu.registers[masky(code)] > my_cpu.registers[maskx(code)] {
                        my_cpu.registers[0xF as usize] = 1;                        
                    }else {
                        my_cpu.registers[0xF as usize] = 0;
                    }
                    my_cpu.registers[maskx(code)] = 
                        my_cpu.registers[masky(code)] - my_cpu.registers[maskx(code)]
                } 
                0xE => {
                    if code & 0xF000 == 0x1 {
                        my_cpu.registers[0xF as usize] = 1;
                    }else {
                        my_cpu.registers[0xF as usize] = 0;
                    }
                    my_cpu.registers[maskx(code)] *= 2;
                }
                _ => panic!("invalid opcode on 8 {} ",code),
            }
        }
        0x9000 => {
            if my_cpu.registers[maskx(code)] != my_cpu.registers[masky(code)] {
                my_cpu.programc += 2;
            }
        }
        0xA000 => my_cpu.i_pointer = code & 0x0FFF,
        
        0xB000 => my_cpu.programc = (code & 0x0FFF) + my_cpu.registers[0x0],
        
        0xC000 => {
            let randnum =  rand::thread_rng().gen::<u8>() as u16;
            my_cpu.registers[maskx(code)] = randnum & (code & 0x00FF );
        }
        0xD000 => {//64 x 32
            let xpos = my_cpu.registers[maskx(code)] as usize;
            let ypos = my_cpu.registers[masky(code)] as usize;
            let (sx, sy) = my_window.get_inner_size().unwrap();
            let sprite_size = code & 0x000F;
            for _ in my_cpu.i_pointer..(my_cpu.i_pointer + sprite_size) {
                my_cpu.screen_mem[xpos][ypos] = 1;
            }
        }// to do with drawing still needs work for graphics method
        0xE000 => {
            match code & 0x00FF {
                0x9E => { 
                    for events in my_window.poll_events() {
                        match events {
                            glutin::Event::KeyboardInput(_, _, Some(a)) => {
                                if (a as u16) == my_cpu.registers[maskx(code)] {
                                    my_cpu.programc += 2;
                                }
                            }
                            _ => ()
                        }
                    }
                }
                0xA1 => {
                    for events in my_window.poll_events() {
                        match events {
                            glutin::Event::KeyboardInput(_, _, Some(a)) => {
                                if (a as u16) != my_cpu.registers[maskx(code)] {
                                    my_cpu.programc += 2;
                                }
                            }
                            _ => ()
                        }
                    }
                }
                _ => panic!("invalid E code {}", code)
            }
        }
        0xF000 => {}// implement time at 60Hz and more input
        _ => panic!("invalled opcode"),
    }
}

fn main() {
    let mut my_cpu = initialize(); //initialise system
    let mut wind = glutin::Window::new().unwrap();
    'prog: while my_cpu.programc < 0x1000 {
        let opcode1 = my_cpu.memory[my_cpu.programc as usize] as u16;
        let opcode2 = my_cpu.memory[(my_cpu.programc + 1) as usize] as u16;
        let fullopcode = (opcode1 << 8) | opcode2;
        perf_op_code(&mut my_cpu, fullopcode, &mut wind);
        my_cpu.programc += 2;
        for events in wind.poll_events() {
            match events {
                glutin::Event::Closed => break 'prog,
                _ => ()
            }
        }
    }
}