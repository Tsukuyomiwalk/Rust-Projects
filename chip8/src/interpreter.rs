use std::ops::Add;

use crate::{
    data::{Address, Nibble, OpCode, RegisterIndex, Word},
    image::Image,
    platform::{Platform, Point, Sprite},
    Error, Key, Offset, Result,
};
////////////////////////////////////////////////////////////////////////////////

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

////////////////////////////////////////////////////////////////////////////////

pub struct Interpreter<P: Platform> {
    platform: P,
    bytes: [u8; Address::DOMAIN_SIZE],
    register: Address,
    registers: [Word; 16],
    stack: Vec<Address>,
    ptr: Address,
    last_key: Option<Key>,
}

impl<P: Platform> Interpreter<P> {
    pub fn new(image: impl Image, platform: P) -> Self {
        let mut tmp = [0u8; 4096];
        image.load_into_memory(&mut tmp);
        Self {
            platform,
            bytes: tmp,
            register: Address::default(),
            ptr: image.entry_point(),
            registers: [Word::default(); 16],
            stack: Vec::new(),
            last_key: None,
        }
    }

    pub fn platform(&self) -> &P {
        &self.platform
    }

    pub fn platform_mut(&mut self) -> &mut P {
        &mut self.platform
    }

    pub fn run_next_instruction(&mut self) -> Result<()> {
        let byte1 = self.bytes[self.ptr.as_usize()];
        let byte2 = self.bytes[self.ptr.as_usize() + 1];
        self.ptr = self.ptr.add(2);
        let op = Operation::try_from(OpCode::from_bytes(byte1, byte2));
        match op {
            Ok(op) => match op {
                Operation::ClearScreen => {
                    self.platform.clear_screen();
                    Ok(())
                }
                Operation::SetIndexRegister(index) => {
                    self.register = index;
                    Ok(())
                }
                Operation::SetRegister(index, word) => {
                    self.registers[index.as_usize()] = word;
                    Ok(())
                }
                Operation::Jump(address) => {
                    self.ptr = address;
                    Ok(())
                }

                Operation::Draw(index1, index2, address) => {
                    let x_point = self.registers[index1.as_usize()];
                    let y_point = self.registers[index2.as_usize()];
                    let xy = Point {
                        x: x_point,
                        y: y_point,
                    };
                    let data_to_draw = &self.bytes
                        [self.register.as_usize()..(self.register.as_usize() + address.as_usize())];
                    self.registers[15] =
                        Word::from(self.platform.draw_sprite(xy, Sprite::new(data_to_draw)));
                    Ok(())
                }
                Operation::AddValue(index, word) => {
                    self.registers[index.as_usize()] =
                        self.registers[index.as_usize()].wrapping_add(word);
                    Ok(())
                }
                Operation::SkipIfEqual(index, word) => {
                    if self.registers[index.as_usize()] == word {
                        self.ptr = self.ptr.add(2);
                    }
                    Ok(())
                }
                Operation::SkipIfNotEqual(index, word) => {
                    if self.registers[index.as_usize()] != word {
                        self.ptr = self.ptr.add(2);
                    }
                    Ok(())
                }
                Operation::SkipIfRegistersEqual(index1, index2) => {
                    if self.registers[index1.as_usize()] == self.registers[index2.as_usize()] {
                        self.ptr = self.ptr.add(2);
                    }
                    Ok(())
                }
                Operation::SkipIfRegistersNotEqual(index1, index2) => {
                    if self.registers[index1.as_usize()] != self.registers[index2.as_usize()] {
                        self.ptr = self.ptr.add(2);
                    }
                    Ok(())
                }
                Operation::Call(address) => {
                    self.stack.push(self.ptr);
                    self.ptr = address;
                    Ok(())
                }
                Operation::Return => {
                    self.ptr = self.stack.pop().unwrap();
                    Ok(())
                }
                Operation::SetToRegister(index1, index2) => {
                    self.registers[index1.as_usize()] = self.registers[index2.as_usize()];
                    Ok(())
                }
                Operation::Or(index1, index2) => {
                    self.registers[index1.as_usize()] |= self.registers[index2.as_usize()];
                    self.registers[15] = 0;
                    Ok(())
                }
                Operation::And(index1, index2) => {
                    self.registers[index1.as_usize()] &= self.registers[index2.as_usize()];
                    self.registers[15] = 0;

                    Ok(())
                }
                Operation::Xor(index1, index2) => {
                    self.registers[index1.as_usize()] ^= self.registers[index2.as_usize()];
                    self.registers[15] = 0;
                    Ok(())
                }
                Operation::AddRegister(index1, index2) => {
                    let val1 = self.registers[index1.as_usize()];
                    let val2 = self.registers[index2.as_usize()];

                    let add_result = val1.wrapping_add(val2);
                    self.registers[index1.as_usize()] = add_result;

                    self.registers[15] = if add_result >= val1 { 0 } else { 1 };
                    Ok(())
                }
                Operation::SubRegister(index1, index2) => {
                    let val1 = self.registers[index1.as_usize()];
                    let val2 = self.registers[index2.as_usize()];

                    let subtract_result = match val1.checked_sub(val2) {
                        Some(result) => {
                            self.registers[index1.as_usize()] = result;
                            1
                        }
                        None => {
                            self.registers[index1.as_usize()] = val1.wrapping_sub(val2);
                            0
                        }
                    };

                    self.registers[15] = subtract_result;
                    Ok(())
                }
                Operation::SubRegisterReversed(index1, index2) => {
                    let val1 = self.registers[index1.as_usize()];
                    let val2 = self.registers[index2.as_usize()];

                    let subtract_result = val2.wrapping_sub(val1);
                    self.registers[index1.as_usize()] = subtract_result;

                    self.registers[15] = if val2 >= val1 { 1 } else { 0 };
                    Ok(())
                }
                Operation::ShiftRight(index1, index2) => {
                    let tmp = self.registers[index1.as_usize()];
                    self.registers[index1.as_usize()] = self.registers[index2.as_usize()] >> 1;
                    self.registers[15] = tmp & 0x01;
                    Ok(())
                }

                Operation::ShiftLeft(index1, index2) => {
                    let tmp = self.registers[index1.as_usize()];
                    self.registers[index1.as_usize()] = self.registers[index2.as_usize()] << 1;
                    self.registers[15] = Word::from((tmp & 0x80) != 0);
                    Ok(())
                }

                Operation::ReadMemory(index) => {
                    let slice = &self.bytes[self.register.as_usize()..][..=index.as_usize()];
                    self.registers[0..=index.as_usize()].copy_from_slice(slice);
                    self.register = self.register.add(index.as_offset() + 1);
                    Ok(())
                }
                Operation::WriteMemory(index) => {
                    let start_index = self.register.as_usize();
                    let end_index = self.register.add(index.as_offset()).as_usize();
                    let slice = &self.registers[0..=index.as_usize()];
                    self.bytes[start_index..=end_index].copy_from_slice(slice);
                    self.register = self.register.add(index.as_offset() + 1);
                    Ok(())
                }

                Operation::ToDecimal(index) => {
                    let word = self.registers[index.as_usize()];
                    self.bytes[self.register.as_usize()] = word / 100;
                    self.bytes[self.register.as_usize() + 1] = word / 10 % 10;
                    self.bytes[self.register.as_usize() + 2] = word % 10;
                    Ok(())
                }

                Operation::IncrementIndexRegister(index) => {
                    self.register = self
                        .register
                        .add(self.registers[index.as_usize()] as Offset);
                    Ok(())
                }

                Operation::SetDelayTimer(index) => {
                    self.platform
                        .set_delay_timer(self.registers[index.as_usize()]);
                    Ok(())
                }

                Operation::SetToRandom(index, word) => {
                    self.registers[index.as_usize()] = word & self.platform.get_random_word();
                    Ok(())
                }
                Operation::GetDelayTimer(index) => {
                    self.registers[index.as_usize()] = self.platform.get_delay_timer();
                    Ok(())
                }

                Operation::JumpV0(address) => {
                    self.ptr = address.add(self.registers[0] as Offset);
                    Ok(())
                }
                Operation::SkipIfKeyDown(index) => {
                    if self
                        .platform
                        .is_key_down(Key::try_from(self.registers[index.as_usize()]).unwrap())
                    {
                        self.ptr = self.ptr.add(2);
                    }
                    Ok(())
                }
                Operation::SkipIfKeyUp(index) => {
                    if !self
                        .platform
                        .is_key_down(Key::try_from(self.registers[index.as_usize()]).unwrap())
                    {
                        self.ptr = self.ptr.add(2);
                    }
                    Ok(())
                }
                Operation::WaitForKey(index) => {
                    if let Some(last_key) = self
                        .last_key
                        .take()
                        .or_else(|| self.platform.consume_key_press())
                    {
                        if !self.platform.is_key_down(last_key) {
                            self.registers[index.as_usize()] = last_key.as_u8();
                            self.last_key = None;
                            self.ptr = self.ptr.add(2);
                        } else {
                            self.last_key = Some(last_key);
                        }
                    }
                    self.ptr = self.ptr.add(-2);
                    Ok(())
                }

                _ => Err(Error::UnknownOpCode(OpCode::from_bytes(byte1, byte2))),
            },
            _ => Err(Error::UnknownOpCode(OpCode::from_bytes(byte1, byte2))),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy)]
pub enum Operation {
    ClearScreen,
    Return,
    Jump(Address),
    Call(Address),
    SkipIfEqual(RegisterIndex, Word),
    SkipIfNotEqual(RegisterIndex, Word),
    SkipIfRegistersEqual(RegisterIndex, RegisterIndex),
    SetRegister(RegisterIndex, Word),
    AddValue(RegisterIndex, Word),
    SetToRegister(RegisterIndex, RegisterIndex),
    Or(RegisterIndex, RegisterIndex),
    And(RegisterIndex, RegisterIndex),
    Xor(RegisterIndex, RegisterIndex),
    AddRegister(RegisterIndex, RegisterIndex),
    SubRegister(RegisterIndex, RegisterIndex),
    ShiftRight(RegisterIndex, RegisterIndex),
    SubRegisterReversed(RegisterIndex, RegisterIndex),
    ShiftLeft(RegisterIndex, RegisterIndex),
    SkipIfRegistersNotEqual(RegisterIndex, RegisterIndex),
    SetIndexRegister(Address),
    JumpV0(Address),
    SetToRandom(RegisterIndex, Word),
    Draw(RegisterIndex, RegisterIndex, Nibble),
    SkipIfKeyDown(RegisterIndex),
    SkipIfKeyUp(RegisterIndex),
    GetDelayTimer(RegisterIndex),
    WaitForKey(RegisterIndex),
    SetDelayTimer(RegisterIndex),
    SetSoundTimer(RegisterIndex),
    IncrementIndexRegister(RegisterIndex),
    SetIndexRegisterToSprite(Nibble),
    ToDecimal(RegisterIndex),
    WriteMemory(Nibble),
    ReadMemory(Nibble),
}

impl TryFrom<OpCode> for Operation {
    type Error = ();

    fn try_from(code: OpCode) -> std::result::Result<Self, ()> {
        let op_code_as_nibbles = (
            code.extract_nibble(3).as_u8(),
            code.extract_nibble(2).as_u8(),
            code.extract_nibble(1).as_u8(),
            code.extract_nibble(0).as_u8(),
        );

        let op = match op_code_as_nibbles {
            (0x0, 0x0, 0xE, 0x0) => Operation::ClearScreen,
            (0x6, _, _, _) => Operation::SetRegister(code.extract_nibble(2), code.extract_word(0)),
            (0xA, _, _, _) => Operation::SetIndexRegister(code.extract_address()),
            (0x1, _, _, _) => Operation::Jump(code.extract_address()),
            (0xD, _, _, _) => Operation::Draw(
                code.extract_nibble(2),
                code.extract_nibble(1),
                code.extract_nibble(0),
            ),
            (0x7, _, _, _) => Operation::AddValue(code.extract_nibble(2), code.extract_word(0)),

            (0x3, _, _, _) => Operation::SkipIfEqual(code.extract_nibble(2), code.extract_word(0)),
            (0x4, _, _, _) => {
                Operation::SkipIfNotEqual(code.extract_nibble(2), code.extract_word(0))
            }
            (0x5, _, _, 0x0) => {
                Operation::SkipIfRegistersEqual(code.extract_nibble(2), code.extract_nibble(1))
            }
            (0x8, _, _, 0x0) => {
                Operation::SetToRegister(code.extract_nibble(2), code.extract_nibble(1))
            }
            (0x8, _, _, 0x1) => Operation::Or(code.extract_nibble(2), code.extract_nibble(1)),
            (0x8, _, _, 0x2) => Operation::And(code.extract_nibble(2), code.extract_nibble(1)),
            (0x8, _, _, 0x3) => Operation::Xor(code.extract_nibble(2), code.extract_nibble(1)),
            (0x8, _, _, 0x4) => {
                Operation::AddRegister(code.extract_nibble(2), code.extract_nibble(1))
            }
            (0x8, _, _, 0x5) => {
                Operation::SubRegister(code.extract_nibble(2), code.extract_nibble(1))
            }
            (0x8, _, _, 0x6) => {
                Operation::ShiftRight(code.extract_nibble(2), code.extract_nibble(1))
            }
            (0x8, _, _, 0x7) => {
                Operation::SubRegisterReversed(code.extract_nibble(2), code.extract_nibble(1))
            }
            (0x8, _, _, 0xE) => {
                Operation::ShiftLeft(code.extract_nibble(2), code.extract_nibble(1))
            }
            (0x9, _, _, 0x0) => {
                Operation::SkipIfRegistersNotEqual(code.extract_nibble(2), code.extract_nibble(1))
            }
            (0xF, _, 0x6, 0x5) => Operation::ReadMemory(code.extract_nibble(2)),
            (0xF, _, 0x5, 0x5) => Operation::WriteMemory(code.extract_nibble(2)),
            (0xF, _, 0x3, 0x3) => Operation::ToDecimal(code.extract_nibble(2)),
            (0xF, _, 0x1, 0xE) => Operation::IncrementIndexRegister(code.extract_nibble(2)),
            (0x2, _, _, _) => Operation::Call(code.extract_address()),
            (0x0, 0x0, 0xE, 0xE) => Operation::Return,
            (0xF, _, 0x1, 0x5) => Operation::SetDelayTimer(code.extract_nibble(2)),
            (0xF, _, 0x0, 0xA) => Operation::WaitForKey(code.extract_nibble(2)),

            (0xF, _, 0x0, 0x7) => Operation::GetDelayTimer(code.extract_nibble(2)),

            (0xE, _, 0xA, 0x1) => Operation::SkipIfKeyUp(code.extract_nibble(2)),
            (0xE, _, 0x9, 0xE) => Operation::SkipIfKeyDown(code.extract_nibble(2)),
            (0xC, _, _, _) => Operation::SetToRandom(code.extract_nibble(2), code.extract_word(0)),
            (0xB, _, _, _) => Operation::JumpV0(code.extract_address()),

            _ => {
                return Err(());
            }
        };
        Ok(op)
    }
}

////////////////////////////////////////////////////////////////////////////////
