use tokio::{
    io::{ 
        AsyncBufReadExt,
        AsyncWriteExt,
        BufReader,
    },
    net::TcpListener,
    sync::broadcast,
};

//turbofish example
// fn give_me_default<T>() -> T where T: Default {
//     // default is a trait that is defined in the rust binary
//     // some types have a default value: integers, boolean
//     // default() makes it easier to provide a default value for your types
//     Default::default()
// }

// procedural macro available from tokio main
// it takes our async main function, and turns into a normal function with tokio features added
// it saves us some boilerplate code
#[tokio::main]
async fn main() {
    // calling give_me_default() such as below will give us a compiler error due to the fact it can return anythin
    // let value = give_me_default();
    // we can solve this by adding a type annotation into the variable binding, but sometimes that doesnt work
    // let value:i32 = give_me_default();
    // we can usually solve this error by using a turbofish operator which is :: folowed by <SOME_TYPE> example below
    // let value = give_me_default::<i32>();
    // turbofish is used to solve the problem of disambiguating the type return from such function that the compiler is not smart enough to solve

    // await is a rust keyword that tells the rust compiler to suspend the function running until the future resolves
    // tcp listener
    let listener = TcpListener::bind("localhost:0000").await.unwrap();
    let (tx, _rx) = broadcast::channel(10);
    // call accept method on tcp listener
    // accept() is a method that accepts a new connection from a tcp listener and yields the connection as well as the address of the connection, 
    // similar to bind, accept() returns a future and that future outputs a result
    // this outer infinite loop allows us to have new clients join our server, however as it is, this solution blocks at the task level
    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();
        let tx = tx.clone();
        let mut rx = tx.subscribe();
        // async move - is an async block, wraps the code into a separate future
        tokio::spawn(async move {
           let (reader, mut writer) = socket.split();
            // tokio supplies us with BuffReader
            // a buff reader wraps any kind of reader and maintains its own buffer
            // and it allows you to run some higher order operations such as reading an entire line of text from a stream
            let mut reader = BufReader::new(reader);
            // string creation
            let mut line = String::new();
            // this inner infinite loop allows us to keep the connection alive after a message has been written
            loop {
                // select - also a golang concept, allows us to run multiple asynchrounous processes concurrently,
                // and act on the first one that returns a result
                // it has its own syntax due to its nature as a macro
                // it requires an identifier, a future, and then its own block of code
                // it will first run the future, it will assign the result of the future to the identifier that you give it 
                // and then it will run the block of code you give it.
                tokio::select!{
                    result = reader.read_line(&mut line) => {
                        if result.unwrap() == 0 {
                            break;
                        }
                        tx.send((line.clone(), addr)).unwrap();
                        line.clear();
                    }
                    result = rx.recv() => {
                        let (msg, other_addr) = result.unwrap();

                        if addr != other_addr {
                            writer.write_all(&msg.as_bytes()).await.unwrap();
                        }
                    }
                }
                // define buffer in the form of a stack array
                // 0u8;1024 is about one kilobyte
                // 1024 bytes
                // this is not a great approach to use as we have to constantly manage it
                // let mut buffer = [0u8; 1024];
                // async function, suspends function until read is done and then it will unwrap the results
                // socket.read() returns the number of bytes that were from the stream onto the buffer
                // we may receive less bytes than the size we set on our buffer so we use bytes_read to truncate that response
                // let bytes_read = socket.read(&mut buffer).await.unwrap();
                // let bytes_read = reader.read_line(&mut line).await.unwrap();
                // theres a bug above, when we call read_line, it pins the line before above the new message
                // it is not read_lines job to clear out the input buffer
                // if bytes_read == 0 {
                //     break;
                // }
                // tx.send(line.clone()).unwrap();
                // let msg = rx.recv().await.unwrap();
                // write_all() does not write a message to every single socket that is connected to a TCP listener, it 
                // instead it writes every single byte that is in the input buffer out to the output buffer
                // socket.write_all(&buffer[..bytes_read]).await.unwrap();
                // writer.write_all(&msg.as_bytes()).await.unwrap();
                // clear() - clears out the input buffer
                // line.clear();
            } 
        });
    }
}
// a future is a value that does not have a known value yet but may have a known value at some point in the future
// rust does not know how to execute a future but knows how to generate them

// tokio spawn vs tokio select
// rule of thumb, select is very useful when you need things to operate on the same shared state and you have a finite number of things
//
// select is better in this case, we only have two tasks that need to run concurrently
//  tokio::select!{
//        result = reader.read_line(&mut line) => {
//            if result.unwrap() == 0 {
//                break;
//            }
//            tx.send((line.clone(), addr)).unwrap();
//            line.clear();
//        }
//        result = rx.recv() => {
//            let (msg, other_addr) = result.unwrap();
//
//            if addr != other_addr {
//                writer.write_all(&msg.as_bytes()).await.unwrap();
//            }
//        }
//  }