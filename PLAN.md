Apparently we would have to make our own multithreaded ring buffer.

shit.

multiqueue is dead
mpmc is useles because it's not broadcast

Buffer and sequencer must be publicly readable.

ringbuffer is neat but does things for us that we do not want.

Sequencer: can probably be integer n < Buffer.len 
    Atomic BS?

can Condvar be used by UDPlistener to ping when data updates?
    in effect: Can a thread be busy for three pings and then wait for the 4th?

multithread utopia:

UdpSocket listener writes to log, pings sequencer system somehow

TcpStream writer listens to sequencer pings, reads log. tracks own position and core sequencer. deals with slow client desync that way?
    
    own pos + core sequencer means we can know if we are in sync.
    We actually do not care if we are in sync, we only care that we do not read ahead of the write position.
    
        gives weird behavior if > 1 full buffer is written between tcp sends, but that is ok.
            Slow clients can't do live log.
            High output systems require larger buffers

TcpSocket
    


the crazy(?) idea:
    
    UDP listener grabs atomic lock of buffer element, uses it in the socket read.
    same thing as writing manually, right?
    
    => 

    UDP listener owns the buffer and the writing position.
    other threads has references to these

    StreamWriter pushes own_pos..ref_pos of buffer onto stream then checks ref_pos vs own_pos again. if matched, start waiting for update signal.
    



can send:
&'a [u8]
tcpStream

can not send:
[u8]
Sender
