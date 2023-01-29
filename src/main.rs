use std::net::UdpSocket;
use std::os::unix::io::AsRawFd;
use std::io::ErrorKind;


const PACKET_SIZE: usize = 1316;
const REQUESTED_SEND_BUF_SIZE: i32 = 20000;

const SOURCE_ADDR: &str = "10.0.0.2:1234";
const DEST_ADDR: &str = "10.0.0.8:1235";

const NUM_PACKETS_TO_SEND: i32 = 200;

fn main() {
    let socket = UdpSocket::bind(SOURCE_ADDR).unwrap();
    socket.set_nonblocking(true).unwrap();
    let mut data = vec![0u8; PACKET_SIZE];
    let bufsize = get_send_buffer_size(&socket).unwrap();
    println!("The initial send buffer size is: {}", bufsize);
    set_send_buffer_size(&socket, REQUESTED_SEND_BUF_SIZE).unwrap();
    let actual_bufsize = get_send_buffer_size(&socket).unwrap();
    println!("The new send buffer size after trying to set it to {} is: {}", REQUESTED_SEND_BUF_SIZE, actual_bufsize);
    
    let mut would_block_errors: u32 = 0;
    for seq_num in 1..NUM_PACKETS_TO_SEND {
        data[0..4].copy_from_slice(&seq_num.to_be_bytes());
        match socket.send_to(&data, DEST_ADDR) {
            Ok(_) => {
                println!("Did send packet")
            }
            Err(e) => {
                if e.kind() == ErrorKind::WouldBlock {
                    would_block_errors += 1;
                    println!("Send buffer is full, unable to send more data, seq num: {}", seq_num);
                } else {
                    println!("An error occurred: {:?}", e);
                    break;
                }
            }
        }
    }
    println!("WouldBlock errors encountered: {}", would_block_errors)
}

fn set_send_buffer_size(socket: &UdpSocket, new_buffer_size: libc::c_int) -> std::io::Result<()> {
    return setsockopt(socket.as_raw_fd(), libc::SOL_SOCKET, libc::SO_SNDBUF, new_buffer_size);
}

fn get_send_buffer_size(socket: &UdpSocket) -> std::io::Result<libc::c_int> {
    return getsockopt(socket.as_raw_fd(), libc::SOL_SOCKET, libc::SO_SNDBUF);
}


fn getsockopt(fd: std::os::unix::io::RawFd, level: libc::c_int, optname: libc::c_int) -> std::io::Result<libc::c_int> {
    let mut optval = 0;
    let mut optlen = std::mem::size_of::<libc::c_int>() as libc::socklen_t;
    unsafe {
        if libc::getsockopt(fd, level, optname, &mut optval as *mut _ as *mut _, &mut optlen) == 0 {
            Ok(optval)
        } else {
            Err(std::io::Error::last_os_error())
        }
    }
}

fn setsockopt(fd: std::os::unix::io::RawFd, level: libc::c_int, optname: libc::c_int, optval: libc::c_int) -> std::io::Result<()> {
    let optval = optval as libc::c_int;
    let optlen = std::mem::size_of::<libc::c_int>() as libc::socklen_t;
    unsafe {
        if libc::setsockopt(fd, level, optname, &optval as *const _ as *const _, optlen) == 0 {
            Ok(())
        } else {
            Err(std::io::Error::last_os_error())
        }
    }
}