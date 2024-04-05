use bitflags::bitflags;
use std::cell::Cell;
use std::rc::Rc;
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
    status: Rc<Cell<CpuIntType>>,
}

impl CpuInterrupt {
    pub fn new() -> Self {
        Self {
            status: Rc::new(Cell::new(CpuIntType::empty())),
        }
    }

    pub fn status(&self) -> CpuIntType {
        self.status.get()
    }

    pub fn raise(&mut self, int_type: CpuIntType) {
        let prev_status = self.status.get();
        self.status.set(prev_status | int_type);

        if self.status.get() != prev_status {
            debug!("CPU Interrupt Raised: {:?}", int_type);
        }
    }

    pub fn clear(&mut self, int_type: CpuIntType) {
        let prev_status = self.status.get();
        self.status.set(prev_status & !int_type);

        if self.status.get() != prev_status {
            debug!("CPU Interrupt Cleared: {:?}", int_type);
        }
    }
}

#[derive(Clone)]
pub struct RcpInterrupt {
    cpu_interrupt: CpuInterrupt,
    status: Rc<Cell<RcpIntType>>,
    mask: Rc<Cell<RcpIntType>>,
}

impl RcpInterrupt {
    pub fn new(cpu_interrupt: CpuInterrupt) -> Self {
        Self {
            cpu_interrupt,
            mask: Rc::new(Cell::new(RcpIntType::empty())),
            status: Rc::new(Cell::new(RcpIntType::empty())),
        }
    }

    pub fn mask(&self) -> RcpIntType {
        self.mask.get()
    }

    pub fn set_mask(&mut self, mask: RcpIntType) {
        self.mask.set(mask);
        debug!("RCP Interrupt Mask: {:06b}", mask);
        self.update();
    }

    pub fn status(&self) -> RcpIntType {
        self.status.get()
    }

    pub fn has(&self, int_type: RcpIntType) -> bool {
        self.status.get().intersects(int_type)
    }

    pub fn raise(&mut self, int_type: RcpIntType) {
        let prev_status = self.status.get();
        self.status.set(prev_status | int_type);

        if self.status.get() != prev_status {
            debug!("RCP Interrupt Raised: {:?}", int_type);
        }

        self.update();
    }

    pub fn clear(&mut self, int_type: RcpIntType) {
        let prev_status = self.status.get();
        self.status.set(prev_status & !int_type);

        if self.status.get() != prev_status {
            debug!("RCP Interrupt Cleared: {:?}", int_type);
        }

        self.update();
    }

    fn update(&mut self) {
        let active = self.status.get() & self.mask.get();

        if active.is_empty() {
            self.cpu_interrupt.clear(CpuIntType::Rcp);
        } else {
            self.cpu_interrupt.raise(CpuIntType::Rcp);
        }
    }
}
