use std::io::ErrorKind;
use std::process::exit;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

const F_IN: &'static str = "/f_in";
const G_IN: &'static str = "/g_in";
const F_OUT: &'static str = "/f_out";
const G_OUT: &'static str = "/g_out";

fn clean_up() {
    let _ = posixmq::remove_queue(F_IN);
    let _ = posixmq::remove_queue(G_IN);
    let _ = posixmq::remove_queue(F_OUT);
    let _ = posixmq::remove_queue(G_OUT);
}

fn send(name: &str, x: i32) {
    let mq = posixmq::OpenOptions::readwrite()
        .capacity(10)
        .max_msg_len(4)
        .nonblocking()
        .create()
        .open(name)
        .unwrap();
    mq.send(0, &x.to_be_bytes()).unwrap();
}

fn read_input(name: &str) -> i32 {
    let mq = posixmq::OpenOptions::readwrite()
        .capacity(10)
        .max_msg_len(4)
        .nonblocking()
        .create()
        .open(name)
        .unwrap();

    let mut buf = [0; 4];
    mq.recv(&mut buf).unwrap();
    i32::from_be_bytes(buf)
}

fn f(x: i32) -> i32 {
    if x == 10 {
        loop {}
    }
    x
}

fn g(x: i32) -> i32 {
    if x == 10 {
        loop {}
    }
    x
}

fn main() {

    let mut mode = String::new();
    std::io::stdin().read_line(&mut mode).unwrap();
    match mode.as_str().trim() {
        "main" => {
            clean_up();
            let mut line = String::new();
            std::io::stdin().read_line(&mut line).unwrap();
            let x: i32 = line.trim().parse().unwrap();
            send(F_IN, x);
            send(G_IN, x);
            let process = |f_name: String,
                           mq_name: String,
                           pair: Arc<(Mutex<Option<bool>>, Condvar)>| {
                let mut buf = [0; 4];
                let mut ask = true;
                loop {
                    let mut mq = posixmq::OpenOptions::readwrite()
                        .capacity(10)
                        .max_msg_len(4)
                        .nonblocking()
                        .create()
                        .open(&mq_name)
                        .unwrap();
                    let res = if ask {
                        let mut res = mq.recv(&mut buf);
                        for _ in 0..5 {
                            if res.is_ok() {
                                break;
                            } else {
                                std::thread::sleep(Duration::from_secs(1));
                            }
                            res = mq.recv(&mut buf);
                        }
                        res
                    } else {
                        let mut res = mq.recv(&mut buf);
                        loop {
                            if res.is_ok() {
                                break res;
                            } else {
                                std::thread::sleep(Duration::from_secs(1));
                            }
                            res = mq.recv(&mut buf);
                        }
                    };
                    match res {
                        Ok(_) => {
                            let res = i32::from_be_bytes(buf);
                            let (lock, cvar) = &*pair;
                            let mut value = lock.lock().unwrap();
                            if let Some(value) = value.as_mut() {
                                *value &= !(res == 0);
                                cvar.notify_one();
                            } else {
                                if res == 0 {
                                    *value = Some(false);
                                    cvar.notify_one();
                                } else {
                                    *value = Some(true);
                                }
                            }
                            return;
                        }
                        Err(err) if err.kind() == ErrorKind::WouldBlock => {
                            println!("{} is still running, print 1 if you want to continue, 2 if you want to continue and don't receive this message anymore, anything else to stop", f_name);
                            let mut line = String::new();
                            std::io::stdin().read_line(&mut line).unwrap();
                            match line.as_str().trim() {
                                "1" => {}
                                "2" => ask = false,
                                _ => {
                                    println!("Stopping..");
                                    exit(0);
                                }
                            }
                        }
                        Err(err) => panic!("{}", err),
                    }
                }
            };

            let pair = Arc::new((Mutex::new(None), Condvar::new()));
            let pair_f = pair.clone();
            let pair_g = pair.clone();
            let f_handle =
                std::thread::spawn(move || process("f".to_string(), F_OUT.to_string(), pair_f));
            let g_handle =
                std::thread::spawn(move || process("g".to_string(), G_OUT.to_string(), pair_g));

            let (lock, cvar) = &*pair;
            let mut res = lock.lock().unwrap();
            while res.is_none() {
                res = cvar.wait(res).unwrap();
            }
            println!("Result: {}", res.unwrap());

            f_handle.join().unwrap();
            g_handle.join().unwrap();
        }
        "f" => {
            let res = f(read_input(F_IN));
            send(F_OUT, res);
        }
        "g" => {
            let res = g(read_input(G_IN));
            send(G_OUT, res);
        }
        _ => {
            println!("Available modes are `main`, `f`, `g`");
        }
    }
}
