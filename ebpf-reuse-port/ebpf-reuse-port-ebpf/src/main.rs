#![no_std]
#![no_main]
#![allow(nonstandard_style, dead_code)]

mod bindings;

use aya_bpf::{
    bindings::xdp_action,
    macros::{map, xdp},
    maps::PerfEventArray,
    programs::XdpContext,
};
use aya_log_ebpf::info;

use core::mem;
use ebpf_reuse_port_common::PacketLog;
use memoffset::offset_of;

use bindings::{ethhdr, iphdr};

#[xdp(name = "ebpf_reuse_port")]
pub fn ebpf_reuse_port(ctx: XdpContext) -> u32 {
    match try_ebpf_reuse_port(ctx) {
        Ok(ret) => ret,
        Err(_) => xdp_action::XDP_ABORTED,
    }
}

fn try_ebpf_reuse_port(ctx: XdpContext) -> Result<u32, u32> {
    info!(&ctx, "received a packet");
    Ok(xdp_action::XDP_PASS)
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}

#[map(name = "EVENTS")] //
static mut EVENTS: PerfEventArray<PacketLog> =
    PerfEventArray::<PacketLog>::with_max_entries(1024, 0);

#[xdp]
pub fn xdp_firewall(ctx: XdpContext) -> u32 {
    match try_xdp_firewall(ctx) {
        Ok(ret) => ret,
        Err(_) => xdp_action::XDP_ABORTED,
    }
}

#[inline(always)] //
unsafe fn ptr_at<T>(ctx: &XdpContext, offset: usize) -> Result<*const T, ()> {
    let start = ctx.data();
    let end = ctx.data_end();
    let len = mem::size_of::<T>();

    if start + offset + len > end {
        return Err(());
    }

    Ok((start + offset) as *const T)
}

fn try_xdp_firewall(ctx: XdpContext) -> Result<u32, ()> {
    let h_proto = u16::from_be(unsafe {
        *ptr_at(&ctx, offset_of!(ethhdr, h_proto))? //
    });
    if h_proto != ETH_P_IP {
        return Ok(xdp_action::XDP_PASS);
    }
    let source = u32::from_be(unsafe { *ptr_at(&ctx, ETH_HDR_LEN + offset_of!(iphdr, saddr))? });

    let log_entry = PacketLog {
        ipv4_address: source,
        action: xdp_action::XDP_PASS,
    };
    unsafe {
        EVENTS.output(&ctx, &log_entry, 0); //
    }
    Ok(xdp_action::XDP_PASS)
}

const ETH_P_IP: u16 = 0x0800;
const ETH_HDR_LEN: usize = mem::size_of::<ethhdr>();
