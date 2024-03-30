pub use regs::Cp0Register;

use regs::Status;

mod regs;

#[derive(Debug)]
pub struct Cp0 {
    regs: [i64; 32],
}

impl Cp0 {
    pub fn new() -> Self {
        Self { regs: [0; 32] }
    }

    pub fn write_reg(&mut self, reg: Cp0Register, value: i64) {
        self.regs[reg as usize] = value;

        match reg {
            Cp0Register::Status => {
                let status = Status::from(value as u32);
                println!("  Status: {:?}", status);
                assert_eq!(status.ksu(), 0, "Only kernel mode is supported");
                assert!(!status.kx(), "Only 32-bit addressing is supported");
                assert_eq!(status.ds(), 0, "Diagnostics are not supported");
                assert!(!status.rp(), "Low power mode is not supported");
            }
            _ => todo!("Write to {:?}", reg),
        }
    }
}
