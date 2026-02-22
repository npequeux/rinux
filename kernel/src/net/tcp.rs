//! Transmission Control Protocol (TCP)
//!
//! TCP implementation with connection management, reliable transmission,
//! flow control, and congestion control.

use alloc::collections::{btree_map, BTreeMap, VecDeque};
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, AtomicU16, AtomicU32, Ordering};
use spin::Mutex;

use super::ipv4::{calculate_pseudo_header_checksum, IpProtocol, Ipv4Addr};
use super::socket::{
    ShutdownHow, Socket, SocketAddr, SocketAddrV4, SocketError, SocketOption, SocketOptionType,
    SocketState,
};

/// TCP header (20 bytes minimum)
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct TcpHeader {
    /// Source port
    pub src_port: u16,
    /// Destination port
    pub dst_port: u16,
    /// Sequence number
    pub seq_num: u32,
    /// Acknowledgment number
    pub ack_num: u32,
    /// Data offset (4 bits), reserved (3 bits), flags (9 bits)
    pub data_offset_flags: u16,
    /// Window size
    pub window: u16,
    /// Checksum
    pub checksum: u16,
    /// Urgent pointer
    pub urgent_ptr: u16,
}

impl TcpHeader {
    /// TCP header minimum size
    pub const MIN_SIZE: usize = 20;

    /// TCP header maximum size (with options)
    pub const MAX_SIZE: usize = 60;

    /// Create new TCP header
    pub const fn new(src_port: u16, dst_port: u16, seq: u32, ack: u32) -> Self {
        Self {
            src_port: src_port.to_be(),
            dst_port: dst_port.to_be(),
            seq_num: seq.to_be(),
            ack_num: ack.to_be(),
            data_offset_flags: ((5u16) << 12).to_be(), // 5 words (20 bytes), no flags
            window: 0,
            checksum: 0,
            urgent_ptr: 0,
        }
    }

    /// Parse header from bytes
    pub fn parse(data: &[u8]) -> Result<Self, TcpError> {
        if data.len() < Self::MIN_SIZE {
            return Err(TcpError::TooShort);
        }

        let data_offset_flags = u16::from_be_bytes([data[12], data[13]]);
        let data_offset = ((data_offset_flags >> 12) as usize) * 4;

        if !(Self::MIN_SIZE..=Self::MAX_SIZE).contains(&data_offset) {
            return Err(TcpError::InvalidHeaderLength);
        }

        if data.len() < data_offset {
            return Err(TcpError::TooShort);
        }

        Ok(Self {
            src_port: u16::from_be_bytes([data[0], data[1]]).to_be(),
            dst_port: u16::from_be_bytes([data[2], data[3]]).to_be(),
            seq_num: u32::from_be_bytes([data[4], data[5], data[6], data[7]]).to_be(),
            ack_num: u32::from_be_bytes([data[8], data[9], data[10], data[11]]).to_be(),
            data_offset_flags: data_offset_flags.to_be(),
            window: u16::from_be_bytes([data[14], data[15]]).to_be(),
            checksum: u16::from_be_bytes([data[16], data[17]]).to_be(),
            urgent_ptr: u16::from_be_bytes([data[18], data[19]]).to_be(),
        })
    }

    /// Write header to buffer
    pub fn write_to(&self, buffer: &mut [u8]) -> Result<(), TcpError> {
        if buffer.len() < Self::MIN_SIZE {
            return Err(TcpError::BufferTooSmall);
        }

        buffer[0..2].copy_from_slice(&u16::from_be(self.src_port).to_be_bytes());
        buffer[2..4].copy_from_slice(&u16::from_be(self.dst_port).to_be_bytes());
        buffer[4..8].copy_from_slice(&u32::from_be(self.seq_num).to_be_bytes());
        buffer[8..12].copy_from_slice(&u32::from_be(self.ack_num).to_be_bytes());
        buffer[12..14].copy_from_slice(&u16::from_be(self.data_offset_flags).to_be_bytes());
        buffer[14..16].copy_from_slice(&u16::from_be(self.window).to_be_bytes());
        buffer[16..18].copy_from_slice(&u16::from_be(self.checksum).to_be_bytes());
        buffer[18..20].copy_from_slice(&u16::from_be(self.urgent_ptr).to_be_bytes());

        Ok(())
    }

    /// Get source port
    pub fn src_port(&self) -> u16 {
        u16::from_be(self.src_port)
    }

    /// Get destination port
    pub fn dst_port(&self) -> u16 {
        u16::from_be(self.dst_port)
    }

    /// Get sequence number
    pub fn seq_num(&self) -> u32 {
        u32::from_be(self.seq_num)
    }

    /// Get acknowledgment number
    pub fn ack_num(&self) -> u32 {
        u32::from_be(self.ack_num)
    }

    /// Get data offset (header length in bytes)
    pub fn data_offset(&self) -> usize {
        ((u16::from_be(self.data_offset_flags) >> 12) as usize) * 4
    }

    /// Get window size
    pub fn window(&self) -> u16 {
        u16::from_be(self.window)
    }

    /// Get checksum
    pub fn checksum(&self) -> u16 {
        u16::from_be(self.checksum)
    }

    /// Get urgent pointer
    pub fn urgent_ptr(&self) -> u16 {
        u16::from_be(self.urgent_ptr)
    }

    /// Get flags
    pub fn flags(&self) -> TcpFlags {
        let flags_raw = u16::from_be(self.data_offset_flags) & 0x1FF;
        TcpFlags::from_bits_truncate(flags_raw)
    }

    /// Set flags
    pub fn set_flags(&mut self, flags: TcpFlags) {
        let offset_flags = u16::from_be(self.data_offset_flags);
        let offset = offset_flags & 0xF000;
        self.data_offset_flags = (offset | flags.bits()).to_be();
    }

    /// Set window size
    pub fn set_window(&mut self, window: u16) {
        self.window = window.to_be();
    }

    /// Calculate checksum
    pub fn calculate_checksum(&mut self, src_ip: Ipv4Addr, dst_ip: Ipv4Addr, payload: &[u8]) {
        // Zero out checksum field
        self.checksum = 0;

        // Build TCP segment (header + payload)
        let length = Self::MIN_SIZE + payload.len();
        let mut buffer = Vec::with_capacity(length);

        // Serialize TCP header
        let mut header_bytes = [0u8; Self::MIN_SIZE];
        let _ = self.write_to(&mut header_bytes);
        buffer.extend_from_slice(&header_bytes);

        // Add payload
        buffer.extend_from_slice(payload);

        // Calculate pseudo-header checksum
        let mut sum =
            calculate_pseudo_header_checksum(src_ip, dst_ip, IpProtocol::Tcp, length as u16);

        // Add TCP header and payload checksum
        for chunk in buffer.chunks(2) {
            let word = if chunk.len() == 2 {
                u16::from_be_bytes([chunk[0], chunk[1]])
            } else {
                u16::from_be_bytes([chunk[0], 0])
            };
            sum += word as u32;
        }

        // Fold 32-bit sum to 16 bits
        while sum >> 16 != 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }

        // One's complement
        let checksum = !sum as u16;
        self.checksum = if checksum == 0 { 0xFFFF } else { checksum }.to_be();
    }

    /// Verify checksum
    pub fn verify_checksum(&self, src_ip: Ipv4Addr, dst_ip: Ipv4Addr, payload: &[u8]) -> bool {
        let mut header = *self;
        header.calculate_checksum(src_ip, dst_ip, payload);
        header.checksum() == self.checksum()
    }
}

/// TCP flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TcpFlags {
    bits: u16,
}

impl TcpFlags {
    /// FIN - No more data from sender
    pub const FIN: u16 = 1 << 0;
    /// SYN - Synchronize sequence numbers
    pub const SYN: u16 = 1 << 1;
    /// RST - Reset the connection
    pub const RST: u16 = 1 << 2;
    /// PSH - Push function
    pub const PSH: u16 = 1 << 3;
    /// ACK - Acknowledgment field is significant
    pub const ACK: u16 = 1 << 4;
    /// URG - Urgent pointer field is significant
    pub const URG: u16 = 1 << 5;
    /// ECE - ECN-Echo
    pub const ECE: u16 = 1 << 6;
    /// CWR - Congestion Window Reduced
    pub const CWR: u16 = 1 << 7;
    /// NS - Nonce Sum
    pub const NS: u16 = 1 << 8;

    /// Create empty flags
    pub const fn empty() -> Self {
        Self { bits: 0 }
    }

    /// Create from bits
    pub const fn from_bits(bits: u16) -> Option<Self> {
        if bits & !0x1FF == 0 {
            Some(Self { bits })
        } else {
            None
        }
    }

    /// Create from bits, truncating invalid bits
    pub const fn from_bits_truncate(bits: u16) -> Self {
        Self { bits: bits & 0x1FF }
    }

    /// Get bits
    pub const fn bits(&self) -> u16 {
        self.bits
    }

    /// Check if flag is set
    pub const fn contains(&self, flag: u16) -> bool {
        self.bits & flag == flag
    }

    /// Set flag
    pub fn set(&mut self, flag: u16) {
        self.bits |= flag & 0x1FF;
    }

    /// Clear flag
    pub fn clear(&mut self, flag: u16) {
        self.bits &= !(flag & 0x1FF);
    }

    /// Check if FIN flag is set
    pub const fn is_fin(&self) -> bool {
        self.contains(Self::FIN)
    }

    /// Check if SYN flag is set
    pub const fn is_syn(&self) -> bool {
        self.contains(Self::SYN)
    }

    /// Check if RST flag is set
    pub const fn is_rst(&self) -> bool {
        self.contains(Self::RST)
    }

    /// Check if PSH flag is set
    pub const fn is_psh(&self) -> bool {
        self.contains(Self::PSH)
    }

    /// Check if ACK flag is set
    pub const fn is_ack(&self) -> bool {
        self.contains(Self::ACK)
    }

    /// Check if URG flag is set
    pub const fn is_urg(&self) -> bool {
        self.contains(Self::URG)
    }
}

/// TCP connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TcpState {
    /// Connection closed
    Closed,
    /// Listening for connections
    Listen,
    /// SYN sent, waiting for SYN-ACK
    SynSent,
    /// SYN received, waiting for ACK
    SynReceived,
    /// Connection established
    Established,
    /// FIN sent, waiting for ACK
    FinWait1,
    /// FIN received and ACKed, waiting for FIN
    FinWait2,
    /// Both sides have closed, waiting for FIN ACK
    Closing,
    /// Waiting for remote to close
    CloseWait,
    /// FIN sent after remote close
    LastAck,
    /// Waiting for all packets to expire
    TimeWait,
}

impl TcpState {
    /// Check if state allows sending data
    pub fn can_send(&self) -> bool {
        matches!(self, TcpState::Established | TcpState::CloseWait)
    }

    /// Check if state allows receiving data
    pub fn can_recv(&self) -> bool {
        matches!(
            self,
            TcpState::Established | TcpState::FinWait1 | TcpState::FinWait2
        )
    }

    /// Check if connection is closed
    pub fn is_closed(&self) -> bool {
        matches!(self, TcpState::Closed | TcpState::TimeWait)
    }

    /// Check if connection is established
    pub fn is_established(&self) -> bool {
        matches!(self, TcpState::Established)
    }

    /// Check if connection is listening
    pub fn is_listening(&self) -> bool {
        matches!(self, TcpState::Listen)
    }
}

/// TCP Control Block (TCB) - per-connection state
pub struct TcpControlBlock {
    /// Connection state
    state: TcpState,
    /// Local address
    local_addr: SocketAddrV4,
    /// Remote address
    remote_addr: Option<SocketAddrV4>,
    /// Send sequence variables
    send: TcpSendSequence,
    /// Receive sequence variables
    recv: TcpRecvSequence,
    /// Send buffer
    send_buffer: VecDeque<u8>,
    /// Receive buffer
    recv_buffer: VecDeque<u8>,
    /// Maximum segment size
    mss: u16,
    /// Window size
    window_size: u16,
    /// Retransmission timeout (milliseconds)
    rto: u32,
    /// Round-trip time estimate (milliseconds)
    rtt: u32,
    /// Round-trip time variation (milliseconds)
    rtt_var: u32,
}

impl TcpControlBlock {
    /// Default MSS (Maximum Segment Size)
    const DEFAULT_MSS: u16 = 1460;

    /// Default window size
    const DEFAULT_WINDOW: u16 = 65535;

    /// Initial RTO (Retransmission Timeout) in milliseconds
    const INITIAL_RTO: u32 = 1000;

    /// Create new TCB
    pub fn new(local_addr: SocketAddrV4) -> Self {
        Self {
            state: TcpState::Closed,
            local_addr,
            remote_addr: None,
            send: TcpSendSequence::new(),
            recv: TcpRecvSequence::new(),
            send_buffer: VecDeque::new(),
            recv_buffer: VecDeque::new(),
            mss: Self::DEFAULT_MSS,
            window_size: Self::DEFAULT_WINDOW,
            rto: Self::INITIAL_RTO,
            rtt: 0,
            rtt_var: 0,
        }
    }

    /// Get connection state
    pub fn state(&self) -> TcpState {
        self.state
    }

    /// Set connection state
    pub fn set_state(&mut self, state: TcpState) {
        self.state = state;
    }

    /// Get local address
    pub fn local_addr(&self) -> SocketAddrV4 {
        self.local_addr
    }

    /// Get remote address
    pub fn remote_addr(&self) -> Option<SocketAddrV4> {
        self.remote_addr
    }

    /// Set remote address
    pub fn set_remote_addr(&mut self, addr: SocketAddrV4) {
        self.remote_addr = Some(addr);
    }

    /// Initialize send sequence
    pub fn init_send_sequence(&mut self, isn: u32) {
        self.send.una = isn;
        self.send.nxt = isn + 1;
        self.send.iss = isn;
    }

    /// Initialize receive sequence
    pub fn init_recv_sequence(&mut self, irs: u32) {
        self.recv.nxt = irs + 1;
        self.recv.irs = irs;
    }

    /// Add data to send buffer
    pub fn buffer_send(&mut self, data: &[u8]) -> Result<usize, TcpError> {
        if !self.state.can_send() {
            return Err(TcpError::InvalidState);
        }

        self.send_buffer.extend(data);
        Ok(data.len())
    }

    /// Get data from receive buffer
    pub fn buffer_recv(&mut self, buffer: &mut [u8]) -> Result<usize, TcpError> {
        if self.recv_buffer.is_empty() {
            return Err(TcpError::WouldBlock);
        }

        let len = buffer.len().min(self.recv_buffer.len());
        for (i, byte) in self.recv_buffer.drain(..len).enumerate() {
            buffer[i] = byte;
        }

        Ok(len)
    }

    /// Add data to receive buffer
    pub fn buffer_recv_data(&mut self, data: &[u8]) {
        self.recv_buffer.extend(data);
    }

    /// Update RTT estimate
    pub fn update_rtt(&mut self, measured_rtt: u32) {
        if self.rtt == 0 {
            // First measurement
            self.rtt = measured_rtt;
            self.rtt_var = measured_rtt / 2;
        } else {
            // RFC 6298 RTT estimation
            let delta = measured_rtt.abs_diff(self.rtt);

            self.rtt_var = (3 * self.rtt_var + delta) / 4;
            self.rtt = (7 * self.rtt + measured_rtt) / 8;
        }

        // Calculate RTO (RFC 6298)
        self.rto = self.rtt + 4 * self.rtt_var;
        if self.rto < 200 {
            self.rto = 200; // Minimum RTO of 200ms
        }
    }

    /// Get current RTO
    pub fn rto(&self) -> u32 {
        self.rto
    }

    /// Get available window size
    pub fn available_window(&self) -> u16 {
        let buffered = self.send_buffer.len() as u16;
        self.window_size.saturating_sub(buffered)
    }
}

/// TCP send sequence variables
#[derive(Debug, Clone, Copy)]
pub struct TcpSendSequence {
    /// Send unacknowledged
    una: u32,
    /// Send next
    nxt: u32,
    /// Send window
    wnd: u16,
    /// Send urgent pointer
    up: u16,
    /// Segment sequence number used for last window update
    wl1: u32,
    /// Segment acknowledgment number used for last window update
    wl2: u32,
    /// Initial send sequence number
    iss: u32,
}

impl TcpSendSequence {
    fn new() -> Self {
        Self {
            una: 0,
            nxt: 0,
            wnd: 0,
            up: 0,
            wl1: 0,
            wl2: 0,
            iss: 0,
        }
    }
}

/// TCP receive sequence variables
#[derive(Debug, Clone, Copy)]
pub struct TcpRecvSequence {
    /// Receive next
    nxt: u32,
    /// Receive window
    wnd: u16,
    /// Receive urgent pointer
    up: u16,
    /// Initial receive sequence number
    irs: u32,
}

impl TcpRecvSequence {
    fn new() -> Self {
        Self {
            nxt: 0,
            wnd: 65535,
            up: 0,
            irs: 0,
        }
    }
}

/// TCP socket implementation
pub struct TcpSocket {
    /// TCP control block
    tcb: Arc<Mutex<TcpControlBlock>>,
    /// Socket state (for Socket trait)
    socket_state: SocketState,
    /// Backlog for listening socket
    backlog: u32,
    /// Accept queue for incoming connections
    accept_queue: VecDeque<Arc<Mutex<TcpControlBlock>>>,
}

impl TcpSocket {
    /// Create new TCP socket
    pub fn new() -> Result<Self, SocketError> {
        // Allocate ephemeral port
        let port = allocate_ephemeral_port().map_err(|_| SocketError::AddrNotAvail)?;

        let local_addr = SocketAddrV4 {
            ip: [0, 0, 0, 0], // Bind to any address
            port,
        };

        let tcb = Arc::new(Mutex::new(TcpControlBlock::new(local_addr)));

        Ok(Self {
            tcb,
            socket_state: SocketState::Closed,
            backlog: 0,
            accept_queue: VecDeque::new(),
        })
    }

    /// Create TCP socket from existing TCB
    fn from_tcb(tcb: Arc<Mutex<TcpControlBlock>>) -> Self {
        Self {
            tcb,
            socket_state: SocketState::Connected,
            backlog: 0,
            accept_queue: VecDeque::new(),
        }
    }

    /// Send SYN packet to establish connection
    fn send_syn(&mut self) -> Result<(), SocketError> {
        let mut tcb = self.tcb.lock();
        let remote = tcb.remote_addr.ok_or(SocketError::NotConnected)?;

        // Generate initial sequence number
        let isn = generate_isn();
        tcb.init_send_sequence(isn);

        // Build SYN packet
        let mut header = TcpHeader::new(tcb.local_addr.port, remote.port, tcb.send.iss, 0);

        let mut flags = TcpFlags::empty();
        flags.set(TcpFlags::SYN);
        header.set_flags(flags);
        header.set_window(tcb.recv.wnd);

        // Calculate checksum
        let src_ip = Ipv4Addr(tcb.local_addr.ip);
        let dst_ip = Ipv4Addr(remote.ip);
        header.calculate_checksum(src_ip, dst_ip, &[]);

        // TODO: Send packet via network stack
        // Integration required with ipv4::send_packet() or equivalent
        // Example: ipv4::send_packet(dst_ip, IpProtocol::Tcp, &packet_data)?;
        // For now, just update state
        tcb.set_state(TcpState::SynSent);

        Ok(())
    }

    /// Send SYN-ACK packet
    fn send_syn_ack(&mut self, seq: u32, ack: u32) -> Result<(), SocketError> {
        let tcb = self.tcb.lock();
        let remote = tcb.remote_addr.ok_or(SocketError::NotConnected)?;

        // Build SYN-ACK packet
        let mut header = TcpHeader::new(tcb.local_addr.port, remote.port, seq, ack);

        let mut flags = TcpFlags::empty();
        flags.set(TcpFlags::SYN);
        flags.set(TcpFlags::ACK);
        header.set_flags(flags);
        header.set_window(tcb.recv.wnd);

        // Calculate checksum
        let src_ip = Ipv4Addr(tcb.local_addr.ip);
        let dst_ip = Ipv4Addr(remote.ip);
        header.calculate_checksum(src_ip, dst_ip, &[]);

        // TODO: Send packet via network stack
        // Integration required: Build complete IP+TCP packet and transmit
        // This requires ipv4 layer to provide send_packet() API

        Ok(())
    }

    /// Send ACK packet
    fn send_ack(&mut self) -> Result<(), SocketError> {
        let tcb = self.tcb.lock();
        let remote = tcb.remote_addr.ok_or(SocketError::NotConnected)?;

        // Build ACK packet
        let mut header =
            TcpHeader::new(tcb.local_addr.port, remote.port, tcb.send.nxt, tcb.recv.nxt);

        let mut flags = TcpFlags::empty();
        flags.set(TcpFlags::ACK);
        header.set_flags(flags);
        header.set_window(tcb.recv.wnd);

        // Calculate checksum
        let src_ip = Ipv4Addr(tcb.local_addr.ip);
        let dst_ip = Ipv4Addr(remote.ip);
        header.calculate_checksum(src_ip, dst_ip, &[]);

        // TODO: Send packet via network stack
        // Integration required: Serialize and transmit TCP ACK packet

        Ok(())
    }

    /// Send FIN packet to close connection
    fn send_fin(&mut self) -> Result<(), SocketError> {
        let tcb = self.tcb.lock();
        let remote = tcb.remote_addr.ok_or(SocketError::NotConnected)?;

        // Build FIN packet
        let mut header =
            TcpHeader::new(tcb.local_addr.port, remote.port, tcb.send.nxt, tcb.recv.nxt);

        let mut flags = TcpFlags::empty();
        flags.set(TcpFlags::FIN);
        flags.set(TcpFlags::ACK);
        header.set_flags(flags);
        header.set_window(tcb.recv.wnd);

        // Calculate checksum
        let src_ip = Ipv4Addr(tcb.local_addr.ip);
        let dst_ip = Ipv4Addr(remote.ip);
        header.calculate_checksum(src_ip, dst_ip, &[]);

        // TODO: Send packet via network stack
        // Integration required: Transmit FIN packet for connection termination

        Ok(())
    }

    /// Send data packet
    fn send_data(&mut self, data: &[u8]) -> Result<usize, SocketError> {
        let mut tcb = self.tcb.lock();

        if !tcb.state.can_send() {
            return Err(SocketError::NotConnected);
        }

        let remote = tcb.remote_addr.ok_or(SocketError::NotConnected)?;

        // Limit by MSS
        let send_len = data.len().min(tcb.mss as usize);

        // Build data packet
        let mut header =
            TcpHeader::new(tcb.local_addr.port, remote.port, tcb.send.nxt, tcb.recv.nxt);

        let mut flags = TcpFlags::empty();
        flags.set(TcpFlags::ACK);
        flags.set(TcpFlags::PSH);
        header.set_flags(flags);
        header.set_window(tcb.recv.wnd);

        // Calculate checksum
        let src_ip = Ipv4Addr(tcb.local_addr.ip);
        let dst_ip = Ipv4Addr(remote.ip);
        header.calculate_checksum(src_ip, dst_ip, &data[..send_len]);

        // Update send sequence number
        tcb.send.nxt = tcb.send.nxt.wrapping_add(send_len as u32);

        // TODO: Send packet via network stack
        // Integration required: Serialize TCP header + payload and transmit
        // Must integrate with IPv4 layer for packet encapsulation and routing

        Ok(send_len)
    }

    /// Process incoming TCP segment
    pub fn process_segment(
        &mut self,
        header: &TcpHeader,
        payload: &[u8],
        src_ip: Ipv4Addr,
        dst_ip: Ipv4Addr,
    ) -> Result<(), TcpError> {
        // Verify checksum
        if !header.verify_checksum(src_ip, dst_ip, payload) {
            return Err(TcpError::InvalidChecksum);
        }

        let mut tcb = self.tcb.lock();
        let flags = header.flags();

        match tcb.state {
            TcpState::Closed => {
                // Send RST
                Err(TcpError::InvalidState)
            }
            TcpState::Listen => {
                if flags.is_syn() && !flags.is_ack() {
                    // Received SYN, send SYN-ACK
                    let irs = header.seq_num();
                    tcb.init_recv_sequence(irs);

                    let isn = generate_isn();
                    tcb.init_send_sequence(isn);

                    tcb.set_state(TcpState::SynReceived);
                    Ok(())
                } else {
                    Err(TcpError::InvalidState)
                }
            }
            TcpState::SynSent => {
                if flags.is_syn() && flags.is_ack() {
                    // Received SYN-ACK
                    let ack = header.ack_num();
                    if ack == tcb.send.nxt {
                        let irs = header.seq_num();
                        tcb.init_recv_sequence(irs);
                        tcb.send.una = ack;
                        tcb.set_state(TcpState::Established);
                        Ok(())
                    } else {
                        Err(TcpError::InvalidSequence)
                    }
                } else if flags.is_syn() {
                    // Simultaneous open
                    let irs = header.seq_num();
                    tcb.init_recv_sequence(irs);
                    tcb.set_state(TcpState::SynReceived);
                    Ok(())
                } else {
                    Err(TcpError::InvalidState)
                }
            }
            TcpState::SynReceived => {
                if flags.is_ack() && !flags.is_syn() {
                    // Received ACK, connection established
                    let ack = header.ack_num();
                    if ack == tcb.send.nxt {
                        tcb.send.una = ack;
                        tcb.set_state(TcpState::Established);
                        Ok(())
                    } else {
                        Err(TcpError::InvalidSequence)
                    }
                } else {
                    Err(TcpError::InvalidState)
                }
            }
            TcpState::Established => {
                if flags.is_fin() {
                    // Received FIN, go to CLOSE_WAIT
                    tcb.recv.nxt = header.seq_num().wrapping_add(1);
                    tcb.set_state(TcpState::CloseWait);
                    Ok(())
                } else if flags.is_ack() {
                    // Update unacknowledged sequence
                    tcb.send.una = header.ack_num();

                    // Process data
                    if !payload.is_empty() {
                        let seq = header.seq_num();
                        if seq == tcb.recv.nxt {
                            tcb.buffer_recv_data(payload);
                            tcb.recv.nxt = tcb.recv.nxt.wrapping_add(payload.len() as u32);
                        }
                    }
                    Ok(())
                } else {
                    Ok(())
                }
            }
            TcpState::FinWait1 => {
                if flags.is_fin() && flags.is_ack() {
                    // Received FIN+ACK
                    tcb.recv.nxt = header.seq_num().wrapping_add(1);
                    tcb.send.una = header.ack_num();
                    tcb.set_state(TcpState::TimeWait);
                    Ok(())
                } else if flags.is_fin() {
                    // Received FIN only
                    tcb.recv.nxt = header.seq_num().wrapping_add(1);
                    tcb.set_state(TcpState::Closing);
                    Ok(())
                } else if flags.is_ack() {
                    // Received ACK only
                    tcb.send.una = header.ack_num();
                    tcb.set_state(TcpState::FinWait2);
                    Ok(())
                } else {
                    Err(TcpError::InvalidState)
                }
            }
            TcpState::FinWait2 => {
                if flags.is_fin() {
                    // Received FIN
                    tcb.recv.nxt = header.seq_num().wrapping_add(1);
                    tcb.set_state(TcpState::TimeWait);
                    Ok(())
                } else {
                    Ok(())
                }
            }
            TcpState::CloseWait => {
                // Waiting for application to close
                Ok(())
            }
            TcpState::Closing => {
                if flags.is_ack() {
                    // Received ACK of our FIN
                    tcb.send.una = header.ack_num();
                    tcb.set_state(TcpState::TimeWait);
                    Ok(())
                } else {
                    Err(TcpError::InvalidState)
                }
            }
            TcpState::LastAck => {
                if flags.is_ack() {
                    // Received ACK of our FIN
                    tcb.send.una = header.ack_num();
                    tcb.set_state(TcpState::Closed);
                    Ok(())
                } else {
                    Err(TcpError::InvalidState)
                }
            }
            TcpState::TimeWait => {
                // Wait for all packets to expire
                Ok(())
            }
        }
    }
}

impl Socket for TcpSocket {
    fn bind(&mut self, addr: SocketAddr) -> Result<(), SocketError> {
        match addr {
            SocketAddr::V4(addr_v4) => {
                let mut tcb = self.tcb.lock();
                if tcb.state != TcpState::Closed {
                    return Err(SocketError::AlreadyConnected);
                }

                // Check if port is available
                bind_port(addr_v4.port).map_err(|_| SocketError::AddrInUse)?;

                // Update local address
                tcb.local_addr = addr_v4;
                self.socket_state = SocketState::Bound;
                Ok(())
            }
            _ => Err(SocketError::NotSupported),
        }
    }

    fn listen(&mut self, backlog: u32) -> Result<(), SocketError> {
        let mut tcb = self.tcb.lock();
        if tcb.state != TcpState::Closed {
            return Err(SocketError::InvalidArg);
        }

        tcb.set_state(TcpState::Listen);
        self.socket_state = SocketState::Listening;
        self.backlog = backlog;
        Ok(())
    }

    fn accept(&mut self) -> Result<Arc<Mutex<dyn Socket>>, SocketError> {
        if self.socket_state != SocketState::Listening {
            return Err(SocketError::InvalidArg);
        }

        // Check accept queue
        if let Some(new_tcb) = self.accept_queue.pop_front() {
            let new_socket = TcpSocket::from_tcb(new_tcb);
            Ok(Arc::new(Mutex::new(new_socket)))
        } else {
            Err(SocketError::WouldBlock)
        }
    }

    fn connect(&mut self, addr: SocketAddr) -> Result<(), SocketError> {
        match addr {
            SocketAddr::V4(addr_v4) => {
                let mut tcb = self.tcb.lock();
                if tcb.state != TcpState::Closed {
                    return Err(SocketError::AlreadyConnected);
                }

                tcb.set_remote_addr(addr_v4);
                drop(tcb); // Release lock before calling send_syn

                // Initiate 3-way handshake
                self.send_syn()?;
                self.socket_state = SocketState::Connecting;

                Ok(())
            }
            _ => Err(SocketError::NotSupported),
        }
    }

    fn send(&mut self, data: &[u8], _flags: u32) -> Result<usize, SocketError> {
        let tcb = self.tcb.lock();
        if !tcb.state.can_send() {
            return Err(SocketError::NotConnected);
        }
        drop(tcb);

        // Send data in segments
        let mut total_sent = 0;
        while total_sent < data.len() {
            let sent = self.send_data(&data[total_sent..])?;
            total_sent += sent;
        }

        Ok(total_sent)
    }

    fn recv(&mut self, buffer: &mut [u8], _flags: u32) -> Result<usize, SocketError> {
        let mut tcb = self.tcb.lock();
        if !tcb.state.can_recv() && tcb.recv_buffer.is_empty() {
            return Err(SocketError::NotConnected);
        }

        tcb.buffer_recv(buffer).map_err(|e| match e {
            TcpError::WouldBlock => SocketError::WouldBlock,
            _ => SocketError::Other,
        })
    }

    fn sendto(
        &mut self,
        _data: &[u8],
        _addr: SocketAddr,
        _flags: u32,
    ) -> Result<usize, SocketError> {
        // TCP doesn't support sendto
        Err(SocketError::NotSupported)
    }

    fn recvfrom(
        &mut self,
        _buffer: &mut [u8],
        _flags: u32,
    ) -> Result<(usize, SocketAddr), SocketError> {
        // TCP doesn't support recvfrom
        Err(SocketError::NotSupported)
    }

    fn shutdown(&mut self, how: ShutdownHow) -> Result<(), SocketError> {
        let tcb = self.tcb.lock();
        let state = tcb.state;
        drop(tcb);

        match how {
            ShutdownHow::Read => {
                // Close receive side
                self.socket_state = SocketState::Closing;
                Ok(())
            }
            ShutdownHow::Write => {
                // Close send side - send FIN
                if state.can_send() {
                    self.send_fin()?;
                    let mut tcb = self.tcb.lock();
                    tcb.set_state(TcpState::FinWait1);
                    self.socket_state = SocketState::Closing;
                }
                Ok(())
            }
            ShutdownHow::Both => {
                // Close both sides
                if state.can_send() {
                    self.send_fin()?;
                    let mut tcb = self.tcb.lock();
                    match state {
                        TcpState::Established => tcb.set_state(TcpState::FinWait1),
                        TcpState::CloseWait => tcb.set_state(TcpState::LastAck),
                        _ => {}
                    }
                    self.socket_state = SocketState::Closing;
                }
                Ok(())
            }
        }
    }

    fn close(&mut self) -> Result<(), SocketError> {
        let mut tcb = self.tcb.lock();
        let local_port = tcb.local_addr.port;

        // Close connection
        match tcb.state {
            TcpState::Listen | TcpState::Closed => {
                tcb.set_state(TcpState::Closed);
            }
            TcpState::SynSent | TcpState::SynReceived => {
                // TODO: Send RST
                tcb.set_state(TcpState::Closed);
            }
            _ => {
                // Connection established, need proper close
                drop(tcb);
                self.shutdown(ShutdownHow::Both)?;
                return Ok(());
            }
        }

        // Release port
        release_port(local_port);
        self.socket_state = SocketState::Closed;

        Ok(())
    }

    fn state(&self) -> SocketState {
        self.socket_state
    }

    fn setsockopt(&mut self, _option: SocketOption) -> Result<(), SocketError> {
        // TODO: Implement socket options
        Err(SocketError::NotSupported)
    }

    fn getsockopt(&self, _option: SocketOptionType) -> Result<SocketOption, SocketError> {
        // TODO: Implement socket options
        Err(SocketError::NotSupported)
    }

    fn local_addr(&self) -> Option<SocketAddr> {
        let tcb = self.tcb.lock();
        Some(SocketAddr::V4(tcb.local_addr))
    }

    fn peer_addr(&self) -> Option<SocketAddr> {
        let tcb = self.tcb.lock();
        tcb.remote_addr.map(SocketAddr::V4)
    }
}

/// TCP connection identifier (4-tuple)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TcpConnectionId {
    pub local_ip: Ipv4Addr,
    pub local_port: u16,
    pub remote_ip: Ipv4Addr,
    pub remote_port: u16,
}

/// TCP connection manager
pub struct TcpConnectionManager {
    /// Active connections
    connections: BTreeMap<TcpConnectionId, Arc<Mutex<TcpControlBlock>>>,
    /// Listening sockets
    listeners: BTreeMap<u16, Arc<Mutex<TcpSocket>>>,
}

impl TcpConnectionManager {
    /// Create new connection manager
    pub fn new() -> Self {
        Self {
            connections: BTreeMap::new(),
            listeners: BTreeMap::new(),
        }
    }

    /// Add connection
    pub fn add_connection(&mut self, id: TcpConnectionId, tcb: Arc<Mutex<TcpControlBlock>>) {
        self.connections.insert(id, tcb);
    }

    /// Get connection
    pub fn get_connection(&self, id: &TcpConnectionId) -> Option<Arc<Mutex<TcpControlBlock>>> {
        self.connections.get(id).cloned()
    }

    /// Remove connection
    pub fn remove_connection(
        &mut self,
        id: &TcpConnectionId,
    ) -> Option<Arc<Mutex<TcpControlBlock>>> {
        self.connections.remove(id)
    }

    /// Add listener
    pub fn add_listener(&mut self, port: u16, socket: Arc<Mutex<TcpSocket>>) {
        self.listeners.insert(port, socket);
    }

    /// Get listener
    pub fn get_listener(&self, port: u16) -> Option<Arc<Mutex<TcpSocket>>> {
        self.listeners.get(&port).cloned()
    }

    /// Remove listener
    pub fn remove_listener(&mut self, port: u16) -> Option<Arc<Mutex<TcpSocket>>> {
        self.listeners.remove(&port)
    }
}

impl Default for TcpConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Global TCP connection manager
static TCP_CONNECTION_MANAGER: Mutex<TcpConnectionManager> = Mutex::new(TcpConnectionManager {
    connections: BTreeMap::new(),
    listeners: BTreeMap::new(),
});

/// TCP port range for ephemeral ports
pub const TCP_PORT_MIN: u16 = 32768;
pub const TCP_PORT_MAX: u16 = 60999;

/// TCP port manager
pub struct TcpPortManager {
    /// Next ephemeral port
    next_port: AtomicU16,
    /// Bound ports
    bound_ports: BTreeMap<u16, ()>,
}

impl TcpPortManager {
    /// Create new port manager
    pub fn new() -> Self {
        Self {
            next_port: AtomicU16::new(TCP_PORT_MIN),
            bound_ports: BTreeMap::new(),
        }
    }

    /// Allocate ephemeral port
    pub fn allocate_ephemeral(&mut self) -> Result<u16, TcpError> {
        let start = self.next_port.load(Ordering::Relaxed);
        let mut port = start;

        loop {
            if let btree_map::Entry::Vacant(entry) = self.bound_ports.entry(port) {
                entry.insert(());
                self.next_port.store(
                    if port == TCP_PORT_MAX {
                        TCP_PORT_MIN
                    } else {
                        port + 1
                    },
                    Ordering::Relaxed,
                );
                return Ok(port);
            }

            port = if port == TCP_PORT_MAX {
                TCP_PORT_MIN
            } else {
                port + 1
            };

            if port == start {
                return Err(TcpError::NoPortsAvailable);
            }
        }
    }

    /// Bind specific port
    pub fn bind(&mut self, port: u16) -> Result<(), TcpError> {
        if self.bound_ports.contains_key(&port) {
            return Err(TcpError::PortInUse);
        }

        self.bound_ports.insert(port, ());
        Ok(())
    }

    /// Release port
    pub fn release(&mut self, port: u16) {
        self.bound_ports.remove(&port);
    }

    /// Check if port is bound
    pub fn is_bound(&self, port: u16) -> bool {
        self.bound_ports.contains_key(&port)
    }
}

impl Default for TcpPortManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Global TCP port manager
static TCP_PORT_MANAGER: Mutex<TcpPortManager> = Mutex::new(TcpPortManager {
    next_port: AtomicU16::new(TCP_PORT_MIN),
    bound_ports: BTreeMap::new(),
});

/// Allocate ephemeral port
pub fn allocate_ephemeral_port() -> Result<u16, TcpError> {
    TCP_PORT_MANAGER.lock().allocate_ephemeral()
}

/// Bind port
pub fn bind_port(port: u16) -> Result<(), TcpError> {
    TCP_PORT_MANAGER.lock().bind(port)
}

/// Release port
pub fn release_port(port: u16) {
    TCP_PORT_MANAGER.lock().release(port);
}

/// Check if port is bound
pub fn is_port_bound(port: u16) -> bool {
    TCP_PORT_MANAGER.lock().is_bound(port)
}

/// Generate initial sequence number
///
/// # Security Note
///
/// This implementation uses a simple counter-based approach which is NOT
/// cryptographically secure and vulnerable to sequence prediction attacks.
/// RFC 6528 requires cryptographically random ISNs to prevent TCP hijacking.
///
/// TODO: Implement secure ISN generation using:
/// - Hardware random number generator (RDRAND instruction)
/// - ChaCha20 or similar CSPRNG
/// - ISN = hash(src_ip, src_port, dst_ip, dst_port, timestamp, secret)
///
/// For production use, this MUST be replaced with a secure implementation.
fn generate_isn() -> u32 {
    static ISN_COUNTER: AtomicU32 = AtomicU32::new(1000);
    // Increment by a large value to make prediction harder (but still not secure)
    ISN_COUNTER.fetch_add(64000, Ordering::Relaxed)
}

/// Process incoming TCP packet
pub fn process_packet(
    ip_payload: &[u8],
    src_ip: Ipv4Addr,
    dst_ip: Ipv4Addr,
) -> Result<(), TcpError> {
    let header = TcpHeader::parse(ip_payload)?;
    let payload_offset = header.data_offset();
    let payload = &ip_payload[payload_offset..];

    // Verify checksum
    if !header.verify_checksum(src_ip, dst_ip, payload) {
        return Err(TcpError::InvalidChecksum);
    }

    // Look up connection
    let conn_id = TcpConnectionId {
        local_ip: dst_ip,
        local_port: header.dst_port(),
        remote_ip: src_ip,
        remote_port: header.src_port(),
    };

    let manager = TCP_CONNECTION_MANAGER.lock();

    if let Some(tcb) = manager.get_connection(&conn_id) {
        // Existing connection
        drop(manager);
        let mut socket = TcpSocket::from_tcb(tcb);
        socket.process_segment(&header, payload, src_ip, dst_ip)?;
    } else if let Some(listener) = manager.get_listener(header.dst_port()) {
        // Listening socket
        drop(manager);
        let mut listener = listener.lock();
        listener.process_segment(&header, payload, src_ip, dst_ip)?;
    } else {
        // No matching connection or listener
        // TODO: Send RST
        return Err(TcpError::ConnectionNotFound);
    }

    Ok(())
}

/// TCP errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TcpError {
    /// Packet too short
    TooShort,
    /// Buffer too small
    BufferTooSmall,
    /// Invalid header length
    InvalidHeaderLength,
    /// Invalid checksum
    InvalidChecksum,
    /// Invalid sequence number
    InvalidSequence,
    /// Invalid state for operation
    InvalidState,
    /// Port in use
    PortInUse,
    /// No ports available
    NoPortsAvailable,
    /// Connection not found
    ConnectionNotFound,
    /// Operation would block
    WouldBlock,
}

/// TCP subsystem initialized flag
static TCP_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Initialize TCP subsystem
pub fn init() {
    if TCP_INITIALIZED.load(Ordering::Acquire) {
        return;
    }

    TCP_INITIALIZED.store(true, Ordering::Release);
}

/// Check if TCP subsystem is initialized
pub fn is_initialized() -> bool {
    TCP_INITIALIZED.load(Ordering::Acquire)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tcp_header() {
        let header = TcpHeader::new(12345, 80, 1000, 2000);
        assert_eq!(header.src_port(), 12345);
        assert_eq!(header.dst_port(), 80);
        assert_eq!(header.seq_num(), 1000);
        assert_eq!(header.ack_num(), 2000);
        assert_eq!(header.data_offset(), 20);
    }

    #[test]
    fn test_tcp_flags() {
        let mut flags = TcpFlags::empty();
        assert!(!flags.is_syn());

        flags.set(TcpFlags::SYN);
        assert!(flags.is_syn());

        flags.set(TcpFlags::ACK);
        assert!(flags.is_syn());
        assert!(flags.is_ack());

        flags.clear(TcpFlags::SYN);
        assert!(!flags.is_syn());
        assert!(flags.is_ack());
    }

    #[test]
    fn test_tcp_state() {
        assert!(TcpState::Established.can_send());
        assert!(TcpState::Established.can_recv());
        assert!(!TcpState::Closed.can_send());
        assert!(!TcpState::Closed.can_recv());
        assert!(TcpState::Closed.is_closed());
    }

    #[test]
    fn test_tcp_checksum() {
        let src_ip = Ipv4Addr::new(192, 168, 1, 1);
        let dst_ip = Ipv4Addr::new(192, 168, 1, 2);
        let payload = b"Hello, TCP!";

        let mut header = TcpHeader::new(12345, 80, 1000, 2000);
        header.calculate_checksum(src_ip, dst_ip, payload);

        assert_ne!(header.checksum(), 0);
        assert!(header.verify_checksum(src_ip, dst_ip, payload));
    }

    #[test]
    fn test_tcp_port_manager() {
        let mut manager = TcpPortManager::new();

        // Allocate ephemeral port
        let port1 = manager.allocate_ephemeral().unwrap();
        assert!(port1 >= TCP_PORT_MIN && port1 <= TCP_PORT_MAX);
        assert!(manager.is_bound(port1));

        // Bind specific port
        let port2 = 8080;
        manager.bind(port2).unwrap();
        assert!(manager.is_bound(port2));

        // Try to bind same port again
        assert_eq!(manager.bind(port2), Err(TcpError::PortInUse));

        // Release port
        manager.release(port1);
        assert!(!manager.is_bound(port1));
    }

    #[test]
    fn test_tcp_control_block() {
        let local_addr = SocketAddrV4 {
            ip: [192, 168, 1, 1],
            port: 12345,
        };

        let mut tcb = TcpControlBlock::new(local_addr);
        assert_eq!(tcb.state(), TcpState::Closed);

        tcb.set_state(TcpState::Established);
        assert!(tcb.state().can_send());
        assert!(tcb.state().can_recv());

        // Test sequence initialization
        tcb.init_send_sequence(1000);
        assert_eq!(tcb.send.iss, 1000);
        assert_eq!(tcb.send.una, 1000);
        assert_eq!(tcb.send.nxt, 1001);

        tcb.init_recv_sequence(2000);
        assert_eq!(tcb.recv.irs, 2000);
        assert_eq!(tcb.recv.nxt, 2001);
    }

    #[test]
    fn test_connection_id() {
        let id1 = TcpConnectionId {
            local_ip: Ipv4Addr::new(192, 168, 1, 1),
            local_port: 12345,
            remote_ip: Ipv4Addr::new(192, 168, 1, 2),
            remote_port: 80,
        };

        let id2 = TcpConnectionId {
            local_ip: Ipv4Addr::new(192, 168, 1, 1),
            local_port: 12345,
            remote_ip: Ipv4Addr::new(192, 168, 1, 2),
            remote_port: 80,
        };

        assert_eq!(id1, id2);
    }
}
