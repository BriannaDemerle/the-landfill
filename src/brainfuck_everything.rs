//! A program that reads any file as a brainfuck program by interpreting every
//! three bits as an instruction.
//! 
//! `Coded on 4/17/26`
use std::io::{self, Read};
use std::path::Path;

/// An iterator that reads bits from an iterator returning `u8`s.
struct BitReader<I> {
    current_bit_index: usize,
    current_byte: Option<u8>,
    inner: I,
}

impl<I: Iterator<Item = u8>> BitReader<I> {
    /// Creates a new `BitReader` from an iterator yielding `u8`s.
    pub fn new(mut byte_iterator: I) -> Self {
        Self {
            current_bit_index: 0,
            current_byte: byte_iterator.next(),
            inner: byte_iterator,
        }
    }

    /// Tries to return a `[bool; N]` of the next `N` bits if possible,
    /// returning `None` if there aren't enough bits. If `None` is returned, 
    /// the iterator is fully consumed.
    pub fn get_bits<const N: usize>(
        &mut self,
    ) -> Option<[<Self as Iterator>::Item; N]> {
        self.take(N).collect::<Vec<_>>().as_array::<N>().copied()
    }
}

impl<I: Iterator<Item = u8>> Iterator for BitReader<I> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        self.current_byte.map(|byte| {
            let bit = ((byte >> self.current_bit_index) & 1) == 1;

            self.current_bit_index += 1;
            if self.current_bit_index == 8 {
                self.current_bit_index = 0;
                self.current_byte = self.inner.next();
            }

            bit
        })
    }
}

/// An instruction in brainfuck.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BFInstruction {
    IncrementHead,
    DecrementHead,
    IncrementByte,
    DecrementByte,
    OutputByte,
    InputByte,
    BranchForwards,
    BranchBackwards,
}

impl std::fmt::Display for BFInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            Self::IncrementHead => '>',
            Self::DecrementHead => '<',
            Self::IncrementByte => '+',
            Self::DecrementByte => '-',
            Self::OutputByte => '.',
            Self::InputByte => ',',
            Self::BranchForwards => '[',
            Self::BranchBackwards => ']',
        };

        write!(f, "{}", c)
    }
}

impl BFInstruction {
    /// Gets a brainfuck instruction from an array of 3 bits.
    /// Every array of 3 bits is guaranteed to return an instruction. 
    pub fn from_bit_array(bits: [bool; 3]) -> Self {
        let index = bits.map(|b| b as u8);
        let instruction_num = (index[0] << 2) + (index[1] << 1) + index[2];

        // SAFETY: instruction number is guaranteed to be in 0..8,
        // which correspond with BFInstructions
        unsafe { std::mem::transmute::<u8, Self>(instruction_num) }
    }

    /// Returns the 3-bit representation of the instruction.
    /// `BFInstruction::to_bits(from_bit_array(arr)) == arr` will always hold.
    pub fn to_bits(&self) -> [bool; 3] {
        let num = *self as u8;

        [
            (num >> 2) & 1 == 1,
            (num >> 1) & 1 == 1,
            num & 1 == 1
        ]
    }
}

/// A brainfuck program with `N` cells on the tape.
/// The read-write head does not loop around the edges.
pub struct BFProgram<const N: usize> {
    values: [u8; N],
    head: usize,

    program: Vec<BFInstruction>,
    instruction_pointer: usize,
}

impl<const N: usize> BFProgram<N> {
    /// Loads a program from a file,
    /// where every 3 bits is the corresponding instruction.
    /// If the number of bits is not a multiple of 3,
    /// the remaining bits are ignored.
    pub fn load_from_binary<P: AsRef<Path>>(filename: P) -> io::Result<Self> {
        let file = std::fs::File::open(filename)?;

        // `.bytes()` yields `Result<u8, _>`, flatten ignores `Err`s
        let instructions = BitReader::new(file.bytes().flatten())
            .array_chunks::<3>()
            .map(BFInstruction::from_bit_array)
            .collect::<Vec<_>>();

        for inst in &instructions {
            print!("{inst}");
        }
        println!();

        Ok(Self::new_from_instructions(instructions))
    }

    /// Creates a new program from instructions. 
    fn new_from_instructions(program: Vec<BFInstruction>) -> Self {
        Self {
            values: [0; N],
            head: 0,
            program,
            instruction_pointer: 0,
        }
    }

    /// Gets the byte at the read-write head.
    fn get_current_value(&self) -> u8 {
        self.values[self.head]
    }

    /// Gets a mutable reference to the byte at the read-write head.
    fn get_current_value_mut(&mut self) -> &mut u8 {
        self.values
            .get_mut(self.head)
            .expect("Head is out of bounds")
    }

    /// Sets the byte at the read-write head.
    fn set_current_value(&mut self, value: u8) {
        *self
            .values
            .get_mut(self.head)
            .expect("Head is out of bounds") = value;
    }

    /// Moves the instruction pointer to the matching
    /// branch backwards instruction.
    fn branch_forwards(&mut self) {
        let mut seen_branches_forwards = 1;

        while seen_branches_forwards > 0 {
            self.instruction_pointer += 1;

            if self.instruction_pointer >= self.program.len() {
                println!("Could not find matching branch");
                std::process::exit(1);
            }

            let current_instruction = self.program[self.instruction_pointer];

            if current_instruction == BFInstruction::BranchForwards {
                seen_branches_forwards += 1;
            }
            if current_instruction == BFInstruction::BranchBackwards {
                seen_branches_forwards -= 1;
            }
        }
    }

    /// Moves the instruction pointer to the matching
    /// branch forwards instruction.
    fn branch_backwards(&mut self) {
        let mut seen_branches_backwards = 1;

        while seen_branches_backwards > 0 {
            if self.instruction_pointer == 0 {
                println!("Could not find matching branch");
                std::process::exit(1);
            }

            self.instruction_pointer -= 1;

            let current_instruction = self.program[self.instruction_pointer];

            if current_instruction == BFInstruction::BranchForwards {
                seen_branches_backwards -= 1;
            }
            if current_instruction == BFInstruction::BranchBackwards {
                seen_branches_backwards += 1;
            }
        }
    }

    /// Runs the program, returning the final cell tape.
    pub fn run(mut self) -> [u8; N] {
        while let Some(instruction) = self.program.get(self.instruction_pointer)
        {
            match instruction {
                BFInstruction::IncrementHead => {
                    self.head += 1;
                    self.head %= N;
                },
                BFInstruction::DecrementHead => {
                    self.head = if self.head == 0 {
                        N
                    } else {
                        self.head
                    } - 1;
                },
                BFInstruction::IncrementByte => {
                    let new_val = self.get_current_value_mut().wrapping_add(1);
                    self.set_current_value(new_val);
                },
                BFInstruction::DecrementByte => {
                    let new_val = self.get_current_value_mut().wrapping_sub(1);
                    self.set_current_value(new_val);
                },
                BFInstruction::InputByte => {
                    let input_byte = std::io::stdin()
                        .bytes()
                        .next()
                        .map(Result::ok)
                        .flatten()
                        .expect("Could not read byte.");

                    *self.get_current_value_mut() = input_byte;
                },
                BFInstruction::OutputByte => {
                    print!("{}", self.get_current_value() as char);
                },
                BFInstruction::BranchForwards => {
                    if self.get_current_value() == 0 {
                        self.branch_forwards();
                    }
                },
                BFInstruction::BranchBackwards => {
                    if self.get_current_value() != 0 {
                        self.branch_backwards();
                    }
                },
            }

            self.instruction_pointer += 1;
        }

        self.values
    }
}