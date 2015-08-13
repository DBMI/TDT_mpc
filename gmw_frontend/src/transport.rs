#![allow(dead_code)]
use std::net::{TcpStream, TcpListener};
use std::io::{Write, Read};
use std::borrow::{Borrow, BorrowMut};
use std::sync::{Arc, Mutex};
use std::thread;

const HOST: usize = 0;

fn handle_client(stream: TcpStream) {
    let mut buff: Vec<u8> = vec![0; 100];
    let mut stream = stream;
    stream.read_to_end(&mut buff).ok();
    println!("Received {:?}", buff);
}

pub trait ProtocolTransport {
    fn init(this_party: usize, sockets: Vec<(usize, String)>);
    fn send_to_party<T>(&mut self, data: &T, party: usize) 
        where T: Borrow<[u8]>;
    fn recv_from_party<T>(&mut self, data: &mut T, party: usize) 
        where T: BorrowMut<[u8]>;
    fn broadcast<T>(data: &T) where T: Borrow<[u8]>;
    fn terminate(&mut self);
}

pub struct TCP {
    this_party: usize,
    send_streams: Vec<TcpStream>,
    recv_streams: Vec<TcpStream>
}

impl TCP {
    pub fn init(this_party: usize, send_sockets: Vec<String>, 
                recv_sockets: Vec<String>) -> Self 
    {
        let num_others = send_sockets.len();
        let mut send_streams = Vec::new();
        let recv_streams = Arc::new(Mutex::new(Some(Vec::new())));
        let mut send_sockets_iter = send_sockets.iter();
        let mut threads = Vec::new();
        for (order, recv_socket) in (0..num_others).zip(recv_sockets.into_iter()) {
            let recv_streams = recv_streams.clone();
            threads.push(thread::spawn(move || {
                let listener = TcpListener::bind(&recv_socket as &str)
                                                .unwrap();
                let mut recv_streams = recv_streams.lock().unwrap();
                let mut recv_streams = recv_streams.as_mut().unwrap();
                recv_streams.push((order, listener.accept().unwrap().0));
            }));
        }

        for _ in 0..this_party {
            send_streams.push(TcpStream::connect(
                    &send_sockets_iter.next().unwrap() as &str).unwrap()); 
        } 
        for thread in threads {
            thread.join().ok();
        }
        for _ in (this_party+1)..(num_others+1) {
            send_streams.push(TcpStream::connect(
                    &send_sockets_iter.next().unwrap() as &str).unwrap()); 
        } 

        let mut recv_streams = recv_streams.lock().unwrap();
        let mut recv_streams = recv_streams.take().unwrap();
        recv_streams.sort_by(|a,b| a.0.cmp(&b.0));
        let recv_streams = recv_streams.into_iter().map(|x| x.1)
                                       .collect::<Vec<_>>();
        TCP { this_party: this_party, send_streams: send_streams, 
              recv_streams: recv_streams}
    }

    fn send_tcp_stream<T>(send_stream: &mut TcpStream, data: &T) 
        where T: Borrow<[u8]> 
    {
        match send_stream.write_all(data.borrow()) {
            Err(e) => panic!("Error: Unable to transport data to {:?}. [{:?}]",
                             send_stream.peer_addr().ok().unwrap(), e),
            _ => ()
        };
        send_stream.flush().ok();
    }
    
    pub fn send<T>(&mut self, data: &T, dest_party: usize) 
        where T: Borrow<[u8]> 
    {
        if dest_party==self.this_party { 
            panic!("Error: Attempt to send to an invalid party {}.", 
                   dest_party) 
        }
        let index = self.get_index(dest_party);
        Self::send_tcp_stream(&mut self.send_streams[index], data);
    }

    pub fn broadcast<T>(&mut self, data: &T) 
        where T: Borrow<[u8]> 
    {
        for stream in &mut self.send_streams {
            Self::send_tcp_stream(stream, data);
        }
    }

    pub fn recv<T>(&mut self, data: &mut T, src_party: usize) 
        where T: BorrowMut<[u8]> 
    {
        let index = self.get_index(src_party);
        let len = data.borrow_mut().len();
        let mut buffer = Vec::new();
        self.recv_streams.get_mut(index).unwrap().take(len as u64)
                         .read_to_end(&mut buffer).ok();
        if len != buffer.len() {
            panic!("Incompleted transmission.");
        }
        for i in 0..len {
           *data.borrow_mut().get_mut(i).unwrap()=buffer[i];
        } 
    }

    fn get_index(&self, party: usize) -> usize {
        if party < self.this_party {
            party
        }
        else if party > self.this_party {
           party-1 
        }
        else {
            panic!("Error: Party {} is this party.");
        }
    }
}



#[cfg(test)]
mod test {
    use super::*; 
    use std::thread;
    use std::thread::sleep_ms;
    #[test]
    fn test_host_join() {
        let sockets0_recv = vec!["localhost:1234".to_string(), 
                                   "localhost:1235".to_string()];
        let sockets0_send = vec!["localhost:1236".to_string(), 
                                   "localhost:1238".to_string()];
        let p0_thread = thread::spawn(move || {
            let mut p0 = TCP::init(0, sockets0_send, sockets0_recv);
            let mut buffer = vec![0; 8];
            p0.broadcast(&"Hello! 0".to_string().into_bytes());
            p0.recv(&mut buffer, 1);
            assert_eq!(String::from_utf8(buffer.clone()).unwrap(), 
                       "Hello! 1".to_string());
            p0.recv(&mut buffer, 2);
            assert_eq!(String::from_utf8(buffer.clone()).unwrap(), 
                       "Hello! 2".to_string());
        });
        sleep_ms(50);
        let sockets1_recv = vec!["localhost:1236".to_string(), 
                                   "localhost:1237".to_string()];
        let sockets1_send = vec!["localhost:1234".to_string(), 
                                   "localhost:1239".to_string()];
        let p1_thread = thread::spawn(move || {
            let mut p1 = TCP::init(1, sockets1_send, sockets1_recv);
            let mut buffer = vec![0; 8];
            p1.recv(&mut buffer, 0);
            assert_eq!(String::from_utf8(buffer.clone()).unwrap(), 
                       "Hello! 0".to_string());
            p1.broadcast(&"Hello! 1".to_string().into_bytes());
            p1.recv(&mut buffer, 2);
            assert_eq!(String::from_utf8(buffer.clone()).unwrap(), 
                       "Hello! 2".to_string());
        });
        sleep_ms(50);
        let sockets2_recv = vec!["localhost:1238".to_string(), 
                                   "localhost:1239".to_string()];
        let sockets2_send = vec!["localhost:1235".to_string(), 
                                   "localhost:1237".to_string()];
        let p2_thread = thread::spawn(move || {
            let mut p2 = TCP::init(2, sockets2_send, sockets2_recv);
            let mut buffer = vec![0; 8];
            p2.recv(&mut buffer, 0);
            assert_eq!(String::from_utf8(buffer.clone()).unwrap(), 
                       "Hello! 0".to_string());
            p2.recv(&mut buffer, 1);
            assert_eq!(String::from_utf8(buffer).unwrap(), 
                       "Hello! 1".to_string());
            p2.broadcast(&"Hello! 2".to_string().into_bytes());
        });
        p0_thread.join().ok();
        p1_thread.join().ok();
        p2_thread.join().ok();
    }
}
