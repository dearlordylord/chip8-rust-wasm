use std::fmt;

use ux::u12;

use crate::cpu_instructions::*;

pub type DecodeResult = std::result::Result<Box<Instruction>, DecodeError>;
#[derive(Debug, Clone)]
pub struct DecodeError(u16);

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Can't decode operation {}", self.0)
    }
}

impl std::error::Error for DecodeError {

}

pub fn decode(opcode: u16) -> DecodeResult {
    let x = X(((opcode & 0x0F00) >> 8).into());
    let y = Y(((opcode & 0x00F0) >> 4).into());
    let kk = KK((opcode & 0x00FF).to_be_bytes()[1]);
    let nnn = NNN(u12::new(opcode & 0x0FFF));
    let n = N(opcode & 0x000F);
    match opcode & 0xF000 {
        0x0000 => match opcode {
            0x00E0 => DecodeResult::Ok(cls()),
            0x00EE => DecodeResult::Ok(ret()),
            _ => DecodeResult::Ok(sys()),
        },
        0x1000 => DecodeResult::Ok(jp_nnn(nnn)),
        0x2000 => DecodeResult::Ok(call_nnn(nnn)),
        0x3000 => DecodeResult::Ok(se_vx_kk(x, kk)),
        0x4000 => DecodeResult::Ok(sne_vx_kk(x, kk)),
        0x5000 => DecodeResult::Ok(se_vx_vy(x, y)),
        0x6000 => DecodeResult::Ok(ld_vx_kk(x, kk)),
        0x7000 => DecodeResult::Ok(add_vx_kk(x, kk)),
        0x8000 => match opcode & 0x000F {
            0x0000 => DecodeResult::Ok(ld_vx_vy(x, y)),
            0x0001 => DecodeResult::Ok(or_vx_vy(x, y)),
            0x0002 => DecodeResult::Ok(and_vx_vy(x, y)),
            0x0003 => DecodeResult::Ok(xor_vx_vy(x, y)),
            0x0004 => DecodeResult::Ok(add_vx_vy(x, y)),
            0x0005 => DecodeResult::Ok(sub_vx_vy(x, y)),
            0x0006 => DecodeResult::Ok(shr_vx_vy(x, y)),
            0x0007 => DecodeResult::Ok(subn_vx_vy(x, y)),
            0x000E => DecodeResult::Ok(shl_vx_vy(x, y)),
            _ => DecodeResult::Err(DecodeError(opcode)),
        },
        0x9000 => DecodeResult::Ok(sne_vx_vy(x, y)),
        0xA000 => DecodeResult::Ok(ld_i_nnn(nnn)),
        0xB000 => DecodeResult::Ok(jp_v0_nnn(nnn)),
        0xC000 => DecodeResult::Ok(rnd_vx_kk(x, kk)),
        0xD000 => DecodeResult::Ok(drw_vx_vy_n(x, y, n)),
        0xE000 => match opcode & 0x00FF {
            0x009E => DecodeResult::Ok(skp_vx(x)),
            0x00A1 => DecodeResult::Ok(sknp_vx(x)),
            _ => DecodeResult::Err(DecodeError(opcode)),
        },
        0xF000 => match opcode & 0x00FF {
            0x0007 => DecodeResult::Ok(ld_vx_dt(x)),
            0x000A => DecodeResult::Ok(ld_vx_k(x)),
            0x0015 => DecodeResult::Ok(ld_dt_vx(x)),
            0x0018 => DecodeResult::Ok(ld_st_vx(x)),
            0x001E => DecodeResult::Ok(add_i_vx(x)),
            0x0029 => DecodeResult::Ok(ld_f_vx(x)),
            0x0033 => DecodeResult::Ok(ld_b_vx(x)),
            0x0055 => DecodeResult::Ok(ld_i_vx(x)),
            0x0065 => DecodeResult::Ok(ld_vx_i(x)),
            _ => DecodeResult::Err(DecodeError(opcode)),
        },
        _ => DecodeResult::Err(DecodeError(opcode))
    }
}