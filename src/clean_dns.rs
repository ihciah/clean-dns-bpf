#![allow(unused_attributes)]
#![no_std]
#![no_main]

use redbpf_probes::xdp::prelude::*;
program!(0xFFFFFFFE, "GPL");
#[xdp]
fn clean_dns(ctx: XdpContext) -> XdpResult {
    let ip = ctx.ip()?;
    // only match udp 8.8.8.8
    if unsafe { (*ip).protocol as u32 } != IPPROTO_UDP
        || unsafe { (*ip).saddr as u32 } != 0x08080808
    {
        return Ok(XdpAction::Pass);
    }
    let transport = ctx.transport()?;
    // only match 53
    if transport.source() != 53 {
        return Ok(XdpAction::Pass);
    }

    // drop if id is 0
    if unsafe { (*ip).id } == 0 {
        return Ok(XdpAction::Drop);
    }
    // drop if flag is 0x40(Don't fragment)
    if unsafe { (*ip).frag_off } == 0x0040 {
        return Ok(XdpAction::Drop);
    }

    // get first 10 byte udp data(7,8 is Answer RRs, 8,9 is Authority RRs)
    let udp_data = ctx.data()?;
    let data = udp_data.slice(10)?;
    // pass if the dns packet has multiple answers
    if data[6] != 0 || data[7] != 1 {
        // Answer RR != 1
        return Ok(XdpAction::Pass);
    }
    // pass if the dns packet has authority answer
    if data[8] != 0 || data[9] != 0 {
        // Authority RR != 0
        return Ok(XdpAction::Pass);
    }
    // drop if dns flag has Authoritative mark
    if (data[2] & 0b0000_0100) != 0 {
        return Ok(XdpAction::Drop);
    }

    Ok(XdpAction::Pass)
}
