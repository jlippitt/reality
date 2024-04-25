use bitflags::bitflags;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
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

#[repr(transparent)]
pub struct CpuInterrupt {
    status: AtomicU8,
}

impl CpuInterrupt {
    pub fn new() -> Self {
        Self {
            status: AtomicU8::new(0),
        }
    }

    pub fn status(&self) -> u8 {
        self.status.load(Ordering::Relaxed)
    }

    pub fn raise(&self, int_type: CpuIntType) {
        let bits = int_type.bits();
        let prev_status = self.status.fetch_or(bits, Ordering::Relaxed);

        if (prev_status & bits) == 0 {
            debug!("CPU Interrupt Raised: {:?}", int_type);
        }
    }

    pub fn clear(&self, int_type: CpuIntType) {
        let bits = int_type.bits();
        let prev_status = self.status.fetch_and(!bits, Ordering::Relaxed);

        if (prev_status & bits) != 0 {
            debug!("CPU Interrupt Cleared: {:?}", int_type);
        }
    }
}

#[derive(Clone)]
pub struct RcpInterrupt {
    cpu_interrupt: Arc<CpuInterrupt>,
    status: RcpIntType,
    mask: RcpIntType,
}

impl RcpInterrupt {
    pub fn new(cpu_interrupt: Arc<CpuInterrupt>) -> Self {
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
        let active = self.status & self.mask;

        if active.is_empty() {
            self.cpu_interrupt.clear(CpuIntType::Rcp);
        } else {
            self.cpu_interrupt.raise(CpuIntType::Rcp);
        }
    }
}
