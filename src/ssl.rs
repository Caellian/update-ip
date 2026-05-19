//! Minimal OpenSSL FFI wrapper.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::os::fd::AsRawFd;

const CA_BUNDLE: &[u8] = concat!(env!("CA_BUNDLE"), "\0").as_bytes();

#[allow(non_camel_case_types)]
mod ffi {
    pub enum SSL_CTX {}
    pub enum SSL {}
    pub enum SSL_METHOD {}

    #[link(name = "ssl")]
    #[link(name = "crypto")]
    unsafe extern "C" {
        pub fn TLS_client_method() -> *const SSL_METHOD;
        pub fn SSL_CTX_new(method: *const SSL_METHOD) -> *mut SSL_CTX;
        pub fn SSL_CTX_free(ctx: *mut SSL_CTX);
        pub fn SSL_CTX_load_verify_locations(
            ctx: *mut SSL_CTX, file: *const u8, dir: *const u8,
        ) -> i32;
        pub fn SSL_CTX_set_verify(ctx: *mut SSL_CTX, mode: i32, cb: *const ());
        pub fn SSL_new(ctx: *mut SSL_CTX) -> *mut SSL;
        pub fn SSL_free(ssl: *mut SSL);
        pub fn SSL_set_fd(ssl: *mut SSL, fd: i32) -> i32;
        pub fn SSL_ctrl(ssl: *mut SSL, cmd: i32, larg: i64, parg: *const u8) -> i64;
        pub fn SSL_connect(ssl: *mut SSL) -> i32;
        pub fn SSL_write(ssl: *mut SSL, buf: *const u8, num: i32) -> i32;
        pub fn SSL_read(ssl: *mut SSL, buf: *mut u8, num: i32) -> i32;
        pub fn SSL_get_verify_result(ssl: *const SSL) -> i64;
    }

    pub const SSL_VERIFY_PEER: i32 = 0x01;
    pub const SSL_CTRL_SET_TLSEXT_HOSTNAME: i32 = 55;
    pub const X509_V_OK: i64 = 0;
}

pub struct SslStream {
    ssl: *mut ffi::SSL,
    ctx: *mut ffi::SSL_CTX,
    _tcp: TcpStream,
}

impl SslStream {
    pub fn connect(host: &str, tcp: TcpStream) -> Option<Self> {
        unsafe {
            let ctx = ffi::SSL_CTX_new(ffi::TLS_client_method());
            if ctx.is_null() { return None; }
            ffi::SSL_CTX_load_verify_locations(ctx, CA_BUNDLE.as_ptr(), std::ptr::null());
            ffi::SSL_CTX_set_verify(ctx, ffi::SSL_VERIFY_PEER, std::ptr::null());

            let ssl = ffi::SSL_new(ctx);
            if ssl.is_null() {
                ffi::SSL_CTX_free(ctx);
                return None;
            }

            let mut host_z = Vec::with_capacity(host.len() + 1);
            host_z.extend_from_slice(host.as_bytes());
            host_z.push(0);
            ffi::SSL_ctrl(ssl, ffi::SSL_CTRL_SET_TLSEXT_HOSTNAME, 0, host_z.as_ptr());
            ffi::SSL_set_fd(ssl, tcp.as_raw_fd());

            if ffi::SSL_connect(ssl) != 1 {
                ffi::SSL_free(ssl);
                ffi::SSL_CTX_free(ctx);
                return None;
            }
            if ffi::SSL_get_verify_result(ssl) != ffi::X509_V_OK {
                ffi::SSL_free(ssl);
                ffi::SSL_CTX_free(ctx);
                return None;
            }

            Some(SslStream { ssl, ctx, _tcp: tcp })
        }
    }
}

impl Write for SslStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = unsafe { ffi::SSL_write(self.ssl, buf.as_ptr(), buf.len() as i32) };
        if n <= 0 {
            Err(std::io::Error::other("SSL_write failed"))
        } else {
            Ok(n as usize)
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Read for SslStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = unsafe { ffi::SSL_read(self.ssl, buf.as_mut_ptr(), buf.len() as i32) };
        if n < 0 {
            Err(std::io::Error::other("SSL_read failed"))
        } else {
            Ok(n as usize)
        }
    }
}

impl Drop for SslStream {
    fn drop(&mut self) {
        unsafe {
            ffi::SSL_free(self.ssl);
            ffi::SSL_CTX_free(self.ctx);
        }
    }
}
