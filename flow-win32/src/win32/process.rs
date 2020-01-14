//pub mod user;
//pub use user::UserProcess;

//pub mod kernel;
//pub use kernel::KernelProcess;

//pub mod user_iter;
//pub use user_iter::ProcessIterator;

// new temp module (will replace all the above)
pub mod win_user;
pub use win_user::*;

use crate::error::Result;

use super::Win32;
use crate::offsets::Win32Offsets;

use flow_core::address::Address;
use flow_core::arch::ArchitectureTrait;
use flow_core::mem::*;
use flow_core::process::*;

pub trait Win32Process {
    fn wow64(&self) -> Address;
    fn peb(&self) -> Address;
    fn peb_module(&self) -> Address;

    fn peb_list<T: VirtualRead>(&self, mem: &mut T, offsets: &Win32Offsets)
        -> Result<Vec<Address>>;
}

/*
use crate::win::module::{Module, ModuleIterator};

pub trait ProcessModuleTrait {
    fn first_peb_entry(&mut self) -> Result<Address>;
    fn module_iter(&self) -> Result<ModuleIterator<Self>>
    where
        Self: Sized + ArchitectureTrait + VirtualReadHelper + VirtualReadHelperFuncs;
}

pub trait ProcessModuleHelper
where
    Self:
        Sized + ProcessModuleTrait + ArchitectureTrait + VirtualReadHelper + VirtualReadHelperFuncs,
{
    fn first_module(&self) -> Result<Module<Self>>;
    fn module(&self, name: &str) -> Result<Module<Self>>;
    fn containing_module(&self, addr: Address) -> Result<Module<Self>>;
}

impl<T> ProcessModuleHelper for T
where
    T: Sized + ProcessModuleTrait + ArchitectureTrait + VirtualReadHelper + VirtualReadHelperFuncs,
{
    fn first_module(&self) -> Result<Module<Self>> {
        Ok(self
            .module_iter()?
            .nth(0)
            .ok_or_else(|| "unable to read first module")?)
    }

    fn module(&self, name: &str) -> Result<Module<Self>> {
        Ok(self
            .module_iter()?
            .filter_map(|mut m| {
                if m.name().unwrap_or_default() == name {
                    Some(m)
                } else {
                    None
                }
            })
            .nth(0)
            .ok_or_else(|| "unable to find module")?)
    }

    fn containing_module(&self, addr: Address) -> Result<Module<Self>> {
        Ok(self
            .module_iter()?
            .filter_map(|mut m| {
                let base = m.base().unwrap_or_default();
                let size = m.size().unwrap_or_default();

                if base <= addr && addr <= base + size {
                    Some(m)
                } else {
                    None
                }
            })
            .nth(0)
            .ok_or_else(|| "unable to find containing module")?)
    }
}
*/
