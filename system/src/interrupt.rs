use bitflags::bitflags;
use std::sync::{Arc, Mutex};
use tracing::debug;

bitflags! {
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub struct CpuIntType: u8 {
        const Rcp = 0x04;
        const Pif = 0x10;
    }
}

bitflags! {
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub struct RcpIntType: u8 {
        const SP = 0x01;
        const SI = 0x02;
        const AI = 0x04;
        const VI = 0x08;
        const PI = 0x10;
        const DP = 0x20;
    }
}

#[derive(Clone)]
pub struct CpuInterrupt {
    status: CpuIntType,
}

impl CpuInterrupt {
    pub fn new() -> Self {
        Self {
            status: CpuIntType::empty(),
        }
    }

    pub fn status(&self) -> CpuIntType {
        self.status
    }

    pub fn raise(&mut self, int_type: CpuIntType) {
        let prev_status = self.status;
        self.status |= int_type;

        if self.status != prev_status {
            debug!("CPU Interrupt Raised: {:?}", int_type);
        }
    }

    pub fn clear(&mut self, int_type: CpuIntType) {
        let prev_status = self.status;
        self.status &= !int_type;

        if self.status != prev_status {
            debug!("CPU Interrupt Cleared: {:?}", int_type);
        }
    }
}

#[derive(Clone)]
pub struct RcpInterrupt {
    cpu_interrupt: Arc<Mutex<CpuInterrupt>>,
    status: RcpIntType,
    mask: RcpIntType,
}

impl RcpInterrupt {
    pub fn new(cpu_interrupt: Arc<Mutex<CpuInterrupt>>) -> Self {
        Self {
            cpu_interrupt,
            mask: RcpIntType::empty(),
            status: RcpIntType::empty(),
        }
    }

    pub fn mask(&self) -> RcpIntType {
        self.mask
    }

    pub fn set_mask(&mut self, mask: RcpIntType) {
        self.mask = mask;
        debug!("RCP Interrupt Mask: {:06b}", mask);
        self.update();
    }

    pub fn status(&self) -> RcpIntType {
        self.status
    }

    pub fn has(&self, int_type: RcpIntType) -> bool {
        self.status.intersects(int_type)
    }

    pub fn raise(&mut self, int_type: RcpIntType) {
        let prev_status = self.status;
        self.status |= int_type;

        if self.status != prev_status {
            debug!("RCP Interrupt Raised: {:?}", int_type);
        }

        self.update();
    }

    pub fn clear(&mut self, int_type: RcpIntType) {
        let prev_status = self.status;
        self.status &= !int_type;

        if self.status != prev_status {
            debug!("RCP Interrupt Cleared: {:?}", int_type);
        }

        self.update();
    }

    fn update(&mut self) {
        let mut cpu_int = self.cpu_interrupt.lock().unwrap();
        let active = self.status & self.mask;

        if active.is_empty() {
            cpu_int.clear(CpuIntType::Rcp);
        } else {
            cpu_int.raise(CpuIntType::Rcp);
        }
    }
}
