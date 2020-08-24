pub mod x32;
pub mod x32_pae;
pub mod x64;

use super::{
    mmu_spec::{ArchMMUSpec, MMUTranslationBase},
    Architecture, Endianess, ScopedVirtualTranslate,
};

use super::Bump;
use crate::error::{Error, Result};
use crate::iter::SplitAtIndex;
use crate::mem::PhysicalMemory;
use crate::types::{Address, PhysicalAddress};
use std::ptr;

pub struct X86Architecture {
    /// Defines how many bits does the native word size have
    bits: u8,
    /// Defines the byte order of the architecture
    endianess: Endianess,
    /// Defines the underlying MMU used for address translation
    mmu: ArchMMUSpec,
}

impl Architecture for X86Architecture {
    fn bits(&self) -> u8 {
        self.bits
    }

    fn endianess(&self) -> Endianess {
        self.endianess
    }

    fn page_size(&self) -> usize {
        self.mmu.page_size_level(1)
    }

    fn size_addr(&self) -> usize {
        self.mmu.addr_size.into()
    }
}

#[derive(Clone, Copy)]
pub struct X86ScopedVirtualTranslate {
    arch: &'static X86Architecture,
    dtb: X86PageTableBase,
}

impl X86ScopedVirtualTranslate {
    pub fn new(arch: &'static X86Architecture, dtb: Address) -> Self {
        Self {
            arch,
            dtb: X86PageTableBase(dtb),
        }
    }
}

impl ScopedVirtualTranslate for X86ScopedVirtualTranslate {
    fn virt_to_phys_iter<
        T: PhysicalMemory + ?Sized,
        B: SplitAtIndex,
        VI: Iterator<Item = (Address, B)>,
        VO: Extend<(PhysicalAddress, B)>,
        FO: Extend<(Error, Address, B)>,
    >(
        &self,
        mem: &mut T,
        addrs: VI,
        out: &mut VO,
        out_fail: &mut FO,
        arena: &Bump,
    ) {
        self.arch
            .mmu
            .virt_to_phys_iter(mem, self.dtb, addrs, out, out_fail, arena)
    }

    fn translation_table_id(&self, address: Address) -> usize {
        self.dtb.0.as_u64().overflowing_shr(12).0 as usize
    }

    fn arch(&self) -> &dyn Architecture {
        self.arch
    }
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct X86PageTableBase(Address);

impl MMUTranslationBase for X86PageTableBase {
    fn get_initial_pt(&self, _: Address) -> Address {
        self.0
    }
}

fn underlying_arch(arch: &dyn Architecture) -> Option<&'static X86Architecture> {
    if ptr::eq(arch, x64::ARCH) {
        Some(&x64::ARCH_SPEC)
    } else if ptr::eq(arch, x32::ARCH) {
        Some(&x32::ARCH_SPEC)
    } else if ptr::eq(arch, x32_pae::ARCH) {
        Some(&x32_pae::ARCH_SPEC)
    } else {
        None
    }
}

pub fn new_translator(
    dtb: Address,
    arch: &dyn Architecture,
) -> Result<impl ScopedVirtualTranslate> {
    let arch = underlying_arch(arch).ok_or(Error::InvalidArchitecture)?;
    Ok(X86ScopedVirtualTranslate::new(arch, dtb))
}

pub fn is_x86_arch(arch: &dyn Architecture) -> bool {
    underlying_arch(arch).is_some()
}
