use std::{ffi::OsString, time::Duration};

fn main() {
    //let output = othello_gui::run::run(OsString::from("test_programs\\hang.exe"), b"3\n");
    let output = othello_gui::run::run_timeout(
        OsString::from("test_programs\\cat.exe"),
        b"3\n",
        Duration::from_secs(3),
    );

    println!("{:?}", output);

    /*if let Ok(res) = output {
        println!("{}", String::from_utf8(res).unwrap());
    }*/
}
