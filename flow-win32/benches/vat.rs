use std::time::Duration;

#[macro_use]
extern crate bencher;

use bencher::Bencher;

extern crate flow_core;
extern crate flow_qemu_procfs;
extern crate flow_win32;
extern crate rand;

use flow_core::address::Address;
use flow_core::arch::Architecture;
use flow_core::mem::{AccessPhysicalMemory, AccessVirtualMemory, CachedMemoryAccess, TimedCache};

use flow_qemu_procfs::Memory;

use flow_core::{Length, OsProcess, OsProcessModule, PageType};
use flow_win32::{Win32, Win32Module, Win32Offsets, Win32Process};

use rand::prelude::*;
use rand::{prng::XorShiftRng as CurRng, Rng, SeedableRng};

fn find_module<T: AccessPhysicalMemory + AccessVirtualMemory>(
    mem: &mut T,
) -> flow_core::Result<(Win32, Win32Process, Win32Module)> {
    let os = Win32::try_with(mem)?;
    let offsets = Win32Offsets::try_with_guid(&os.kernel_guid())?;

    let mut rng = CurRng::from_rng(thread_rng()).unwrap();

    let proc_list = os.eprocess_list(mem, &offsets).unwrap();

    for i in -100..(proc_list.len() as isize) {
        let idx = if i >= 0 {
            i as usize
        } else {
            rng.gen_range(0, proc_list.len())
        };

        if let Ok(proc) = Win32Process::try_with_eprocess(mem, &os, &offsets, proc_list[idx]) {
            let mod_list: Vec<Win32Module> = proc
                .peb_list(mem)
                .unwrap_or_default()
                .iter()
                .filter_map(|&x| {
                    if let Ok(module) = Win32Module::try_with_peb(mem, &proc, &offsets, x) {
                        if module.size() > 0x10000.into() {
                            Some(module)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();

            if !mod_list.is_empty() {
                let tmod = &mod_list[rng.gen_range(0, mod_list.len())];
                return Ok((os, proc, tmod.clone()));
            }
        }
    }

    Err("No module found!".into())
}

fn vat_test<T: AccessVirtualMemory + AccessPhysicalMemory>(
    bench: &mut Bencher,
    mem: &mut T,
    range_start: u64,
    range_end: u64,
    translations: usize,
    dtb: Address,
    arch: Architecture,
) {
    let mut rng = CurRng::from_rng(thread_rng()).unwrap();

    bench.iter(|| {
        for _ in 0..translations {
            let vaddr: Address = rng.gen_range(range_start, range_end).into();
            let _thing = arch.virt_to_phys(mem, dtb, vaddr);
        }
    });

    bench.bytes = (translations * 8) as u64;
}

fn translate_module(bench: &mut Bencher) {
    let mut mem_sys = Memory::new().unwrap();
    let (os, proc, tmod) = find_module(&mut mem_sys).unwrap();
    let mut cache = TimedCache::new(
        os.start_block.arch,
        Length::from_mb(32),
        Duration::from_millis(1000).into(),
        PageType::PAGE_TABLE | PageType::READ_ONLY,
    );
    let mut mem = CachedMemoryAccess::with(&mut mem_sys, &mut cache);
    vat_test(
        bench,
        &mut mem,
        tmod.base().as_u64(),
        tmod.base().as_u64() + tmod.size().as_u64(),
        0x100,
        proc.dtb(),
        proc.sys_arch(),
    );
}

fn translate_module_smallrange(bench: &mut Bencher) {
    let mut mem_sys = Memory::new().unwrap();
    let (os, proc, tmod) = find_module(&mut mem_sys).unwrap();
    let mut cache = TimedCache::new(
        os.start_block.arch,
        Length::from_mb(32),
        Duration::from_millis(1000).into(),
        PageType::PAGE_TABLE | PageType::READ_ONLY,
    );
    let mut mem = CachedMemoryAccess::with(&mut mem_sys, &mut cache);
    vat_test(
        bench,
        &mut mem,
        tmod.base().as_u64(),
        tmod.base().as_u64() + 0x2000,
        0x100,
        proc.dtb(),
        proc.sys_arch(),
    );
}

fn translate_range(bench: &mut Bencher, range_start: u64, range_end: u64) {
    let mut mem_sys = Memory::new().unwrap();
    let (os, proc, _) = find_module(&mut mem_sys).unwrap();
    let mut cache = TimedCache::new(
        os.start_block.arch,
        Length::from_mb(32),
        Duration::from_millis(1000).into(),
        PageType::PAGE_TABLE | PageType::READ_ONLY,
    );
    let mut mem = CachedMemoryAccess::with(&mut mem_sys, &mut cache);
    vat_test(
        bench,
        &mut mem,
        range_start,
        range_end,
        0x100,
        proc.dtb(),
        proc.sys_arch(),
    );
}

fn translate_allmem(bench: &mut Bencher) {
    translate_range(bench, 0, !0);
}

benchmark_group!(
    benches,
    translate_module,
    translate_module_smallrange,
    translate_allmem
);
benchmark_main!(benches);