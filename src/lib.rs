#![forbid(unsafe_code)]

pub mod ast;
pub mod error;
pub mod lexer;
pub mod native;
pub mod optimizer;
pub mod parser;

pub use error::{CompileError, Result};

pub fn compile_to_native_linux_amd64(source: &str, optimize: bool) -> Result<Vec<u8>> {
    let tokens = lexer::lex(source);
    let program = parser::parse(&tokens)?;

    let program = if optimize {
        optimizer::optimize(program)
    } else {
        program
    };

    Ok(native::emit_linux_amd64_elf(&program))
}
