use crate::ast::Node;

const BASE: u64 = 0x0040_0000;
const PAGE_SIZE: u64 = 0x1000;
const EHDR_SIZE: u64 = 64;
const PHDR_SIZE: u64 = 56;
const PROGRAM_HEADER_COUNT: u64 = 2;
const HEADERS_SIZE: u64 = EHDR_SIZE + PROGRAM_HEADER_COUNT * PHDR_SIZE;
const TAPE_SIZE: u64 = 30_000;

pub fn emit_linux_amd64_elf(program: &[Node]) -> Vec<u8> {
    let mut code_len = emit_program(program, 0).len() as u64;
    let mut result = None;

    for _ in 0..16 {
        let code_end = HEADERS_SIZE + code_len;
        let data_offset = align_up(code_end, PAGE_SIZE);
        let tape_addr = BASE + data_offset;

        let code = emit_program(program, tape_addr);

        if code.len() as u64 == code_len {
            result = Some((code, data_offset, tape_addr));
            break;
        }

        code_len = code.len() as u64;
    }

    let (code, data_offset, tape_addr) =
        result.expect("native code generation did not converge");

    build_elf(&code, data_offset, tape_addr)
}

fn align_up(value: u64, align: u64) -> u64 {
    ((value + align - 1) / align) * align
}

fn emit_program(program: &[Node], tape_addr: u64) -> Vec<u8> {
    let mut cg = CodeGen::new(tape_addr);
    cg.emit_init();
    cg.emit_nodes(program);
    cg.emit_exit();
    cg.code
}

struct CodeGen {
    code: Vec<u8>,
    tape_addr: u64,
}

impl CodeGen {
    fn new(tape_addr: u64) -> Self {
        Self {
            code: Vec::new(),
            tape_addr,
        }
    }

    fn emit_init(&mut self) {
        self.movabs_rbx(self.tape_addr);
    }

    fn emit_nodes(&mut self, nodes: &[Node]) {
        for node in nodes {
            self.emit_node(node);
        }
    }

    fn emit_node(&mut self, node: &Node) {
        match node {
            Node::Add(delta) => self.emit_add(*delta),
            Node::Move(delta) => self.emit_move(*delta),
            Node::Put => self.emit_put(),
            Node::Get => self.emit_get(),
            Node::Loop(body) => self.emit_loop(body),
            Node::Clear => self.emit_clear(),
            Node::Transfer { offset, multiplier } => {
                self.emit_transfer(*offset, *multiplier);
            }
        }
    }

    fn emit_add(&mut self, delta: i64) {
        let imm = delta as u8;
        if imm == 0 {
            return;
        }

        self.code.extend_from_slice(&[0x80, 0x03, imm]);
    }

    fn emit_move(&mut self, delta: i64) {
        if delta == 0 {
            return;
        }

        if delta >= i32::MIN as i64 && delta <= i32::MAX as i64 {
            self.code.extend_from_slice(&[0x48, 0x81, 0xC3]);
            self.emit_u32(delta as i32 as u32);
        } else {
            self.movabs_rax(delta as u64);
            self.code.extend_from_slice(&[0x48, 0x01, 0xC3]);
        }
    }

    fn emit_clear(&mut self) {
        self.code.extend_from_slice(&[0xC6, 0x03, 0x00]);
    }

    fn emit_put(&mut self) {
        self.mov_eax_imm32(1);
        self.mov_edi_imm32(1);
        self.mov_rsi_rbx();
        self.mov_edx_imm32(1);
        self.syscall();
    }

    fn emit_get(&mut self) {
        self.mov_eax_imm32(0);
        self.xor_edi();
        self.mov_rsi_rbx();
        self.mov_edx_imm32(1);
        self.syscall();

        self.code.extend_from_slice(&[0x48, 0x85, 0xC0]);
        self.code.extend_from_slice(&[0x7F, 0x03]);
        self.emit_clear();
    }

    fn emit_loop(&mut self, body: &[Node]) {
        let start = self.code.len();

        self.cmp_byte_rbx_zero();
        let end_patch = self.emit_je_rel32_placeholder();

        self.emit_nodes(body);

        self.cmp_byte_rbx_zero();
        self.emit_jne_to(start);

        let end = self.code.len();
        self.patch_rel32(end_patch, end);
    }

    fn emit_transfer(&mut self, offset: i64, multiplier: i64) {
        let multiplier_mod = multiplier as u8;

        if multiplier_mod == 0 {
            self.emit_clear();
            return;
        }

        self.code.extend_from_slice(&[0x0F, 0xB6, 0x03]);

        self.emit_clear();

        self.code.extend_from_slice(&[0x85, 0xC0]);
        let skip_patch = self.emit_jz_rel32_placeholder();

        self.code.extend_from_slice(&[0x48, 0x8D, 0x0B]);
        self.emit_add_rcx_imm64(offset);

        self.code.push(0x6B);
        self.code.push(0xC0);
        self.code.push(multiplier_mod as i8 as u8);

        self.code.extend_from_slice(&[0x00, 0x01]);

        let end = self.code.len();
        self.patch_rel32(skip_patch, end);
    }

    fn emit_exit(&mut self) {
        self.xor_edi();
        self.mov_eax_imm32(60);
        self.syscall();
    }

    fn cmp_byte_rbx_zero(&mut self) {
        self.code.extend_from_slice(&[0x80, 0x3B, 0x00]);
    }

    fn movabs_rbx(&mut self, imm: u64) {
        self.code.push(0x48);
        self.code.push(0xBB);
        self.emit_u64(imm);
    }

    fn movabs_rax(&mut self, imm: u64) {
        self.code.push(0x48);
        self.code.push(0xB8);
        self.emit_u64(imm);
    }

    fn movabs_rdx(&mut self, imm: u64) {
        self.code.push(0x48);
        self.code.push(0xBA);
        self.emit_u64(imm);
    }

    fn emit_add_rcx_imm64(&mut self, value: i64) {
        if value == 0 {
            return;
        }

        if value >= i32::MIN as i64 && value <= i32::MAX as i64 {
            self.code.extend_from_slice(&[0x48, 0x81, 0xC1]);
            self.emit_u32(value as i32 as u32);
        } else {
            self.movabs_rdx(value as u64);
            self.code.extend_from_slice(&[0x48, 0x01, 0xD1]);
        }
    }

    fn mov_eax_imm32(&mut self, imm: u32) {
        self.code.push(0xB8);
        self.emit_u32(imm);
    }

    fn mov_edi_imm32(&mut self, imm: u32) {
        self.code.push(0xBF);
        self.emit_u32(imm);
    }

    fn mov_edx_imm32(&mut self, imm: u32) {
        self.code.push(0xBA);
        self.emit_u32(imm);
    }

    fn xor_edi(&mut self) {
        self.code.extend_from_slice(&[0x31, 0xFF]);
    }

    fn mov_rsi_rbx(&mut self) {
        self.code.extend_from_slice(&[0x48, 0x89, 0xDE]);
    }

    fn syscall(&mut self) {
        self.code.extend_from_slice(&[0x0F, 0x05]);
    }

    fn emit_je_rel32_placeholder(&mut self) -> usize {
        self.emit_jcc_rel32_placeholder(0x84)
    }

    fn emit_jz_rel32_placeholder(&mut self) -> usize {
        self.emit_jcc_rel32_placeholder(0x84)
    }

    fn emit_jne_to(&mut self, target: usize) {
        let patch = self.emit_jcc_rel32_placeholder(0x85);
        self.patch_rel32(patch, target);
    }

    fn emit_jcc_rel32_placeholder(&mut self, opcode2: u8) -> usize {
        self.code.push(0x0F);
        self.code.push(opcode2);

        let patch_pos = self.code.len();
        self.code.extend_from_slice(&[0u8; 4]);

        patch_pos
    }

    fn patch_rel32(&mut self, patch_pos: usize, target: usize) {
        let rel = target as i64 - (patch_pos + 4) as i64;
        let rel32 = i32::try_from(rel).expect("jump distance too large");
        self.code[patch_pos..patch_pos + 4].copy_from_slice(&rel32.to_le_bytes());
    }

    fn emit_u32(&mut self, value: u32) {
        self.code.extend_from_slice(&value.to_le_bytes());
    }

    fn emit_u64(&mut self, value: u64) {
        self.code.extend_from_slice(&value.to_le_bytes());
    }
}

fn build_elf(code: &[u8], data_offset: u64, tape_addr: u64) -> Vec<u8> {
    let code_end = HEADERS_SIZE + code.len() as u64;
    assert!(code_end <= data_offset);

    let file_size = data_offset + TAPE_SIZE;
    let entry = BASE + HEADERS_SIZE;

    let mut out = Vec::with_capacity(file_size as usize);

    out.extend_from_slice(&[
        0x7f, b'E', b'L', b'F',
        2,
        1,
        1,
        0,
        0,
        0, 0, 0, 0, 0, 0, 0,
    ]);

    push_u16(&mut out, 2);
    push_u16(&mut out, 62);
    push_u32(&mut out, 1);
    push_u64(&mut out, entry);
    push_u64(&mut out, EHDR_SIZE);
    push_u64(&mut out, 0);
    push_u32(&mut out, 0);
    push_u16(&mut out, EHDR_SIZE as u16);
    push_u16(&mut out, PHDR_SIZE as u16);
    push_u16(&mut out, PROGRAM_HEADER_COUNT as u16);
    push_u16(&mut out, 0);
    push_u16(&mut out, 0);
    push_u16(&mut out, 0);

    assert_eq!(out.len(), EHDR_SIZE as usize);

    push_u32(&mut out, 1);
    push_u32(&mut out, 5);
    push_u64(&mut out, 0);
    push_u64(&mut out, BASE);
    push_u64(&mut out, BASE);
    push_u64(&mut out, code_end);
    push_u64(&mut out, data_offset);
    push_u64(&mut out, PAGE_SIZE);

    push_u32(&mut out, 1);
    push_u32(&mut out, 6);
    push_u64(&mut out, data_offset);
    push_u64(&mut out, tape_addr);
    push_u64(&mut out, tape_addr);
    push_u64(&mut out, TAPE_SIZE);
    push_u64(&mut out, TAPE_SIZE);
    push_u64(&mut out, PAGE_SIZE);

    assert_eq!(out.len(), HEADERS_SIZE as usize);

    out.extend_from_slice(code);

    out.resize(data_offset as usize, 0);

    out.resize(file_size as usize, 0);

    out
}

fn push_u16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn push_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn push_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Node;

    #[test]
    fn emits_elf_magic() {
        let elf = emit_linux_amd64_elf(&[]);
        assert_eq!(&elf[0..4], &[0x7f, b'E', b'L', b'F']);
    }

    #[test]
    fn code_size_is_stable_between_passes() {
        let program = vec![
            Node::Add(1),
            Node::Loop(vec![Node::Add(-1), Node::Move(1)]),
        ];

        let a = emit_program(&program, 0);
        let b = emit_program(&program, 0x400000);

        assert_eq!(a.len(), b.len());
    }
}
