use bitfield_struct::bitfield;

#[derive(Debug, Default)]
pub struct Regs {
    pub dram_addr: u32,
    pub cart_addr: u32,
    pub bsd_dom: [BsdDom; 2],
}

#[derive(Debug, Default)]
pub struct BsdDom {
    pub lat: BsdDomLat,
    pub pwd: BsdDomPwd,
    pub pgs: BsdDomPgs,
    pub rls: BsdDomRls,
}
#[bitfield(u32)]
pub struct BsdDomLat {
    #[bits(8)]
    pub lat: u32,
    #[bits(24)]
    __: u32,
}

#[bitfield(u32)]
pub struct BsdDomPwd {
    #[bits(8)]
    pub pwd: u32,
    #[bits(24)]
    __: u32,
}

#[bitfield(u32)]
pub struct BsdDomPgs {
    #[bits(4)]
    pub pgs: u32,
    #[bits(28)]
    __: u32,
}

#[bitfield(u32)]
pub struct BsdDomRls {
    #[bits(2)]
    pub rls: u32,
    #[bits(30)]
    __: u32,
}
