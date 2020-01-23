use zmq::{Context, SocketType};

use futures::{SinkExt, StreamExt};
use tmq::{request, Result, Multipart};
use utils::{
    generate_tcp_address, msg, sync_echo,
};

mod utils;

#[tokio::test]
async fn single_message() -> Result<()> {
    let address = generate_tcp_address();
    let ctx = Context::new();
    let mut sock = request(&ctx).connect(&address)?.finish()?;

    let echo = sync_echo(address, SocketType::REP, 1);

    let m1 = "Msg";
    let m2 = "Msg (contd.)";
    sock.send(vec![msg(m1.as_bytes()), msg(m2.as_bytes())]).await?;
    if let Some(multipart) = sock.next().await {
        let multipart = multipart?;
        let expected: Multipart = vec![msg(m1.as_bytes()), msg(m2.as_bytes())].into();
        assert_eq!(expected, multipart);
    } else {
        panic!("Reply is missing.");
    }

    echo.join().unwrap();

    Ok(())
}

// Note that equivalent non-async code produces an error:
//
// ```
// use zmq::{Context};
// fn main() {
//     let req = Context::new().socket(zmq::SocketType::REQ).unwrap();
//     req.connect("tcp://127.0.0.1:7897").expect("connect");
//     assert!(req.send("str",0).is_ok());
//     assert!(req.send("str",0).is_err());
// }
// #[tokio::test] // disabled due to hang rather than error
async fn send_2x_is_err() -> Result<()> {
    let address = generate_tcp_address();
    let ctx = Context::new();
    let mut sock = request(&ctx).connect(&address)?.finish()?;

    sock.send(vec![msg(b"Msg")]).await?;
    let res = sock.send(vec![msg(b"Msg")]).await;
    assert!(res.is_err());

    Ok(())
}

// Note that equivalent non-async code produces an error:
//
// ```
// use zmq::{Context};
// fn main() {
//     let req = Context::new().socket(zmq::SocketType::REQ).unwrap();
//     req.connect("tcp://127.0.0.1:7897").expect("connect");
//     assert!(req.recv(0).is_err());
// }
// #[tokio::test] // disabled due to hang rather than error
async fn recv_first_is_err() -> Result<()> {
    let address = generate_tcp_address();
    let ctx = Context::new();
    let mut sock = request(&ctx).connect(&address)?.finish()?;

    let res = sock.next().await.unwrap();
    assert!(res.is_err());

    Ok(())
}


#[tokio::test]
async fn request_hammer() -> Result<()> {
    let address = generate_tcp_address();
    let ctx = Context::new();
    let mut sock = request(&ctx).connect(&address)?.finish()?;

    let count = 1_000;
    let echo = sync_echo(address, SocketType::REP, count);

    for i in 0..count {
        let m1 = format!("Msg #{}", i);
        let m2 = format!("Msg #{} (contd.)", i);
        sock.send(vec![msg(m1.as_bytes()), msg(m2.as_bytes())])
            .await?;
        if let Some(multipart) = sock.next().await {
            let multipart = multipart?;

            let expected: Multipart = vec![msg(m1.as_bytes()), msg(m2.as_bytes())].into();
            assert_eq!(expected, multipart);
        } else {
            panic!("Reply in stream is missing.");
        }
    }

    echo.join().unwrap();

    Ok(())
}