use std::fmt;

#[derive(Debug)]
pub enum RuntimeError {
    /// `<` at cell 0 or `>` past the last cell.
    MemoryPointerOutOfBounds { pc: usize, pointer: usize },

    /// Memory access outside the tape.
    MemoryAccessOutOfBounds { cell: usize, tape_size: usize },

    /// Program counter outside the program.
    InstructionFetch { pc: usize },

    /// I/O failure on the input or output stream.
    Io(std::io::Error),
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MemoryPointerOutOfBounds { pc, pointer } => {
                write!(
                    f,
                    "memory pointer {pointer} will go out out of bounds at pc={pc}"
                )
            }
            Self::MemoryAccessOutOfBounds { cell, tape_size } => {
                write!(
                    f,
                    "memory access at cell={cell} while tape size is {tape_size}"
                )
            }
            Self::InstructionFetch { pc } => {
                write!(f, "failed to fetch instruction at pc={pc}")
            }
            Self::Io(e) => write!(f, "io error: {e}"),
        }
    }
}

impl std::error::Error for RuntimeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

#[cfg(test)]
mod runtime_error_test {
    use std::{error::Error, io};

    use crate::error::RuntimeError;

    #[test]
    fn source_returns_some() {
        let io_error = io::Error::new(io::ErrorKind::ResourceBusy, "");
        let runtime_error = RuntimeError::Io(io::Error::new(io::ErrorKind::ResourceBusy, ""));

        assert_eq!(
            io_error.to_string(),
            runtime_error.source().unwrap().to_string()
        );
    }

    #[test]
    fn source_returns_none() {
        let runtime_error = RuntimeError::InstructionFetch { pc: 0 };

        assert!(runtime_error.source().is_none());
    }

    #[test]
    fn fmt_instruction_fetch_err() {
        let runtime_error = RuntimeError::InstructionFetch { pc: 0 };

        assert_eq!(
            "failed to fetch instruction at pc=0",
            format!("{}", runtime_error)
        );
    }

    #[test]
    fn fmt_memory_pointer_out_of_bounds_err() {
        let runtime_error = RuntimeError::MemoryPointerOutOfBounds { pc: 0, pointer: 0 };

        assert_eq!(
            "memory pointer 0 will go out out of bounds at pc=0",
            format!("{}", runtime_error)
        );
    }

    #[test]
    fn fmt_memory_access_out_of_bounds_err() {
        let runtime_error = RuntimeError::MemoryAccessOutOfBounds {
            cell: 0,
            tape_size: 30000,
        };

        assert_eq!(
            "memory access at cell=0 while tape size is 30000",
            format!("{}", runtime_error)
        );
    }

    #[test]
    fn fmt_io_err() {
        let runtime_error = RuntimeError::Io(io::Error::new(
            io::ErrorKind::ResourceBusy,
            "super serious error",
        ));

        assert_eq!(
            "io error: super serious error",
            format!("{}", runtime_error)
        );
    }
}
