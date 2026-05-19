use std::io::Write;

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Ipv4Address(pub [u8; 4]);

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Ipv6Address(pub [u8; 16]);

pub trait Address: Copy {
    const RECORD_TYPE: &'static str;
    fn to_str<'a>(&self, buf: &'a mut [u8]) -> &'a str;
    fn write_str<W: Write>(&self, w: &mut W) -> std::io::Result<()>;
}

impl Address for Ipv4Address {
    const RECORD_TYPE: &'static str = "A";

    fn to_str<'a>(&self, buf: &'a mut [u8]) -> &'a str {
        let mut pos = 0;
        for (i, &o) in self.0.iter().enumerate() {
            if i > 0 {
                buf[pos] = b'.';
                pos += 1;
            }
            let mut nbuf = [0u8; 4];
            let s = crate::util::u8_to_str(o, &mut nbuf);
            buf[pos..pos + s.len()].copy_from_slice(s.as_bytes());
            pos += s.len();
        }
        unsafe { std::str::from_utf8_unchecked(&buf[..pos]) }
    }

    fn write_str<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        for (i, &o) in self.0.iter().enumerate() {
            if i > 0 {
                w.write_all(b".")?;
            }
            let mut nbuf = [0u8; 4];
            let s = crate::util::u8_to_str(o, &mut nbuf);
            w.write_all(s.as_bytes())?;
        }
        Ok(())
    }
}

impl Address for Ipv6Address {
    const RECORD_TYPE: &'static str = "AAAA";

    fn to_str<'a>(&self, buf: &'a mut [u8]) -> &'a str {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut pos = 0;
        for (i, chunk) in self.0.chunks(2).enumerate() {
            if i > 0 {
                buf[pos] = b':';
                pos += 1;
            }
            let val = ((chunk[0] as u16) << 8) | (chunk[1] as u16);
            let mut started = false;
            for shift in [12, 8, 4, 0] {
                let nibble = ((val >> shift) & 0xf) as usize;
                if nibble != 0 || started || shift == 0 {
                    buf[pos] = HEX[nibble];
                    pos += 1;
                    started = true;
                }
            }
        }
        unsafe { std::str::from_utf8_unchecked(&buf[..pos]) }
    }

    fn write_str<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        for (i, chunk) in self.0.chunks(2).enumerate() {
            if i > 0 {
                w.write_all(b":")?;
            }
            let val = ((chunk[0] as u16) << 8) | (chunk[1] as u16);
            let mut started = false;
            for shift in [12, 8, 4, 0] {
                let nibble = ((val >> shift) & 0xf) as usize;
                if nibble != 0 || started || shift == 0 {
                    w.write_all(&[HEX[nibble]])?;
                    started = true;
                }
            }
        }
        Ok(())
    }
}
